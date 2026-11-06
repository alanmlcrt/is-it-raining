use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use tokio::sync::watch;

use crate::{
    fallback::Fallback,
    forecast::{self, Grid, Params, Rain, State as RainState},
};

/// Un run arrive 10 min après son horaire, et le suivant 5 min plus tard.
const NEXT_RUN_AFTER_MIN: i64 = 15;

#[derive(Clone)]
struct App {
    grid: watch::Receiver<Option<Arc<Grid>>>,
    params: Params,
    fallback: Arc<Fallback>,
}

#[derive(Deserialize)]
struct Point {
    lat: f64,
    lon: f64,
}

pub fn router(
    grid: watch::Receiver<Option<Arc<Grid>>>,
    params: Params,
    fallback: Fallback,
) -> Router {
    Router::new().route("/v1/rain", get(rain)).with_state(App {
        grid,
        params,
        fallback: Arc::new(fallback),
    })
}

/// Ne jamais journaliser `lat` et `lon` ici.
async fn rain(State(app): State<App>, Query(p): Query<Point>) -> Response {
    // `contains` écarte aussi NaN.
    if !(-90.0..=90.0).contains(&p.lat) || !(-180.0..=180.0).contains(&p.lon) {
        return (StatusCode::BAD_REQUEST, "lat ou lon hors bornes").into_response();
    }
    let grid = app.grid.borrow().clone();
    let now = Utc::now();
    let mut rain = forecast::rain_at(grid.as_deref(), p.lat, p.lon, now, &app.params);
    // Hors zone ou PIAF en panne. Sans repli non plus, la réponse reste `unknown`.
    if rain.state == RainState::Unknown
        && let Some(fallback) = app.fallback.rain(p.lat, p.lon, now, &app.params).await
    {
        rain = fallback;
    }
    (
        [(
            header::CACHE_CONTROL,
            cache_control(&rain, now, app.params.max_data_age),
        )],
        Json(rain),
    )
        .into_response()
}

/// En cache jusqu'au prochain run attendu, donc jamais au-delà de la fraîcheur de la donnée.
fn cache_control(rain: &Rain, now: DateTime<Utc>, max_data_age: Duration) -> String {
    match rain.run_at.filter(|_| !rain.series.is_empty()) {
        Some(run_at) => {
            let next = run_at + Duration::minutes(NEXT_RUN_AFTER_MIN).min(max_data_age);
            format!(
                "private, max-age={}",
                (next - now).num_seconds().clamp(0, 300)
            )
        }
        None => "no-store".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fallback::tests::{down, fake};
    use tokio::net::TcpListener;

    #[derive(Deserialize)]
    struct Body {
        state: String,
        source: Option<String>,
        step_minutes: Option<i64>,
        series: Vec<Step>,
    }

    #[derive(Deserialize)]
    struct Step {
        mm_h: f32,
    }

    async fn serve(grid: Option<Grid>) -> String {
        serve_with(grid, down()).await
    }

    async fn serve_with(grid: Option<Grid>, fallback: Fallback) -> String {
        let (tx, rx) = watch::channel(grid.map(Arc::new));
        let params = Params {
            rain_threshold_mm_h: 0.1,
            min_wet_steps: 2,
            max_data_age: Duration::minutes(20),
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _keep_open = tx;
            axum::serve(listener, router(rx, params, fallback))
                .await
                .unwrap();
        });
        format!("http://{addr}/v1/rain")
    }

    fn grid(age_min: i64) -> Grid {
        Grid {
            run_at: Utc::now() - Duration::minutes(age_min),
            lat0: 48.86,
            lon0: 2.35,
            ni: 1,
            nj: 1,
            steps: vec![vec![0.0]; 39],
        }
    }

    #[tokio::test]
    async fn donnee_fraiche() {
        let url = serve(Some(grid(12))).await;
        let res = reqwest::get(format!("{url}?lat=48.86&lon=2.35"))
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let cache = res.headers()["cache-control"].to_str().unwrap().to_string();
        assert!(cache.starts_with("private, max-age="), "{cache}");
        let body: Body = res.json().await.unwrap();
        assert_eq!(body.state, "dry");
        assert_eq!(body.source.as_deref(), Some("piaf"));
        assert_eq!(body.series.len(), 39);
        assert_eq!(body.series[0].mm_h, 0.0);
    }

    #[test]
    fn cache_jamais_au_dela_de_la_fraicheur() {
        let g = grid(0);
        let now = g.run_at;
        let params = Params {
            rain_threshold_mm_h: 0.1,
            min_wet_steps: 2,
            max_data_age: Duration::minutes(1),
        };
        let rain = forecast::rain_at(Some(&g), 48.86, 2.35, now, &params);
        let cache = cache_control(&rain, now, params.max_data_age);
        assert_eq!(cache, "private, max-age=60");
    }

    #[tokio::test]
    async fn donnee_perimee_sans_cache() {
        let url = serve(Some(grid(30))).await;
        let res = reqwest::get(format!("{url}?lat=48.86&lon=2.35"))
            .await
            .unwrap();
        assert_eq!(res.headers()["cache-control"], "no-store");
        let body: Body = res.json().await.unwrap();
        assert_eq!(body.state, "unknown");
        assert!(body.series.is_empty());
    }

    #[tokio::test]
    async fn sans_run_inconnu() {
        let url = serve(None).await;
        let res = reqwest::get(format!("{url}?lat=48.86&lon=2.35"))
            .await
            .unwrap();
        let body: Body = res.json().await.unwrap();
        assert_eq!(body.state, "unknown");
    }

    #[tokio::test]
    async fn piaf_perime_passe_au_repli() {
        let (fallback, _, _) = fake(vec![0.0; 15]).await;
        let url = serve_with(Some(grid(30)), fallback).await;
        let res = reqwest::get(format!("{url}?lat=48.86&lon=2.35"))
            .await
            .unwrap();
        let cache = res.headers()["cache-control"].to_str().unwrap().to_string();
        assert!(cache.starts_with("private, max-age="), "{cache}");
        let body: Body = res.json().await.unwrap();
        assert_eq!(body.state, "dry");
        assert_eq!(body.source.as_deref(), Some("open_meteo"));
        assert_eq!(body.step_minutes, Some(15));
    }

    #[tokio::test]
    async fn hors_zone_passe_au_repli() {
        let (fallback, _, _) = fake(vec![0.0; 15]).await;
        let url = serve_with(Some(grid(12)), fallback).await;
        let res = reqwest::get(format!("{url}?lat=40&lon=-74")).await.unwrap();
        let body: Body = res.json().await.unwrap();
        assert_eq!(body.source.as_deref(), Some("open_meteo"));
    }

    #[tokio::test]
    async fn piaf_frais_ne_touche_pas_au_repli() {
        let (fallback, calls, _) = fake(vec![0.0; 15]).await;
        let url = serve_with(Some(grid(12)), fallback).await;
        reqwest::get(format!("{url}?lat=48.86&lon=2.35"))
            .await
            .unwrap();
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn coordonnees_invalides() {
        let url = serve(None).await;
        for q in [
            "lat=91&lon=2",
            "lat=48&lon=181",
            "lat=NaN&lon=2",
            "lat=48",
            "lat=a&lon=2",
        ] {
            let res = reqwest::get(format!("{url}?{q}")).await.unwrap();
            assert_eq!(res.status(), 400, "{q}");
        }
    }
}
