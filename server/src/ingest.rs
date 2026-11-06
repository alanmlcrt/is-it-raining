use std::{sync::Arc, time::Duration as StdDuration};

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};
use grib::def::grib2::GridDefinitionTemplate;
use reqwest::{
    StatusCode,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue, RETRY_AFTER},
};
use serde::Deserialize;
use tokio::{
    sync::watch,
    time::{Instant, sleep, sleep_until},
};

use crate::forecast::{Grid, STEP_MINUTES};

type Error = Box<dyn std::error::Error + Send + Sync>;

const TOKEN_URL: &str = "https://portail-api.meteofrance.fr/token";
const WCS_URL: &str =
    "https://api.meteofrance.fr/pro/piaf/1.0/wcs/MF-NWP-HIGHRES-PIAF-001-FRANCE-WCS";
const COVERAGE_PREFIX: &str = "TOTAL_PRECIPITATION_RATE__GROUND_OR_WATER_SURFACE___";
const COVERAGE_TIME: &str = "%Y-%m-%dT%H.%M.%SZ";

/// Une requête par échéance, de 300 à 11 700 s.
const STEPS: u32 = 39;
/// Un cumul `PT5M` en mm vers des mm/h.
const PT5M_TO_MM_H: f32 = 12.0;
/// Écart entre deux appels : ~85 req/min au plus, sous la limite de 100, et un run en ~30 s.
const CALL_GAP: StdDuration = StdDuration::from_millis(700);
const POLL: StdDuration = StdDuration::from_secs(30);
/// Rien à chercher avant : le run suivant sort 5 min plus tard, 10 min après son horaire.
const NEXT_RUN_AFTER_MIN: i64 = 14;
const TOKEN_MARGIN: StdDuration = StdDuration::from_secs(300);
const MAX_TRIES: u32 = 3;
const DEFAULT_RETRY_AFTER: u64 = 30;
const MAX_RETRY_AFTER: u64 = 300;

/// `lon_min,lon_max,lat_min,lat_max` vers les paramètres `subset` du WCS.
pub fn parse_bbox(bbox: &str) -> Option<String> {
    let v = bbox
        .split(',')
        .map(|s| s.trim().parse::<f64>().ok())
        .collect::<Option<Vec<_>>>()?;
    let &[lon_min, lon_max, lat_min, lat_max] = v.as_slice() else {
        return None;
    };
    (lon_min < lon_max && lat_min < lat_max)
        .then(|| format!("subset=long({lon_min},{lon_max})&subset=lat({lat_min},{lat_max})"))
}

/// Le run `PT5M` le plus récent de `GetCapabilities`.
///
/// `ponytail:` découpe de texte plutôt qu'un parseur XML : on ne lit qu'un seul élément.
fn latest_run(capabilities: &str) -> Option<DateTime<Utc>> {
    capabilities
        .split("<wcs:CoverageId>")
        .skip(1)
        .filter_map(|s| {
            let id = s.split('<').next()?;
            let time = id.strip_prefix(COVERAGE_PREFIX)?.strip_suffix("_PT5M")?;
            NaiveDateTime::parse_from_str(time, COVERAGE_TIME).ok()
        })
        .max()
        .map(|t| t.and_utc())
}

struct Field {
    lat0: f64,
    lon0: f64,
    ni: usize,
    nj: usize,
    mm_h: Vec<f32>,
}

