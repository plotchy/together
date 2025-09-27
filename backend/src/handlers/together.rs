use axum::{extract::{State, Path, Query}, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use alloy::primitives::Address;
use crate::{
    utils::{Config, eip712::Eip712Signer},
    constants::*,
    models::attestations::{UserProfile, TogetherAttestation},
    db::attestations,
};

// Request to create an attestation signature
#[derive(Debug, Deserialize)]
pub struct AttestTogetherRequest {
    pub my_address: String,
    pub partner_address: String, 
    pub timestamp: i64,
    pub password: String,
    pub my_username: Option<String>,
    pub partner_username: Option<String>,
    pub my_profile_picture_url: Option<String>,
    pub partner_profile_picture_url: Option<String>,
}

// Response with signature for on-chain attestation
#[derive(Debug, Serialize)]
pub struct AttestTogetherResponse {
    pub signature: String,
    pub nonce: String,
    pub deadline: u64,
}

// Request to submit an attestation (from blockchain watcher or direct submission)
#[derive(Debug, Deserialize)]
pub struct SubmitAttestationRequest {
    pub address_1: String,
    pub address_2: String,
    pub timestamp: i64,
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
    pub username_1: Option<String>,
    pub username_2: Option<String>,
    pub profile_picture_url_1: Option<String>,
    pub profile_picture_url_2: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SubmitAttestationResponse {
    pub success: bool,
    pub attestation_id: Option<String>,
}

// Query parameters for profile endpoint
#[derive(Debug, Deserialize)]
pub struct ProfileQuery {
    #[serde(default)]
    pub limit: Option<i64>,
    pub username: Option<String>,
    pub profile_picture_url: Option<String>,
}

// Query parameters for checking if two addresses have been together
#[derive(Debug, Deserialize)]
pub struct CheckTogetherQuery {
    pub address_2: String,
}

#[derive(Debug, Serialize)]
pub struct TogetherError {
    pub error: String,
}

/// Get user profile with their together connections
pub async fn get_profile(
    State((pool, _config)): State<(PgPool, Config)>,
    Path(address): Path<String>,
    Query(params): Query<ProfileQuery>,
) -> Result<Json<UserProfile>, (StatusCode, Json<TogetherError>)> {
    // Validate address format
    let _address: Address = address.parse()
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(TogetherError {
                error: "Invalid wallet address format".to_string(),
            }),
        ))?;

    // Cache username if provided in query params (from frontend when user visits their own profile)
    if params.username.is_some() || params.profile_picture_url.is_some() {
        if let Err(e) = attestations::upsert_username_cache(
            &pool,
            &address,
            params.username.as_deref(),
            params.profile_picture_url.as_deref(),
        ).await {
            tracing::warn!("Failed to cache username for {}: {}", address, e);
        }
    }

    let profile = attestations::get_user_profile(&pool, &address, params.limit).await
        .map_err(|e| {
            tracing::error!("Failed to get user profile: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TogetherError {
                    error: "Failed to retrieve user profile".to_string(),
                }),
            )
        })?;

    Ok(Json(profile))
}

/// Check if two addresses have been together
pub async fn check_together(
    State((pool, _config)): State<(PgPool, Config)>,
    Path(address_1): Path<String>,
    Query(params): Query<CheckTogetherQuery>,
) -> Result<Json<Option<TogetherAttestation>>, (StatusCode, Json<TogetherError>)> {
    // Validate address formats
    let _addr1: Address = address_1.parse()
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(TogetherError {
                error: "Invalid wallet address format for address_1".to_string(),
            }),
        ))?;

    let _addr2: Address = params.address_2.parse()
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(TogetherError {
                error: "Invalid wallet address format for address_2".to_string(),
            }),
        ))?;

    let attestation = attestations::check_together(&pool, &address_1, &params.address_2).await
        .map_err(|e| {
            tracing::error!("Failed to check together status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TogetherError {
                    error: "Failed to check together status".to_string(),
                }),
            )
        })?;

    Ok(Json(attestation))
}

