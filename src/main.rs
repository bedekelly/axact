use std::net::SocketAddr;

use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = Router::new()
        .route("/", get(root_get));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap()
}

async fn root_get() -> &'static str {
    "hello, world!"
}