/// Décode une échéance. La géométrie vient du fichier, car WCS recale la zone demandée sur la
/// grille. Refuse tout ce qui ne correspond pas à ce qu'on a demandé.
fn decode(bytes: &[u8], run_at: DateTime<Utc>, step: u32) -> Result<Field, Error> {
    let grib = grib::from_bytes(bytes)?;
    let (_, sub) = grib.iter().next().ok_or("GRIB vide")?;

    let GridDefinitionTemplate::_3_0(template) = sub.section3()?.payload.template else {
        return Err("GRIB : grille non lat/lon".into());
    };
    let grid = template.lat_lon;
    if grid.scanning_mode.0 != 0 || grid.i_direction_inc != 10_000 || grid.j_direction_inc != 10_000
    {
        return Err("GRIB : balayage ou pas de grille inattendu".into());
    }

    let raw = sub.temporal_raw_info();
    let time = &raw.ref_time_unchecked;
    let ref_time = NaiveDate::from_ymd_opt(time.year.into(), time.month.into(), time.day.into())
        .and_then(|d| d.and_hms_opt(time.hour.into(), time.minute.into(), time.second.into()))
        .map(|t| t.and_utc());
    if ref_time != Some(run_at) {
        return Err("GRIB : run inattendu".into());
    }
    // Le début du cumul : l'échéance 300 s couvre [run, run + 5 min].
    let start = raw.forecast_time_diff.as_ref().map(|f| i64::from(f.value));
    if start != Some(i64::from(step) * STEP_MINUTES) {
        return Err("GRIB : échéance inattendue".into());
    }

    let (ni, nj) = (grid.grid.ni as usize, grid.grid.nj as usize);
    let mm_h: Vec<f32> = grib::Grib2SubmessageDecoder::from(sub)?
        .dispatch()?
        .map(|v| v * PT5M_TO_MM_H)
        .collect();
    if mm_h.len() != ni * nj {
        return Err("GRIB : taille de grille incohérente".into());
    }

    // La longitude est codée de 0 à 360 : -6° arrive en 354°.
    let lon = f64::from(grid.grid.first_point_lon) / 1e6;
    Ok(Field {
        lat0: f64::from(grid.grid.first_point_lat) / 1e6,
        lon0: if lon > 180.0 { lon - 360.0 } else { lon },
        ni,
        nj,
        mm_h,
    })
}

struct Token {
    value: String,
    renew_at: Instant,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

/// Client PIAF. `&mut self` partout : une seule tâche l'utilise, ce qui donne à lui seul le
/// « un seul renouvellement à la fois ». Pas de `Debug` : il porte le secret.
pub struct Piaf {
    http: reqwest::Client,
    token_url: String,
    wcs_url: String,
    call_gap: StdDuration,
    app_id: String,
    subset: String,
    token: Option<Token>,
    next_call: Instant,
    requests: u64,
    grib_bytes: u64,
}

impl Piaf {
    pub fn new(app_id: String, subset: String) -> Self {
        Piaf {
            http: reqwest::Client::builder()
                .timeout(StdDuration::from_secs(30))
                .build()
                .expect("client HTTP"),
            token_url: TOKEN_URL.to_string(),
            wcs_url: WCS_URL.to_string(),
            call_gap: CALL_GAP,
            app_id,
            subset,
            token: None,
            next_call: Instant::now(),
            requests: 0,
            grib_bytes: 0,
        }
    }

    async fn token(&mut self) -> Result<String, Error> {
        if let Some(token) = &self.token
            && Instant::now() < token.renew_at
        {
            return Ok(token.value.clone());
        }
        let mut auth = HeaderValue::from_str(&format!("Basic {}", self.app_id))
            .map_err(|_| "METEOFRANCE_APP_ID illisible")?;
        auth.set_sensitive(true);
        let res = self
            .http
            .post(&self.token_url)
            .header(AUTHORIZATION, auth)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body("grant_type=client_credentials")
            .send()
            .await?;
        self.requests += 1;
        if !res.status().is_success() {
            // Le corps n'est pas journalisé : il pourrait reprendre l'identifiant.
            return Err(format!("/token : {}", res.status()).into());
        }
        let token: TokenResponse = res.json().await?;
        let life = StdDuration::from_secs(token.expires_in).saturating_sub(TOKEN_MARGIN);
        self.token = Some(Token {
            value: token.access_token.clone(),
            renew_at: Instant::now() + life,
        });
        eprintln!("ingest : nouveau jeton, valable {} s", token.expires_in);
        Ok(token.access_token)
    }

    /// Un `GET` PIAF : espacé, jeton renouvelé sur `401`, nouvel essai sur `429` et `5xx`.
    async fn get(&mut self, url: &str) -> Result<reqwest::Response, Error> {
        let mut renewed = false;
        for attempt in 1..=MAX_TRIES {
            sleep_until(self.next_call).await;
            self.next_call = Instant::now() + self.call_gap;
            let token = self.token().await?;
            let res = self.http.get(url).bearer_auth(token).send().await?;
            self.requests += 1;
            let status = res.status();
            if status.is_success() {
                return Ok(res);
            } else if status == StatusCode::UNAUTHORIZED && !renewed {
                eprintln!("ingest : 401, jeton renouvelé");
                self.token = None;
                renewed = true;
            } else if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                if attempt == MAX_TRIES {
                    break;
                }
                let wait = res
                    .headers()
                    .get(RETRY_AFTER)
                    .and_then(|v| v.to_str().ok()?.parse().ok())
                    .unwrap_or(DEFAULT_RETRY_AFTER)
                    .min(MAX_RETRY_AFTER);
                eprintln!("ingest : {status}, nouvel essai dans {wait} s");
                sleep(StdDuration::from_secs(wait)).await;
            } else {
                return Err(format!("PIAF : {status}").into());
            }
        }
        Err(format!("PIAF : abandon après {MAX_TRIES} essais").into())
    }

