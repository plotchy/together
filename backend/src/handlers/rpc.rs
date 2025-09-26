use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    Json as JsonBody,
};
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{Config, PgPool};

// Rate limiting structure for RPC requests
#[derive(Debug)]
struct RpcRateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    max_requests_per_minute: u32,
}

impl RpcRateLimiter {
    fn new(max_requests_per_minute: u32) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests_per_minute,
        }
    }

    async fn can_make_request(&mut self, key: &str) -> bool {
        let now = Instant::now();
        let minute_ago = now - Duration::from_secs(60);
        
        let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests
        requests.retain(|&time| time > minute_ago);
        
        if requests.len() < self.max_requests_per_minute as usize {
            requests.push(now);
            true
        } else {
            false
        }
    }
}

// Global rate limiter instance
use once_cell::sync::Lazy;

static RPC_RATE_LIMITER: Lazy<Arc<Mutex<RpcRateLimiter>>> = 
    Lazy::new(|| Arc::new(Mutex::new(RpcRateLimiter::new(300)))); // 300 requests per minute

#[derive(serde::Deserialize)]
pub struct RpcQuery {
    #[serde(default)]
    network: Option<String>,
}

/// Proxy RPC requests to avoid frontend rate limiting
/// 
/// This endpoint forwards JSON-RPC requests to the appropriate network endpoint.
/// It includes rate limiting per IP and basic request validation.
pub async fn proxy_rpc(
    Query(params): Query<RpcQuery>,
    headers: HeaderMap,
    State((_pool, _config)): State<(PgPool, Config)>,
    JsonBody(body): JsonBody<Value>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get client IP for rate limiting
    let client_ip = get_client_ip(&headers);
    
    // Rate limiting
    {
        let mut rate_limiter = RPC_RATE_LIMITER.lock().await;
        if !rate_limiter.can_make_request(&client_ip).await {
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "error": "Rate limit exceeded. Please try again later."
                }))
            ));
        }
    }

    // Determine the RPC endpoint based on network parameter
    let rpc_url = match params.network.as_deref() {
        Some("base") | None => {
            // Default to Base network, use Alchemy if available
            if let Ok(alchemy_key) = std::env::var("ALCHEMY_API_KEY") {
                format!("https://base-mainnet.g.alchemy.com/v2/{}", alchemy_key)
            } else {
                "https://mainnet.base.org".to_string()
            }
        },
        Some("ethereum") => {
            if let Ok(alchemy_key) = std::env::var("ALCHEMY_API_KEY") {
                format!("https://eth-mainnet.g.alchemy.com/v2/{}", alchemy_key)
            } else {
                "https://eth.llamarpc.com".to_string()
            }
        },
        Some("polygon") => {
            if let Ok(alchemy_key) = std::env::var("ALCHEMY_API_KEY") {
                format!("https://polygon-mainnet.g.alchemy.com/v2/{}", alchemy_key)
            } else {
                "https://polygon-rpc.com".to_string()
            }
        },
        Some(network) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": format!("Unsupported network: {}", network)
                }))
            ));
        }
    };

    // Validate JSON-RPC request structure
    if !is_valid_jsonrpc_request(&body) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid JSON-RPC request format"
            }))
        ));
    }

    // Log the request for monitoring
    tracing::info!(
        "RPC proxy request from {} to {} network: method={}",
        client_ip,
        params.network.as_deref().unwrap_or("base"),
        body.get("method").and_then(|m| m.as_str()).unwrap_or("unknown")
    );

    // Forward the request
    let client = Client::new();
    let response = client
        .post(&rpc_url)
        .json(&body)
        .timeout(Duration::from_secs(30))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<Value>().await {
                    Ok(json_response) => Ok(Json(json_response)),
                    Err(e) => {
                        tracing::error!("Failed to parse RPC response as JSON: {}", e);
                        Err((
                            StatusCode::BAD_GATEWAY,
                            Json(json!({
                                "error": "Invalid response from RPC endpoint"
                            }))
                        ))
                    }
                }
            } else {
                tracing::error!("RPC endpoint returned error status: {}", resp.status());
                Err((
                    StatusCode::BAD_GATEWAY,
                    Json(json!({
                        "error": "RPC endpoint error"
                    }))
                ))
            }
        }
        Err(e) => {
            tracing::error!("Failed to connect to RPC endpoint: {}", e);
            Err((
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": "Failed to connect to RPC endpoint"
                }))
            ))
        }
    }
}

fn get_client_ip(headers: &HeaderMap) -> String {
    // Try various headers that might contain the real IP
    let ip_headers = [
        "cf-connecting-ip",      // Cloudflare
        "x-forwarded-for",       // Standard proxy header
        "x-real-ip",            // Nginx
        "x-client-ip",          // Apache
    ];

    for header_name in &ip_headers {
        if let Some(header_value) = headers.get(*header_name) {
            if let Ok(ip_str) = header_value.to_str() {
                // x-forwarded-for can be a comma-separated list, take the first one
                let ip = ip_str.split(',').next().unwrap_or(ip_str).trim();
                if !ip.is_empty() {
                    return ip.to_string();
                }
            }
        }
    }

    // Fallback to "unknown"
    "unknown".to_string()
}

fn is_valid_jsonrpc_request(body: &Value) -> bool {
    // Check if it's a valid JSON-RPC request
    if let Some(obj) = body.as_object() {
        // Must have jsonrpc, method, and id fields
        obj.contains_key("jsonrpc") && 
        obj.contains_key("method") && 
        obj.contains_key("id") &&
        obj.get("jsonrpc").and_then(|v| v.as_str()) == Some("2.0") &&
        obj.get("method").and_then(|v| v.as_str()).is_some()
    } else if let Some(array) = body.as_array() {
        // Batch request - validate each item
        !array.is_empty() && array.iter().all(is_valid_jsonrpc_request)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_jsonrpc_request() {
        let valid_request = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": ["0x1234", "latest"],
            "id": 1
        });
        
        assert!(is_valid_jsonrpc_request(&valid_request));
    }

    #[test]
    fn test_invalid_jsonrpc_request() {
        let invalid_request = json!({
            "method": "eth_getBalance",
            "params": ["0x1234", "latest"]
            // Missing jsonrpc and id
        });
        
        assert!(!is_valid_jsonrpc_request(&invalid_request));
    }

    #[test]
    fn test_batch_jsonrpc_request() {
        let batch_request = json!([
            {
                "jsonrpc": "2.0",
                "method": "eth_getBalance",
                "params": ["0x1234", "latest"],
                "id": 1
            },
            {
                "jsonrpc": "2.0", 
                "method": "eth_blockNumber",
                "params": [],
                "id": 2
            }
        ]);
        
        assert!(is_valid_jsonrpc_request(&batch_request));
    }
}
