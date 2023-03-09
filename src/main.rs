use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};

use serde::{Serialize};
use sysinfo::{CpuExt, System, SystemExt};
use tracing::Level;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default global subscriber failed");

    let router = Router::new()
        // .route("/", get(root_get))
        .route("/api/timestamp", get(api_timestamp_get))
        .route("/api/cpus", get(api_cpus_get))
        .with_state(AppState {
            sys: Arc::new(Mutex::new(System::new())),
        });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("Listening on {}", addr);

    let server = axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await;

    match server {
        Ok(r) => r,
        Err(_) => {
            panic!("Unwrap server failed!")
        }
    }

    ()
}

async fn api_timestamp_get() -> String {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Test thing")
        .as_millis()
        .to_string()
}

#[derive(Clone)]
struct AppState {
    sys: Arc<Mutex<System>>,
}

#[derive(Serialize)]
struct InfoResponse {
    cpu_usage: Vec<f32>,
}

#[axum::debug_handler]
async fn api_cpus_get(State(state): State<AppState>) -> (StatusCode, Json<Vec<f32>>) {
    let mut sys = state.sys.lock().unwrap();

    sys.refresh_cpu();

    let response: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    (StatusCode::OK, Json(response))
}
