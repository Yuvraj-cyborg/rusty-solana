use axum::{
    Json,
    response::IntoResponse,
    Router,
    routing::post,
};
use base64::{engine::general_purpose, Engine as _};
use bs58;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    signature::{Keypair, Signature, Signer},
    pubkey::Pubkey,
};
use std::convert::TryInto;
use std::str::FromStr;
use crate::models::response::ApiResponse;

pub fn message_routes() -> Router {
    Router::new()
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
}

#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,
    secret: String,
}

#[derive(Serialize)]
struct SignMessageResponse {
    signature: String,
    public_key: String,
    message: String,
}

#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String,
    pubkey: String,
}

#[derive(Serialize)]
struct VerifyMessageResponse {
    valid: bool,
    message: String,
    pubkey: String,
}

async fn sign_message(
    Json(body): Json<SignMessageRequest>,
) -> impl IntoResponse {
    if body.message.is_empty() || body.secret.is_empty() {
        return ApiResponse::Error("Missing required fields".to_string());
    }

    let secret_bytes = match bs58::decode(&body.secret).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => return ApiResponse::Error("Invalid base58 secret key".to_string()),
    };

    if secret_bytes.len() != 64 {
        return ApiResponse::Error("Invalid secret key length".to_string());
    }

    let keypair = match Keypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(_) => return ApiResponse::Error("Invalid keypair from secret key".to_string()),
    };

    let message_bytes = body.message.as_bytes();
    let signature = keypair.sign_message(message_bytes);
    
    let signature_base64 = general_purpose::STANDARD.encode(signature.as_ref());
    let public_key = keypair.pubkey().to_string();

    ApiResponse::Success(SignMessageResponse {
        signature: signature_base64,
        public_key,
        message: body.message,
    })
}

async fn verify_message(
    Json(body): Json<VerifyMessageRequest>,
) -> impl IntoResponse {
    if body.message.is_empty() || body.signature.is_empty() || body.pubkey.is_empty() {
        return ApiResponse::Error("Missing required fields".to_string());
    }

    let signature_bytes = match general_purpose::STANDARD.decode(&body.signature) {
        Ok(bytes) => bytes,
        Err(_) => return ApiResponse::Error("Invalid base64 signature".to_string()),
    };

    let signature_array: [u8; 64] = match signature_bytes.as_slice().try_into() {
        Ok(arr) => arr,
        Err(_) => return ApiResponse::Error("Invalid signature length".to_string()),
    };

    let signature = Signature::from(signature_array);

    let pubkey = match Pubkey::from_str(&body.pubkey) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid base58 public key".to_string()),
    };

    let message_bytes = body.message.as_bytes();
    let is_valid = signature.verify(&pubkey.to_bytes(), message_bytes);

    ApiResponse::Success(VerifyMessageResponse {
        valid: is_valid,
        message: body.message,
        pubkey: body.pubkey,
    })
}
