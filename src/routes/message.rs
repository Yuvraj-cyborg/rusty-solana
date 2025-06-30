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
use crate::utils::validate_required_string;

pub fn message_routes() -> Router {
    Router::new()
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
}

#[derive(Deserialize)]
struct SignMessageRequest {
    message: Option<String>,
    secret: Option<String>,
}

#[derive(Serialize)]
struct SignMessageResponse {
    signature: String,
    public_key: String,
    message: String,
}

#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: Option<String>,
    signature: Option<String>,
    pubkey: Option<String>,
}

#[derive(Serialize)]
struct VerifyMessageResponse {
    valid: bool,
    message: String,
    pubkey: String,
}

// Helper function to validate and decode base58 secret key
fn validate_secret_key(secret: &str) -> Result<Keypair, String> {
    let secret_bytes = bs58::decode(secret)
        .into_vec()
        .map_err(|_| "Invalid base58 secret key".to_string())?;

    if secret_bytes.len() != 64 {
        return Err("Invalid secret key length".to_string());
    }

    Keypair::from_bytes(&secret_bytes)
        .map_err(|_| "Invalid keypair from secret key".to_string())
}

async fn sign_message(
    Json(body): Json<SignMessageRequest>,
) -> impl IntoResponse {
    let message = match validate_required_string(&body.message, "message") {
        Ok(m) => m,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let secret = match validate_required_string(&body.secret, "secret") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };

    let keypair = match validate_secret_key(&secret) {
        Ok(kp) => kp,
        Err(e) => return ApiResponse::Error(e),
    };

    let message_bytes = message.as_bytes();
    let signature = keypair.sign_message(message_bytes);
    
    let signature_base64 = general_purpose::STANDARD.encode(signature.as_ref());
    let public_key = keypair.pubkey().to_string();

    ApiResponse::Success(SignMessageResponse {
        signature: signature_base64,
        public_key,
        message,
    })
}

async fn verify_message(
    Json(body): Json<VerifyMessageRequest>,
) -> impl IntoResponse {
    let message = match validate_required_string(&body.message, "message") {
        Ok(m) => m,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let signature_str = match validate_required_string(&body.signature, "signature") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let pubkey_str = match validate_required_string(&body.pubkey, "pubkey") {
        Ok(p) => p,
        Err(e) => return ApiResponse::Error(e),
    };

    let signature_bytes = match general_purpose::STANDARD.decode(&signature_str) {
        Ok(bytes) => bytes,
        Err(_) => return ApiResponse::Error("Invalid base64 signature".to_string()),
    };

    let signature_array: [u8; 64] = match signature_bytes.as_slice().try_into() {
        Ok(arr) => arr,
        Err(_) => return ApiResponse::Error("Invalid signature length".to_string()),
    };

    let signature = Signature::from(signature_array);

    let pubkey = match Pubkey::from_str(&pubkey_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid base58 public key".to_string()),
    };

    let message_bytes = message.as_bytes();
    let is_valid = signature.verify(&pubkey.to_bytes(), message_bytes);

    ApiResponse::Success(VerifyMessageResponse {
        valid: is_valid,
        message,
        pubkey: pubkey_str,
    })
}
