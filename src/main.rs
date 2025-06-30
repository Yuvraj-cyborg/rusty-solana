mod routes;
mod models;
mod utils;

use axum::{routing::get, Router};
use routes::keypair::keypair_routes;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .merge(keypair_routes()); // add more later

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Welcome to rusty-proc!"
}
