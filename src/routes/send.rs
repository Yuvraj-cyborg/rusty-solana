use std::str::FromStr;
use axum::{Router, Json, routing::post, response::IntoResponse};
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    system_instruction,
};
use spl_token::instruction::transfer;
use base64::{engine::general_purpose, Engine as _};
use crate::models::response::ApiResponse;

pub fn send_routes() -> Router {
    Router::new()
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token))
}

#[derive(Deserialize)]
struct SendSolRequest {
    from: Option<String>,
    to: Option<String>,
    lamports: Option<u64>,
}

#[derive(Serialize)]
struct SendSolResponse {
    program_id: String,
    accounts: Vec<String>,
    instruction_data: String,
}

#[derive(Deserialize)]
struct SendTokenRequest {
    destination: Option<String>,
    mint: Option<String>,
    owner: Option<String>,
    amount: Option<u64>,
}

#[derive(Serialize)]
struct AccountInfo {
    pubkey: String,
    #[serde(rename = "isSigner")]
    is_signer: bool,
    #[serde(rename = "isWritable")]
    is_writable: bool,
}

#[derive(Serialize)]
struct SendTokenResponse {
    program_id: String,
    accounts: Vec<AccountInfo>,
    instruction_data: String,
}

async fn send_sol(
    Json(body): Json<SendSolRequest>,
) -> impl IntoResponse {
    let from_str = match body.from {
        Some(ref s) if !s.is_empty() => s,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };
    
    let to_str = match body.to {
        Some(ref s) if !s.is_empty() => s,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };
    
    let lamports = match body.lamports {
        Some(l) if l > 0 => l,
        Some(0) => return ApiResponse::Error("Amount must be greater than 0".to_string()),
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };

    let from_pubkey = match Pubkey::from_str(from_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid 'from' address".to_string()),
    };

    let to_pubkey = match Pubkey::from_str(to_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid 'to' address".to_string()),
    };

    let instruction = system_instruction::transfer(&from_pubkey, &to_pubkey, lamports);

    let accounts = vec![
        instruction.accounts[0].pubkey.to_string(),
        instruction.accounts[1].pubkey.to_string(),
    ];

    let instruction_data = general_purpose::STANDARD.encode(&instruction.data);

    ApiResponse::Success(SendSolResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data,
    })
}

async fn send_token(
    Json(body): Json<SendTokenRequest>,
) -> impl IntoResponse {
    let destination_str = match body.destination {
        Some(ref s) if !s.is_empty() => s,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };
    
    let mint_str = match body.mint {
        Some(ref s) if !s.is_empty() => s,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };
    
    let owner_str = match body.owner {
        Some(ref s) if !s.is_empty() => s,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };
    
    let amount = match body.amount {
        Some(a) if a > 0 => a,
        Some(0) => return ApiResponse::Error("Amount must be greater than 0".to_string()),
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };

    let destination = match Pubkey::from_str(destination_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid destination address".to_string()),
    };

    let mint = match Pubkey::from_str(mint_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid mint address".to_string()),
    };

    let owner = match Pubkey::from_str(owner_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid owner address".to_string()),
    };

    let source_ata = spl_associated_token_account::get_associated_token_address(&owner, &mint);
    let dest_ata = spl_associated_token_account::get_associated_token_address(&destination, &mint);

    let instruction = match transfer(
        &spl_token::id(),
        &source_ata,
        &dest_ata,
        &owner,
        &[],
        amount,
    ) {
        Ok(ix) => ix,
        Err(_) => return ApiResponse::Error("Failed to create transfer instruction".to_string()),
    };

    let accounts: Vec<AccountInfo> = instruction.accounts.into_iter()
        .map(|a| AccountInfo {
            pubkey: a.pubkey.to_string(),
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        })
        .collect();

    let instruction_data = general_purpose::STANDARD.encode(&instruction.data);

    ApiResponse::Success(SendTokenResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data,
    })
}