/// Generate a signature for attesting that two users were together
pub async fn attest_together(
    State((pool, config)): State<(PgPool, Config)>,
    Json(req): Json<AttestTogetherRequest>,
) -> Result<Json<AttestTogetherResponse>, (StatusCode, Json<TogetherError>)> {
    // Validate wallet addresses
    let my_address: Address = req.my_address.parse()
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(TogetherError {
                error: "Invalid my_address format".to_string(),
            }),
        ))?;

    let partner_address: Address = req.partner_address.parse()
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(TogetherError {
                error: "Invalid partner_address format".to_string(),
            }),
        ))?;

    // Check if they've already been together at this exact timestamp
    if let Some(_existing) = attestations::check_together(&pool, &req.my_address, &req.partner_address).await
        .map_err(|e| {
            tracing::error!("Failed to check existing attestation: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TogetherError {
                    error: "Failed to check existing attestation".to_string(),
                }),
            )
        })? {
        // They've already been together - could still allow but warn frontend
        tracing::info!("Addresses {} and {} have already been together", req.my_address, req.partner_address);
    }

    // Generate EIP712 signature for the together attestation
    let signer = Eip712Signer::new(&config.private_key_signer, WORLDCHAIN_MAINNET_CHAIN_ID)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TogetherError {
                error: format!("Failed to initialize signer: {}", e),
            }),
        ))?;

    let contract_address: Address = config.together_contract_address.parse()
        .map_err(|_| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TogetherError {
                error: "Invalid contract address in config".to_string(),
            }),
        ))?;

    let nonce = Eip712Signer::generate_nonce();
    let deadline = Eip712Signer::generate_deadline_10_minutes();

    // This would need to be implemented in the EIP712 signer
    let signature_data = signer.sign_together_attestation(
        contract_address,
        my_address,
        partner_address,
        req.timestamp,
        nonce,
        deadline,
    ).await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(TogetherError {
            error: format!("Failed to generate signature: {}", e),
        }),
    ))?;

    // Cache usernames if provided
    if req.my_username.is_some() || req.my_profile_picture_url.is_some() {
        if let Err(e) = attestations::upsert_username_cache(
            &pool,
            &req.my_address,
            req.my_username.as_deref(),
            req.my_profile_picture_url.as_deref(),
        ).await {
            tracing::warn!("Failed to cache username for {}: {}", req.my_address, e);
        }
    }

    if req.partner_username.is_some() || req.partner_profile_picture_url.is_some() {
        if let Err(e) = attestations::upsert_username_cache(
            &pool,
            &req.partner_address,
            req.partner_username.as_deref(),
            req.partner_profile_picture_url.as_deref(),
        ).await {
            tracing::warn!("Failed to cache username for {}: {}", req.partner_address, e);
        }
    }

    Ok(Json(AttestTogetherResponse {
        signature: signature_data.signature,
        nonce: nonce.to_string(),
        deadline,
    }))
}

/// Submit an attestation (typically called by blockchain watcher)
pub async fn submit_attestation(
    State((pool, _config)): State<(PgPool, Config)>,
    Json(req): Json<SubmitAttestationRequest>,
) -> Result<Json<SubmitAttestationResponse>, (StatusCode, Json<TogetherError>)> {
    // Validate addresses
    let _addr1: Address = req.address_1.parse()
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(TogetherError {
                error: "Invalid address_1 format".to_string(),
            }),
        ))?;

    let _addr2: Address = req.address_2.parse()
        .map_err(|_| (
            StatusCode::BAD_REQUEST,
            Json(TogetherError {
                error: "Invalid address_2 format".to_string(),
            }),
        ))?;

    let attestation = attestations::insert_attestation(
        &pool,
        &req.address_1,
        &req.address_2,
        req.timestamp,
        req.tx_hash.as_deref(),
        req.block_number,
    ).await
    .map_err(|e| {
        tracing::error!("Failed to insert attestation: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TogetherError {
                error: "Failed to insert attestation".to_string(),
            }),
        )
    })?;

    // Cache usernames if provided
    if req.username_1.is_some() || req.profile_picture_url_1.is_some() {
        if let Err(e) = attestations::upsert_username_cache(
            &pool,
            &req.address_1,
            req.username_1.as_deref(),
            req.profile_picture_url_1.as_deref(),
        ).await {
            tracing::warn!("Failed to cache username for {}: {}", req.address_1, e);
        }
    }

    if req.username_2.is_some() || req.profile_picture_url_2.is_some() {
        if let Err(e) = attestations::upsert_username_cache(
            &pool,
            &req.address_2,
            req.username_2.as_deref(),
            req.profile_picture_url_2.as_deref(),
        ).await {
            tracing::warn!("Failed to cache username for {}: {}", req.address_2, e);
        }
    }

    tracing::info!(
        "Successfully inserted attestation for {} and {} at timestamp {}",
        req.address_1,
        req.address_2,
        req.timestamp
    );

    Ok(Json(SubmitAttestationResponse {
        success: true,
        attestation_id: Some(attestation.id.to_string()),
    }))
}


