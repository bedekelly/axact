use std::{net::SocketAddr, time::SystemTime, sync::{Arc, Mutex}};

use axum::{routing::get, Router, extract::State};
use sysinfo::{CpuExt, System, SystemExt};
use tracing::Level;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder().with_max_level(Level::DEBUG).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default global subscriber failed");

    let router = Router::new()
        .route("/timestamp", get(root_get))
        .route("/system", get(system_get))
        .with_state(AppState { sys: Arc::new(Mutex::new(System::new())) });

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

async fn root_get() -> String {
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

async fn system_get(State(state): State<AppState>) -> String {
    let mut sys = state.sys.lock().unwrap();

    let mut info = String::new();
    sys.refresh_cpu();

    for cpu in sys.cpus() {
        info.push_str(&cpu.cpu_usage().to_string());
        info.push('\n');
    }

    info
}
