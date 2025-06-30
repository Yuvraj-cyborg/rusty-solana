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
    mintAuthority: String,
    mint: String,
    decimals: u8,
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
    let mint_authority = match Pubkey::from_str(&body.mintAuthority) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid mint authority address".to_string()),
    };

    let mint = match Pubkey::from_str(&body.mint) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid mint address".to_string()),
    };

    let token_program_id = spl_token::id();

    let ix = match initialize_mint(
        &token_program_id,
        &mint,
        &mint_authority,
        None,
        body.decimals,
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
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

async fn mint_token(
    Json(body): Json<MintTokenRequest>,
) -> impl IntoResponse {
    let mint = match Pubkey::from_str(&body.mint) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid mint address".to_string()),
    };

    let destination = match Pubkey::from_str(&body.destination) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid destination address".to_string()),
    };

    let authority = match Pubkey::from_str(&body.authority) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::Error("Invalid authority address".to_string()),
    };

    if body.amount == 0 {
        return ApiResponse::Error("Amount must be greater than 0".to_string());
    }

    // Associated token address for the destination and mint
    let destination_ata = spl_associated_token_account::get_associated_token_address(&destination, &mint);

    let ix = match mint_to(
        &spl_token::id(),
        &mint,
        &destination_ata,
        &authority,
        &[],
        body.amount,
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