// pub async fn wrap_cast(
//     State((pool, config)): State<(PgPool, Config)>,
//     Json(req): Json<WrapRequest>,
// ) -> Result<Json<WrapResponse>, (StatusCode, Json<WrapError>)> {
//     // Validate wallet address format
//     let wallet_address: Address = req.wallet_address.parse()
//         .map_err(|_| (
//             StatusCode::BAD_REQUEST,
//             Json(WrapError {
//                 error: "Invalid wallet address format".to_string(),
//                 invalid_tokens: None,
//             }),
//         ))?;

//     // Parse token IDs
//     let token_ids_u256 = parse_token_ids(&req.token_ids)
//         .map_err(|e| (
//             StatusCode::BAD_REQUEST,
//             Json(WrapError {
//                 error: format!("Invalid token IDs: {}", e),
//                 invalid_tokens: None,
//             }),
//         ))?;

//     // Validate NFTs against database - check they're valid for wrapping
//     let cast_repo = CastRepository::new(pool);
//     let mut invalid_tokens = Vec::new();
    
//     for token_id in &req.token_ids {
//         // Convert token ID to cast hash for database lookup
//         let cast_hash = match token_id_to_cast_hash(token_id) {
//             Ok(hash) => hash,
//             Err(e) => {
//                 invalid_tokens.push(format!("{} (invalid token ID: {})", token_id, e));
//                 continue;
//             }
//         };
        
//         // Check if cast exists and is valid for wrapping
//         if let Ok(Some(cast)) = cast_repo.get_by_hash(&cast_hash).await {
//             // Verify it's from DWR
//             if cast.author_fid != DWR_FARCASTER_FID {
//                 invalid_tokens.push(format!("{} (not authored by DWR)", token_id));
//                 continue;
//             }
            
//             // Verify it's not a reply
//             if cast.parent_hash.is_some() {
//                 invalid_tokens.push(format!("{} (is a reply)", token_id));
//                 continue;
//             }
            
//             // Verify it's marked as included
//             if !cast.include {
//                 // technically since just author and not reply is what "include" is for, we shouldn't be able to reach this.
//                 tracing::error!("Found error on cast {} (is from DWR and is not a reply, but is not included)", token_id);
//                 invalid_tokens.push(format!("{} (not a valid DWRcast)", token_id));
//                 continue;
//             }
//         } else {
//             invalid_tokens.push(format!("{} (cast not found)", token_id));
//         }
//     }

//     if !invalid_tokens.is_empty() {
//         return Err((
//             StatusCode::BAD_REQUEST,
//             Json(WrapError {
//                 error: format!("Some of the selected NFTs are not valid for wrapping: {}", invalid_tokens.join(", ")),
//                 invalid_tokens: Some(invalid_tokens),
//             }),
//         ));
//     }

//     // Generate EIP712 signature
//     let signer = Eip712Signer::new(&config.private_key_signer, BASE_MAINNET_CHAIN_ID)
//         .map_err(|e| (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(WrapError {
//                 error: format!("Failed to initialize signer: {}", e),
//                 invalid_tokens: None,
//             }),
//         ))?;

//     let contract_address: Address = config.dwrcasts_contract_address.parse()
//         .map_err(|_| (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(WrapError {
//                 error: "Invalid contract address in config".to_string(),
//                 invalid_tokens: None,
//             }),
//         ))?;

//     let nonce = Eip712Signer::generate_nonce();
//     let deadline = Eip712Signer::generate_deadline_10_minutes();

//     let signature_data = signer.sign_wrap_permit(
//         contract_address,
//         wallet_address,
//         token_ids_u256,
//         nonce,
//         deadline,
//     ).await
//     .map_err(|e| (
//         StatusCode::INTERNAL_SERVER_ERROR,
//         Json(WrapError {
//             error: format!("Failed to generate signature: {}", e),
//             invalid_tokens: None,
//         }),
//     ))?;

//     Ok(Json(WrapResponse {
//         signature: signature_data.signature,
//         nonce: nonce.to_string(),
//         deadline,
//         token_ids: req.token_ids,
//     }))
// }
