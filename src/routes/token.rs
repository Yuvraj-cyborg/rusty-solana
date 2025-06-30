use axum::{
    Router,
    routing::post,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use spl_token::instruction::{initialize_mint, mint_to};
use crate::models::response::{ApiResponse};
use std::str::FromStr;
use base64::{engine::general_purpose, Engine as _};

#[derive(Deserialize)]
struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    mint_authority: Option<String>,
    mint: Option<String>,
    decimals: Option<u8>,
}

#[derive(Serialize)]
struct AccountMetaResponse {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Serialize)]
struct TokenInstructionResponse {
    program_id: String,
    accounts: Vec<AccountMetaResponse>,
    instruction_data: String,
}

pub fn token_routes() -> Router {
    Router::new()
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
}

async fn create_token(
    Json(body): Json<CreateTokenRequest>,
) -> impl IntoResponse {
    // Validate required fields
    let mint_authority_str = match body.mint_authority {
        Some(ref s) if !s.is_empty() => s,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };
    
    let mint_str = match body.mint {
        Some(ref s) if !s.is_empty() => s,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };
    
    let decimals = match body.decimals {
        Some(d) => d,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };

    let mint_authority = match Pubkey::from_str(mint_authority_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid mint authority address".to_string()),
    };

    let mint = match Pubkey::from_str(mint_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid mint address".to_string()),
    };

    let token_program_id = spl_token::id();

    let ix = match initialize_mint(
        &token_program_id,
        &mint,
        &mint_authority,
        None,
        decimals,
    ) {
        Ok(instruction) => instruction,
        Err(_) => return ApiResponse::Error("Failed to create mint instruction".to_string()),
    };

    let accounts: Vec<AccountMetaResponse> = ix.accounts.into_iter()
        .map(|a| AccountMetaResponse {
            pubkey: a.pubkey.to_string(),
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        })
        .collect();

    let encoded = general_purpose::STANDARD.encode(ix.data);

    ApiResponse::Success(TokenInstructionResponse {
        program_id: ix.program_id.to_string(),
        accounts,
        instruction_data: encoded,
    })
}

#[derive(Deserialize)]
struct MintTokenRequest {
    mint: Option<String>,
    destination: Option<String>,
    authority: Option<String>,
    amount: Option<u64>,
}

async fn mint_token(
    Json(body): Json<MintTokenRequest>,
) -> impl IntoResponse {
    // Validate required fields
    let mint_str = match body.mint {
        Some(ref s) if !s.is_empty() => s,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };
    
    let destination_str = match body.destination {
        Some(ref s) if !s.is_empty() => s,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };
    
    let authority_str = match body.authority {
        Some(ref s) if !s.is_empty() => s,
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };
    
    let amount = match body.amount {
        Some(a) if a > 0 => a,
        Some(0) => return ApiResponse::Error("Amount must be greater than 0".to_string()),
        _ => return ApiResponse::Error("Missing required fields".to_string()),
    };

    let mint = match Pubkey::from_str(mint_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid mint address".to_string()),
    };

    let destination = match Pubkey::from_str(destination_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid destination address".to_string()),
    };

    let authority = match Pubkey::from_str(authority_str) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid authority address".to_string()),
    };

    // Associated token address for the destination and mint
    let destination_ata = spl_associated_token_account::get_associated_token_address(&destination, &mint);

    let ix = match mint_to(
        &spl_token::id(),
        &mint,
        &destination_ata,
        &authority,
        &[],
        amount,
    ) {
        Ok(instruction) => instruction,
        Err(_) => return ApiResponse::Error("Failed to create mint instruction".to_string()),
    };

    let accounts: Vec<AccountMetaResponse> = ix.accounts.into_iter()
        .map(|a| AccountMetaResponse {
            pubkey: a.pubkey.to_string(),
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        })
        .collect();

    let encoded = general_purpose::STANDARD.encode(ix.data);

    ApiResponse::Success(TokenInstructionResponse {
        program_id: ix.program_id.to_string(),
        accounts,
        instruction_data: encoded,
    })
}
