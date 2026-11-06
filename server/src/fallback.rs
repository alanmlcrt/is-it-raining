//! Repli Open-Meteo : hors zone PIAF, ou quand PIAF manque. Voir D1.

use std::{collections::HashMap, sync::Mutex, time::Duration as StdDuration};

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::Deserialize;

use crate::forecast::{self, Params, Rain, Series};

const URL: &str = "https://api.open-meteo.com/v1/forecast";
const SOURCE: &str = "open_meteo";
const STEP_MINUTES: i64 = 15;
/// Deux pas passés pour voir une averse en cours, treize à venir pour l'horizon de PIAF.
const PAST_STEPS: u32 = 2;
const FORECAST_STEPS: u32 = 13;
/// Un cumul sur 15 min en mm vers des mm/h.
const MM_15_TO_MM_H: f32 = 4.0;
const CACHE_MIN: i64 = 15;
/// Environ 11 km : le cache se partage par zone, et Open-Meteo ne voit que le centre de la zone.
const ZONE_DEGREES: f64 = 0.1;
/// Sous les 10 000 appels par jour de la licence gratuite (D1), même si la route est martelée.
const DAILY_CALLS: u32 = 9_000;
const TIMEOUT: StdDuration = StdDuration::from_secs(4);

#[derive(Deserialize)]
struct Body {
    minutely_15: Minutely,
}

#[derive(Deserialize)]
struct Minutely {
    time: Vec<i64>,
    precipitation: Vec<Option<f32>>,
}

#[derive(Clone)]
struct Forecast {
    fetched_at: DateTime<Utc>,
    start: DateTime<Utc>,
    mm_h: Vec<f32>,
}

#[derive(Default)]
struct Cache {
    zones: HashMap<(i32, i32), Forecast>,
    day: Option<NaiveDate>,
    calls: u32,
}

pub struct Fallback {
    url: String,
    daily_calls: u32,
    client: reqwest::Client,
    cache: Mutex<Cache>,
}

/// Début de la série et valeurs en mm/h. `None` si un pas manque : un trou n'est pas du temps sec.
fn parse(m: Minutely) -> Option<(DateTime<Utc>, Vec<f32>)> {
    let step = STEP_MINUTES * 60;
    let regular = m.time.windows(2).all(|t| t[1] - t[0] == step);
    if m.time.is_empty() || m.time.len() != m.precipitation.len() || !regular {
        return None;
    }
    let mm_h = m
        .precipitation
        .iter()
        .map(|v| v.filter(|v| v.is_finite() && *v >= 0.0))
        .map(|v| v.map(|v| v * MM_15_TO_MM_H))
        .collect::<Option<Vec<_>>>()?;
    // Chaque valeur couvre les 15 minutes qui précèdent son horodatage.
    let start = DateTime::from_timestamp(m.time[0] - step, 0)?;
    Some((start, mm_h))
}

impl Fallback {
    pub fn new() -> Self {
        Fallback {
            url: URL.to_string(),
            daily_calls: DAILY_CALLS,
            client: reqwest::Client::builder()
                .timeout(TIMEOUT)
                .build()
                .expect("client HTTP"),
            cache: Mutex::default(),
        }
    }

    /// `None` si Open-Meteo ne répond pas ou si le quota du jour est épuisé.
    pub async fn rain(&self, lat: f64, lon: f64, now: DateTime<Utc>, p: &Params) -> Option<Rain> {
        let zone = (
            (lat / ZONE_DEGREES).round() as i32,
            (lon / ZONE_DEGREES).round() as i32,
        );
        let max_age = Duration::minutes(CACHE_MIN).min(p.max_data_age);
        let cached = self.fresh(zone, now, max_age);
        let forecast = match cached {
            Some(forecast) => forecast,
            None => {
                self.take_call(now)?;
                let forecast = self.fetch(zone, now).await?;
                let mut cache = self.cache.lock().ok()?;
                cache.zones.retain(|_, f| now - f.fetched_at <= max_age);
                cache.zones.insert(zone, forecast.clone());
                forecast
            }
        };
        Some(forecast::from_series(
            &Series {
                source: SOURCE,
                step_minutes: STEP_MINUTES,
                start: forecast.start,
                // ponytail: Open-Meteo ne donne pas l'heure de son run. On cite celle de l'appel.
                updated_at: forecast.fetched_at,
                mm_h: &forecast.mm_h,
            },
            now,
            p,
        ))
    }

    fn fresh(&self, zone: (i32, i32), now: DateTime<Utc>, max_age: Duration) -> Option<Forecast> {
        let cache = self.cache.lock().ok()?;
        cache
            .zones
            .get(&zone)
            .filter(|f| now - f.fetched_at <= max_age)
            .cloned()
    }

    fn take_call(&self, now: DateTime<Utc>) -> Option<()> {
        let mut cache = self.cache.lock().ok()?;
        if cache.day != Some(now.date_naive()) {
            cache.day = Some(now.date_naive());
            cache.calls = 0;
        }
        if cache.calls >= self.daily_calls {
            return None;
        }
        cache.calls += 1;
        Some(())
    }

