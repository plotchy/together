use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use alloy::primitives::Address;
use crate::{
    db::casts::CastRepository,
    utils::{Config, eip712::{Eip712Signer, parse_token_ids}, token_conversion::token_id_to_cast_hash},
    constants::*,
};

#[derive(Debug, Deserialize)]
pub struct TogetherRequest {
    pub wallet_address: String,
    pub token_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WrapResponse {
    pub signature: String,
    pub nonce: String,
    pub deadline: u64,
    pub token_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WrapError {
    pub error: String,
    pub invalid_tokens: Option<Vec<String>>,
}

pub async fn wrap_cast(
    State((pool, config)): State<(PgPool, Config)>,
    Json(req): Json<WrapRequest>,
) -> Result<Json<WrapResponse>, (StatusCode, Json<WrapError>)> {
    // Validate wallet address format
    let wallet_address: Address = req.wallet_address.parse()
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(WrapError {
                error: "Invalid wallet address format".to_string(),
                invalid_tokens: None,
            }),
        ))?;

    // Parse token IDs
    let token_ids_u256 = parse_token_ids(&req.token_ids)
        .map_err(|e| (
            StatusCode::BAD_REQUEST,
            Json(WrapError {
                error: format!("Invalid token IDs: {}", e),
                invalid_tokens: None,
            }),
        ))?;

    // Validate NFTs against database - check they're valid for wrapping
    let cast_repo = CastRepository::new(pool);
    let mut invalid_tokens = Vec::new();
    
    for token_id in &req.token_ids {
        // Convert token ID to cast hash for database lookup
        let cast_hash = match token_id_to_cast_hash(token_id) {
            Ok(hash) => hash,
            Err(e) => {
                invalid_tokens.push(format!("{} (invalid token ID: {})", token_id, e));
                continue;
            }
        };
        
        // Check if cast exists and is valid for wrapping
        if let Ok(Some(cast)) = cast_repo.get_by_hash(&cast_hash).await {
            // Verify it's from DWR
            if cast.author_fid != DWR_FARCASTER_FID {
                invalid_tokens.push(format!("{} (not authored by DWR)", token_id));
                continue;
            }
            
            // Verify it's not a reply
            if cast.parent_hash.is_some() {
                invalid_tokens.push(format!("{} (is a reply)", token_id));
                continue;
            }
            
            // Verify it's marked as included
            if !cast.include {
                // technically since just author and not reply is what "include" is for, we shouldn't be able to reach this.
                tracing::error!("Found error on cast {} (is from DWR and is not a reply, but is not included)", token_id);
                invalid_tokens.push(format!("{} (not a valid DWRcast)", token_id));
                continue;
            }
        } else {
            invalid_tokens.push(format!("{} (cast not found)", token_id));
        }
    }

    if !invalid_tokens.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(WrapError {
                error: format!("Some of the selected NFTs are not valid for wrapping: {}", invalid_tokens.join(", ")),
                invalid_tokens: Some(invalid_tokens),
            }),
        ));
    }

    // Generate EIP712 signature
    let signer = Eip712Signer::new(&config.private_key_signer, BASE_MAINNET_CHAIN_ID)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(WrapError {
                error: format!("Failed to initialize signer: {}", e),
                invalid_tokens: None,
            }),
        ))?;

    let contract_address: Address = config.dwrcasts_contract_address.parse()
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(WrapError {
                error: "Invalid contract address in config".to_string(),
                invalid_tokens: None,
            }),
        ))?;

    let nonce = Eip712Signer::generate_nonce();
    let deadline = Eip712Signer::generate_deadline_10_minutes();

    let signature_data = signer.sign_wrap_permit(
        contract_address,
        wallet_address,
        token_ids_u256,
        nonce,
        deadline,
    ).await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(WrapError {
            error: format!("Failed to generate signature: {}", e),
            invalid_tokens: None,
        }),
    ))?;

    Ok(Json(WrapResponse {
        signature: signature_data.signature,
        nonce: nonce.to_string(),
        deadline,
        token_ids: req.token_ids,
    }))
}
