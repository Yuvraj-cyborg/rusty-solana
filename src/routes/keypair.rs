use axum::{
    routing::post,
    Router,
    Json,
};
use solana_sdk::signature::{Keypair, Signer};
use bs58;
use serde::Serialize;

#[derive(Serialize)]
struct KeypairResponse {
    pubkey: String,
    secret: String,
}

pub fn keypair_routes() -> Router {
    Router::new().route("/keypair", post(generate_keypair))
}

async fn generate_keypair() -> Json<KeypairResponse> {
    let keypair = Keypair::new();

    let secret_key_bytes = keypair.to_bytes(); // 64 bytes [secret || pubkey]
    let secret_base58 = bs58::encode(secret_key_bytes).into_string();
    let pubkey = keypair.pubkey().to_string();

    Json(KeypairResponse {
        pubkey,
        secret: secret_base58,
    })
}