    /// Ne pas journaliser l'erreur de `reqwest` : son texte contient l'URL, donc la zone.
    async fn fetch(&self, zone: (i32, i32), now: DateTime<Utc>) -> Option<Forecast> {
        let url = format!(
            "{}?latitude={:.1}&longitude={:.1}&minutely_15=precipitation\
             &forecast_minutely_15={FORECAST_STEPS}&past_minutely_15={PAST_STEPS}\
             &timezone=GMT&timeformat=unixtime",
            self.url,
            f64::from(zone.0) * ZONE_DEGREES,
            f64::from(zone.1) * ZONE_DEGREES,
        );
        let body = async {
            let res = self.client.get(url).send().await?;
            res.error_for_status()?.json::<Body>().await
        };
        let parsed = match body.await {
            Ok(body) => parse(body.minutely_15),
            Err(_) => None,
        };
        if parsed.is_none() {
            eprintln!("repli : pas de réponse exploitable d'Open-Meteo");
        }
        let (start, mm_h) = parsed?;
        Some(Forecast {
            fetched_at: now,
            start,
            mm_h,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    };

    use axum::{Router, extract::RawQuery, routing::get};
    use tokio::net::TcpListener;

    use super::*;
    use crate::forecast::State;

    /// Une réponse qui démarre juste avant `now`, en mm sur 15 min.
    fn body(now: DateTime<Utc>, mm: &[f32]) -> String {
        let t0 = now.timestamp() / 900 * 900;
        let time: Vec<i64> = (0..mm.len() as i64).map(|i| t0 + i * 900).collect();
        format!(r#"{{"minutely_15":{{"time":{time:?},"precipitation":{mm:?}}}}}"#)
    }

    /// Un faux Open-Meteo. Compte les appels et garde la dernière requête.
    pub async fn fake(mm: Vec<f32>) -> (Fallback, Arc<AtomicU32>, Arc<Mutex<String>>) {
        let calls = Arc::new(AtomicU32::new(0));
        let query = Arc::new(Mutex::new(String::new()));
        let (c, q) = (calls.clone(), query.clone());
        let app = Router::new().route(
            "/",
            get(move |RawQuery(raw): RawQuery| {
                c.fetch_add(1, Ordering::SeqCst);
                *q.lock().unwrap() = raw.unwrap_or_default();
                let mm = mm.clone();
                async move { body(Utc::now(), &mm) }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let mut fallback = Fallback::new();
        fallback.url = format!("http://{addr}/");
        (fallback, calls, query)
    }

    /// Personne n'écoute sur ce port : Open-Meteo est en panne.
    pub fn down() -> Fallback {
        let mut fallback = Fallback::new();
        fallback.url = "http://127.0.0.1:1/".to_string();
        fallback
    }

    fn params() -> Params {
        Params {
            rain_threshold_mm_h: 0.1,
            min_wet_steps: 2,
            max_data_age: Duration::minutes(20),
        }
    }

    #[tokio::test]
    async fn source_et_pas_du_repli() {
        let (fallback, _, _) = fake(vec![0.0; 15]).await;
        let r = fallback
            .rain(50.63, 3.06, Utc::now(), &params())
            .await
            .unwrap();
        assert_eq!(r.state, State::Dry);
        assert_eq!(r.source, Some("open_meteo"));
        assert_eq!(r.step_minutes, Some(15));
        assert_eq!(r.series.len(), 15);
    }

    #[tokio::test]
    async fn cumul_converti_et_un_seul_pas_suffit() {
        // Sec, puis un seul pas à 1 mm en 15 min : soit 4 mm/h, sur un pas de 15 min.
        let (fallback, _, _) = fake(vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0]).await;
        let r = fallback
            .rain(50.63, 3.06, Utc::now(), &params())
            .await
            .unwrap();
        assert_eq!(r.state, State::RainSoon);
        assert_eq!(r.series[4].mm_h, 4.0);
    }

    #[tokio::test]
    async fn cache_par_zone_et_centre_de_zone_seulement() {
        let (fallback, calls, query) = fake(vec![0.0; 15]).await;
        let now = Utc::now();
        fallback.rain(50.63, 3.06, now, &params()).await.unwrap();
        fallback.rain(50.61, 3.08, now, &params()).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let sent = query.lock().unwrap().clone();
        assert!(sent.contains("latitude=50.6&longitude=3.1"), "{sent}");

        let later = now + Duration::minutes(CACHE_MIN + 1);
        fallback.rain(50.63, 3.06, later, &params()).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn quota_du_jour_respecte() {
        let (mut fallback, calls, _) = fake(vec![0.0; 15]).await;
        fallback.daily_calls = 1;
        let now = Utc::now();
        assert!(fallback.rain(50.63, 3.06, now, &params()).await.is_some());
        assert!(fallback.rain(45.0, 5.0, now, &params()).await.is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let tomorrow = now + Duration::days(1);
        assert!(
            fallback
                .rain(45.0, 5.0, tomorrow, &params())
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn panne_sans_reponse() {
        assert!(
            down()
                .rain(50.63, 3.06, Utc::now(), &params())
                .await
                .is_none()
        );
    }

    #[test]
    fn un_trou_n_est_pas_du_sec() {
        let m = |time: Vec<i64>, precipitation| Minutely {
            time,
            precipitation,
        };
        assert!(parse(m(vec![0, 900], vec![Some(0.0), None])).is_none());
        assert!(parse(m(vec![0, 900], vec![Some(0.0)])).is_none());
        assert!(parse(m(vec![0, 1800], vec![Some(0.0), Some(0.0)])).is_none());
        assert!(parse(m(vec![], vec![])).is_none());
        assert!(parse(m(vec![900], vec![Some(-1.0)])).is_none());
        let (start, mm_h) = parse(m(vec![900, 1800], vec![Some(0.5), Some(0.0)])).unwrap();
        assert_eq!(start.timestamp(), 0);
        assert_eq!(mm_h, vec![2.0, 0.0]);
    }
}