    /// Le dernier run publié. Réponse en gzip : ~60 Ko, négligeable devant un run.
    async fn latest_run(&mut self) -> Result<DateTime<Utc>, Error> {
        let url = format!(
            "{}/GetCapabilities?service=WCS&version=2.0.1&language=eng",
            self.wcs_url
        );
        let xml = self.get(&url).await?.text().await?;
        latest_run(&xml).ok_or_else(|| "GetCapabilities : aucun run PT5M".into())
    }

    /// Les 39 échéances d'un run, ou rien : une seule erreur abandonne tout le run.
    async fn fetch_run(&mut self, run_at: DateTime<Utc>) -> Result<Grid, Error> {
        let id = format!("{COVERAGE_PREFIX}{}_PT5M", run_at.format(COVERAGE_TIME));
        let mut steps = Vec::with_capacity(STEPS as usize);
        let mut first: Option<(f64, f64, usize, usize)> = None;
        for step in 0..STEPS {
            let url = format!(
                "{}/GetCoverage?service=WCS&version=2.0.1&coverageid={id}&subset=time({})&{}&format=application/wmo-grib",
                self.wcs_url,
                (step + 1) * 300,
                self.subset
            );
            let bytes = self.get(&url).await?.bytes().await?;
            self.grib_bytes += bytes.len() as u64;
            // `ponytail:` décodage sur la tâche async, 71 ms par champ France entière.
            // À sortir dans `spawn_blocking` si la latence de l'API en souffre.
            let field = decode(&bytes, run_at, step)?;
            let shape = (field.lat0, field.lon0, field.ni, field.nj);
            if *first.get_or_insert(shape) != shape {
                return Err("GRIB : grille différente entre deux échéances".into());
            }
            steps.push(field.mm_h);
        }
        let (lat0, lon0, ni, nj) = first.ok_or("run sans échéance")?;
        Ok(Grid {
            run_at,
            lat0,
            lon0,
            ni,
            nj,
            steps,
        })
    }
}

/// Boucle d'ingestion. Une erreur laisse la grille en place : c'est `MAX_DATA_AGE_MIN` qui la
/// fait passer en `unknown`. S'arrête après `max_runs` runs ; l'API, elle, continue.
pub async fn run(mut piaf: Piaf, grid: watch::Sender<Option<Arc<Grid>>>, max_runs: Option<u32>) {
    let mut last: Option<DateTime<Utc>> = None;
    let mut tried: Option<DateTime<Utc>> = None;
    let mut done = 0;
    loop {
        if max_runs.is_some_and(|max| done >= max) {
            eprintln!("ingest : PIAF_MAX_RUNS atteint, arrêt de l'ingestion");
            return;
        }
        let wait_until = tried.map(|t| t + Duration::minutes(NEXT_RUN_AFTER_MIN));
        if wait_until.is_none_or(|t| Utc::now() >= t) {
            match ingest_new(&mut piaf, &mut tried).await {
                Ok(Some(new)) => {
                    let run_at = new.run_at;
                    log_run(&piaf, run_at, last);
                    grid.send_replace(Some(Arc::new(new)));
                    last = Some(run_at);
                    done += 1;
                    continue;
                }
                Ok(None) => {}
                Err(e) => eprintln!("ingest : {e}, grille précédente gardée"),
            }
        }
        sleep(POLL).await;
    }
}

/// Un seul essai par run : un run qui échoue à chaque fois ne doit pas être retéléchargé en
/// boucle, le pire cas reste ainsi le volume normal d'un run toutes les 5 min.
async fn ingest_new(
    piaf: &mut Piaf,
    tried: &mut Option<DateTime<Utc>>,
) -> Result<Option<Grid>, Error> {
    let run_at = piaf.latest_run().await?;
    if tried.is_some_and(|t| run_at <= t) {
        return Ok(None);
    }
    *tried = Some(run_at);
    piaf.fetch_run(run_at).await.map(Some)
}

