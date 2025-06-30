use axum::{
    Router,
    routing::post,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use spl_token::instruction::{initialize_mint, mint_to};
use crate::models::response::{ApiResponse};
use base64::{engine::general_purpose, Engine as _};
use crate::utils::{validate_required_string, validate_pubkey, validate_amount, validate_required_numeric};

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

#[derive(Deserialize)]
struct MintTokenRequest {
    mint: Option<String>,
    destination: Option<String>,
    authority: Option<String>,
    amount: Option<u64>,
}

pub fn token_routes() -> Router {
    Router::new()
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
}

// Helper function to create instruction response
fn create_instruction_response(instruction: solana_sdk::instruction::Instruction) -> TokenInstructionResponse {
    let accounts: Vec<AccountMetaResponse> = instruction.accounts.into_iter()
        .map(|a| AccountMetaResponse {
            pubkey: a.pubkey.to_string(),
            is_signer: a.is_signer,
            is_writable: a.is_writable,
        })
        .collect();

    let encoded = general_purpose::STANDARD.encode(instruction.data);

    TokenInstructionResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data: encoded,
    }
}

async fn create_token(
    Json(body): Json<CreateTokenRequest>,
) -> impl IntoResponse {
    let mint_authority_str = match validate_required_string(&body.mint_authority, "mintAuthority") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let mint_str = match validate_required_string(&body.mint, "mint") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let decimals = match validate_required_numeric(&body.decimals, "decimals") {
        Ok(d) => d,
        Err(e) => return ApiResponse::Error(e),
    };

    let mint_authority = match validate_pubkey(&mint_authority_str, "mintAuthority") {
        Ok(pk) => pk,
        Err(e) => return ApiResponse::Error(e),
    };

    let mint = match validate_pubkey(&mint_str, "mint") {
        Ok(pk) => pk,
        Err(e) => return ApiResponse::Error(e),
    };

    let ix = match initialize_mint(
        &spl_token::id(),
        &mint,
        &mint_authority,
        None,
        decimals,
    ) {
        Ok(instruction) => instruction,
        Err(_) => return ApiResponse::Error("Failed to create mint instruction".to_string()),
    };

    ApiResponse::Success(create_instruction_response(ix))
}

async fn mint_token(
    Json(body): Json<MintTokenRequest>,
) -> impl IntoResponse {
    let mint_str = match validate_required_string(&body.mint, "mint") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let destination_str = match validate_required_string(&body.destination, "destination") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let authority_str = match validate_required_string(&body.authority, "authority") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let amount = match validate_amount(body.amount, "amount") {
        Ok(a) => a,
        Err(e) => return ApiResponse::Error(e),
    };

    let mint = match validate_pubkey(&mint_str, "mint") {
        Ok(pk) => pk,
        Err(e) => return ApiResponse::Error(e),
    };

    let destination = match validate_pubkey(&destination_str, "destination") {
        Ok(pk) => pk,
        Err(e) => return ApiResponse::Error(e),
    };

    let authority = match validate_pubkey(&authority_str, "authority") {
        Ok(pk) => pk,
        Err(e) => return ApiResponse::Error(e),
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

    ApiResponse::Success(create_instruction_response(ix))
}
