mod api;
mod forecast;
mod ingest;

use std::{env, str::FromStr};

use chrono::Duration;
use tokio::{net::TcpListener, sync::watch};

use crate::{forecast::Params, ingest::Piaf};

/// Variable optionnelle. Une valeur illisible arrête le démarrage plutôt que d'être ignorée.
fn env_or<T: FromStr>(name: &str, default: T) -> T {
    match env::var(name) {
        Ok(v) => v
            .parse()
            .unwrap_or_else(|_| panic!("{name} : valeur illisible")),
        Err(_) => default,
    }
}

#[tokio::main]
async fn main() {
    let app_id = env::var("METEOFRANCE_APP_ID").expect("METEOFRANCE_APP_ID manquant");
    // Sans valeur par défaut : un oubli ne doit pas lancer une ingestion France entière (D9).
    let bbox = env::var("PIAF_BBOX").expect("PIAF_BBOX manquant, voir .env.example");
    let subset = ingest::parse_bbox(&bbox).expect("PIAF_BBOX : lon_min,lon_max,lat_min,lat_max");
    let max_runs = env::var("PIAF_MAX_RUNS")
        .ok()
        .map(|v| v.parse().expect("PIAF_MAX_RUNS : entier attendu"));
    let params = Params {
        rain_threshold_mm_h: env_or("RAIN_THRESHOLD_MM_H", 0.1),
        min_wet_steps: env_or("MIN_WET_STEPS", 2),
        max_data_age: Duration::minutes(env_or("MAX_DATA_AGE_MIN", 20)),
    };
    let addr = env_or("BIND_ADDR", "127.0.0.1:8080".to_string());

    let (tx, rx) = watch::channel(None);
    tokio::spawn(ingest::run(Piaf::new(app_id, subset), tx, max_runs));

    let listener = TcpListener::bind(&addr).await.expect("port indisponible");
    eprintln!("api : écoute sur {addr}");
    axum::serve(listener, api::router(rx, params))
        .await
        .expect("serveur HTTP arrêté");
}
