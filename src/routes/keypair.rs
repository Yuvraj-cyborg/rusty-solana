use axum::{
    routing::post,
    Router,
    Json,
};
use solana_sdk::signature::{Keypair, Signer};
use bs58;
use crate::models::response::{json_success, json_error};
use serde::Serialize;

#[derive(Serialize)]
struct KeypairResponse {
    pubkey: String,
    secret: String,
}

pub fn keypair_routes() -> Router {
    Router::new().route("/keypair", post(generate_keypair))
}

async fn generate_keypair() -> Json<impl serde::Serialize> {
    let keypair = Keypair::new();

    let pubkey = keypair.pubkey().to_string();
    let secret = bs58::encode(keypair.to_bytes()).into_string();

    Json(KeypairResponse {
        pubkey,
        secret,
    })
}
