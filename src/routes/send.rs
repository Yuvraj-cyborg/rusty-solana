use axum::{Router, Json, routing::post, response::IntoResponse};
use serde::{Deserialize, Serialize};
use solana_sdk::system_instruction;
use spl_token::instruction::transfer;
use base64::{engine::general_purpose, Engine as _};
use crate::models::response::ApiResponse;
use crate::utils::{validate_required_string, validate_pubkey, validate_amount};

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
    let from_str = match validate_required_string(&body.from, "from") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let to_str = match validate_required_string(&body.to, "to") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let lamports = match validate_amount(body.lamports, "lamports") {
        Ok(a) => a,
        Err(e) => return ApiResponse::Error(e),
    };

    let from_pubkey = match validate_pubkey(&from_str, "from") {
        Ok(pk) => pk,
        Err(e) => return ApiResponse::Error(e),
    };

    let to_pubkey = match validate_pubkey(&to_str, "to") {
        Ok(pk) => pk,
        Err(e) => return ApiResponse::Error(e),
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
    let destination_str = match validate_required_string(&body.destination, "destination") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let mint_str = match validate_required_string(&body.mint, "mint") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let owner_str = match validate_required_string(&body.owner, "owner") {
        Ok(s) => s,
        Err(e) => return ApiResponse::Error(e),
    };
    
    let amount = match validate_amount(body.amount, "amount") {
        Ok(a) => a,
        Err(e) => return ApiResponse::Error(e),
    };

    let destination = match validate_pubkey(&destination_str, "destination") {
        Ok(pk) => pk,
        Err(e) => return ApiResponse::Error(e),
    };

    let mint = match validate_pubkey(&mint_str, "mint") {
        Ok(pk) => pk,
        Err(e) => return ApiResponse::Error(e),
    };

    let owner = match validate_pubkey(&owner_str, "owner") {
        Ok(pk) => pk,
        Err(e) => return ApiResponse::Error(e),
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

    let accounts: Vec<AccountInfo> = vec![
        AccountInfo { // Source ATA
            pubkey: instruction.accounts[0].pubkey.to_string(),
            is_signer: false,
        },
        AccountInfo { // Destination ATA
            pubkey: instruction.accounts[1].pubkey.to_string(),
            is_signer: false,
        },
        AccountInfo { // Owner
            pubkey: instruction.accounts[2].pubkey.to_string(),
            is_signer: true,
        },
    ];

    let instruction_data = general_purpose::STANDARD.encode(&instruction.data);

    ApiResponse::Success(SendTokenResponse {
        program_id: instruction.program_id.to_string(),
        accounts,
        instruction_data,
    })
}