/// Volume téléchargé depuis le démarrage, retard du run et runs manqués.
fn log_run(piaf: &Piaf, run_at: DateTime<Utc>, last: Option<DateTime<Utc>>) {
    let age = (Utc::now() - run_at).num_seconds();
    let grib_mb = piaf.grib_bytes as f64 / 1e6;
    eprintln!(
        "ingest : run {run_at} en place, âge {age} s, total {grib_mb:.2} Mo de GRIB, {} requêtes",
        piaf.requests
    );
    if let Some(last) = last {
        let missed = (run_at - last).num_minutes() / STEP_MINUTES - 1;
        if missed > 0 {
            eprintln!("ingest : TROU, {missed} run(s) manqué(s) entre {last} et {run_at}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing};
    use serde::Serialize;
    use tokio::net::TcpListener;

    const FIXTURE: &[u8] =
        include_bytes!("../tests/fixtures/piaf-2026-11-05T1955Z-step300-alpes.grib2");

    fn fixture_run() -> DateTime<Utc> {
        "2026-11-05T19:55:00Z".parse().unwrap()
    }

    #[test]
    fn decode_un_vrai_fichier_piaf() {
        let f = decode(FIXTURE, fixture_run(), 0).unwrap();
        assert_eq!((f.ni, f.nj), (21, 21));
        assert_eq!((f.lat0, f.lon0), (45.1, 6.4));
        // Référence : GeoTIFF du même run, lu séparément, cumuls en mm.
        // Coins opposés différents : une grille retournée échoue.
        let mm = |row: usize, col: usize| f.mm_h[row * f.ni + col] / PT5M_TO_MM_H;
        assert!((mm(20, 0) - 0.27001953).abs() < 1e-4, "44,90 N 6,40 E");
        assert!((mm(0, 20) - 0.02001953).abs() < 1e-4, "45,10 N 6,60 E");
        assert!((mm(10, 10) - 0.07006836).abs() < 1e-4, "45,00 N 6,50 E");
        assert!((mm(5, 15) - 0.04003906).abs() < 1e-4, "45,05 N 6,55 E");
    }

    #[test]
    fn decode_refuse_un_autre_run_ou_une_autre_echeance() {
        let other = fixture_run() + Duration::minutes(5);
        assert!(decode(FIXTURE, other, 0).is_err());
        assert!(decode(FIXTURE, fixture_run(), 1).is_err());
        assert!(decode(b"pas un grib", fixture_run(), 0).is_err());
    }

    #[test]
    fn dernier_run_pt5m() {
        let id =
            |t: &str, d: &str| format!("<wcs:CoverageId>{COVERAGE_PREFIX}{t}_{d}</wcs:CoverageId>");
        let xml = [
            id("2026-11-05T19.50.00Z", "PT5M"),
            id("2026-11-05T19.55.00Z", "PT5M"),
            id("2026-11-05T19.45.00Z", "PT5M"),
            id("2026-11-05T20.00.00Z", "PT15M"),
        ]
        .concat();
        assert_eq!(latest_run(&xml), Some(fixture_run()));
        assert_eq!(latest_run("<xml/>"), None);
    }

    #[test]
    fn bbox() {
        assert_eq!(
            parse_bbox("2.0, 2.7,48.6,49.1").as_deref(),
            Some("subset=long(2,2.7)&subset=lat(48.6,49.1)")
        );
        assert_eq!(parse_bbox("2,1,48,49"), None);
        assert_eq!(parse_bbox("2,3,48"), None);
        assert_eq!(parse_bbox("a,b,c,d"), None);
    }

    /// Faux Météo-France. `script` : statut de chaque appel PIAF successif, puis `200`.
    struct Fake {
        tokens: AtomicU32,
        calls: AtomicU32,
        coverages: AtomicU32,
        expires_in: u64,
        script: Vec<u16>,
    }

    #[derive(Serialize)]
    struct FakeToken {
        access_token: String,
        expires_in: u64,
    }

    async fn fake_token(State(fake): State<Arc<Fake>>) -> Json<FakeToken> {
        let n = fake.tokens.fetch_add(1, Ordering::SeqCst);
        Json(FakeToken {
            access_token: format!("jeton-{n}"),
            expires_in: fake.expires_in,
        })
    }

    async fn fake_wcs(State(fake): State<Arc<Fake>>) -> impl IntoResponse {
        let n = fake.calls.fetch_add(1, Ordering::SeqCst) as usize;
        let code = fake.script.get(n).copied().unwrap_or(200);
        (
            StatusCode::from_u16(code).unwrap(),
            [("retry-after", "0")],
            "ok",
        )
    }

    async fn fake_capabilities() -> String {
        format!("<wcs:CoverageId>{COVERAGE_PREFIX}2026-11-05T19.55.00Z_PT5M</wcs:CoverageId>")
    }

    /// La première échéance est valide, la deuxième illisible.
    async fn fake_coverage(State(fake): State<Arc<Fake>>) -> Vec<u8> {
        match fake.coverages.fetch_add(1, Ordering::SeqCst) {
            0 => FIXTURE.to_vec(),
            _ => b"illisible".to_vec(),
        }
    }

    async fn client(expires_in: u64, script: &[u16]) -> (Piaf, Arc<Fake>) {
        let fake = Arc::new(Fake {
            tokens: AtomicU32::new(0),
            calls: AtomicU32::new(0),
            coverages: AtomicU32::new(0),
            expires_in,
            script: script.to_vec(),
        });
        let app = Router::new()
            .route("/token", routing::post(fake_token))
            .route("/wcs", routing::get(fake_wcs))
            .route("/wcs/GetCapabilities", routing::get(fake_capabilities))
            .route("/wcs/GetCoverage", routing::get(fake_coverage))
            .with_state(fake.clone());
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let mut piaf = Piaf::new("secret".to_string(), String::new());
        piaf.token_url = format!("http://{addr}/token");
        piaf.wcs_url = format!("http://{addr}/wcs");
        piaf.call_gap = StdDuration::ZERO;
        (piaf, fake)
    }

    #[tokio::test]
    async fn jeton_reutilise_tant_qu_il_est_valide() {
        let (mut piaf, fake) = client(3600, &[]).await;
        let url = piaf.wcs_url.clone();
        piaf.get(&url).await.unwrap();
        piaf.get(&url).await.unwrap();
        assert_eq!(fake.tokens.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn jeton_renouvele_avant_expiration() {
        // Durée de vie sous la marge : à renouveler à chaque appel.
        let (mut piaf, fake) = client(60, &[]).await;
        let url = piaf.wcs_url.clone();
        piaf.get(&url).await.unwrap();
        piaf.get(&url).await.unwrap();
        assert_eq!(fake.tokens.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn jeton_renouvele_sur_401() {
        let (mut piaf, fake) = client(3600, &[401]).await;
        let url = piaf.wcs_url.clone();
        piaf.get(&url).await.unwrap();
        assert_eq!(fake.tokens.load(Ordering::SeqCst), 2);
        assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn deux_401_abandonnent() {
        let (mut piaf, fake) = client(3600, &[401, 401]).await;
        let url = piaf.wcs_url.clone();
        assert!(piaf.get(&url).await.is_err());
        assert_eq!(fake.calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn nouvel_essai_sur_429() {
        let (mut piaf, fake) = client(3600, &[429, 503]).await;
        let url = piaf.wcs_url.clone();
        piaf.get(&url).await.unwrap();
        assert_eq!(fake.calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn abandon_apres_trois_429() {
        let (mut piaf, fake) = client(3600, &[429, 429, 429]).await;
        let url = piaf.wcs_url.clone();
        assert!(piaf.get(&url).await.is_err());
        assert_eq!(fake.calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn pas_de_nouvel_essai_sur_404() {
        let (mut piaf, fake) = client(3600, &[404]).await;
        let url = piaf.wcs_url.clone();
        assert!(piaf.get(&url).await.is_err());
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn max_runs_zero_ne_telecharge_rien() {
        let (piaf, fake) = client(3600, &[]).await;
        let (tx, _rx) = watch::channel(None);
        run(piaf, tx, Some(0)).await;
        assert_eq!(fake.tokens.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn run_incomplet_rejete() {
        // Une échéance valide puis une illisible : rien n'est livré, on s'arrête là.
        let (mut piaf, fake) = client(3600, &[]).await;
        assert!(piaf.fetch_run(fixture_run()).await.is_err());
        assert_eq!(fake.coverages.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn un_run_en_echec_n_est_pas_retelecharge() {
        let (mut piaf, fake) = client(3600, &[]).await;
        let mut tried = None;
        assert!(ingest_new(&mut piaf, &mut tried).await.is_err());
        assert_eq!(tried, Some(fixture_run()));
        assert!(ingest_new(&mut piaf, &mut tried).await.unwrap().is_none());
        assert_eq!(fake.coverages.load(Ordering::SeqCst), 2);
    }
}
