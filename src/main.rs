use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{SystemTime},
};

use axum::{extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}}, http::StatusCode, routing::get, Json, Router, response::{IntoResponse, Html}};


use sysinfo::{CpuExt, System, SystemExt};
use tracing::Level;
use tokio::sync::broadcast;


type Snapshot = Vec<f32>;


#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<Snapshot>
}


#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Snapshot>(1);
    let app_state = AppState {
        tx: tx.clone(),
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default global subscriber failed");

    let router = Router::new()
        .route("/", get(root_get))
        .route("/api/timestamp", get(api_timestamp_get))
        .route("/api/cpus", get(api_cpus_get))
        .route("/wsapi/cpus", get(wsapi_cpus_get))
        .with_state(app_state.clone());


    // Update CPU in the background.
    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu();
            let v: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

            // Send can fail, but only if there are no receivers.
            let _ = app_state.tx.send(v);

            std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("Listening on {}", addr);

    let server = axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await;

    // Why not `unwrap()`? https://github.com/rust-lang/rust-analyzer/issues/14264
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
        .unwrap()
        .as_millis()
        .to_string()
}

async fn api_cpus_get(State(state): State<AppState>) -> (StatusCode, Json<Vec<f32>>) {
    let mut rx = state.tx.subscribe();
    let msg = rx.recv().await.unwrap();

    (StatusCode::OK, Json(msg))
}

#[axum::debug_handler]
async fn wsapi_cpus_get(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|sock| async {
        realtime_cpus_stream(sock, state).await
    })
}

async fn realtime_cpus_stream(mut ws: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    while let Ok(msg) = rx.recv().await {
        let payload = serde_json::to_string(&msg).unwrap();
        ws.send(Message::Text(payload)).await.unwrap();
    }
}

async fn root_get() -> impl IntoResponse {
    let markup = tokio::fs::read_to_string("src/index.html").await.unwrap();

    Html(markup)
}