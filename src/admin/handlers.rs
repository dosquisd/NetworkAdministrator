use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpStream,
    time::{self as TokioTime, Duration},
};

use crate::config::{
    constants::CONFIG_PATH,
    settings::{ProxyConfig, get_global_config, set_global_config},
};
use crate::filters::{
    ListConfigType, add_domain_to_blacklist, add_domain_to_whitelist, get_blacklist, get_whitelist,
    is_domain_blacklisted, is_domain_whitelisted, merge_from_file, remove_domain_from_blacklist,
    remove_domain_from_whitelist, replace_from_file,
};

// ============================================================
// Config Handlers
// ============================================================

pub async fn get_config_handler() -> Json<ProxyConfig> {
    let config = get_global_config();
    Json(config)
}

pub async fn update_config_handler(Json(payload): Json<ProxyConfig>) -> Json<ProxyConfig> {
    tracing::info!("Config update requested: {:?}", payload);
    set_global_config(payload.clone());
    tracing::info!("Config updated successfully");
    Json(get_global_config())
}

// ============================================================
// Health Handlers
// ============================================================

#[derive(Serialize)]
pub struct HealthResponse {
    pub admin_server_status: String,
    pub proxy_server_status: String,
}

#[derive(Deserialize)]
pub struct HealthQuery {
    pub proxy_port: Option<u16>,
    pub detailed: Option<bool>,
}

pub async fn get_health_handler(
    health_query: Query<HealthQuery>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let health_query = health_query.0;
    let detailed = health_query.detailed.unwrap_or(false);

    match (health_query.proxy_port, detailed) {
        (Some(port), true) => {
            let addr = format!("127.0.0.1:{}", port);
            let mut connect =
                TokioTime::timeout(Duration::from_secs(5), TcpStream::connect(addr)).await;

            if connect.is_err() {
                return Err(StatusCode::REQUEST_TIMEOUT);
            }

            if connect.as_mut().unwrap().is_err() {
                return Err(StatusCode::SERVICE_UNAVAILABLE);
            }

            let response = HealthResponse {
                admin_server_status: "Running".to_string(),
                proxy_server_status: format!("Proxy on port {} is healthy", port),
            };
            Ok(Json(response))
        }
        (None, true) => Err(StatusCode::BAD_REQUEST),
        _ => {
            // Basic health check
            let response = HealthResponse {
                admin_server_status: "Running".to_string(),
                proxy_server_status: "Running".to_string(),
            };
            Ok(Json(response))
        }
    }
}

// ============================================================
// List Handlers
// ============================================================

#[derive(Deserialize)]
pub struct ListQuery {
    pub is_blacklist: bool,
    pub text: Option<String>,
    pub config_type: ListConfigType,
}

#[derive(Serialize)]
pub struct ListResponse {
    pub entries: Vec<String>,
    pub total: usize,
    pub config_type: ListConfigType,
    pub is_blacklist: bool,
}

pub async fn get_list_handler(query: Query<ListQuery>) -> Json<ListResponse> {
    let query = query.0;
    match query.is_blacklist {
        true => {
            let list = get_blacklist(query.config_type);
            Json(ListResponse {
                entries: list.clone(),
                total: list.len(),
                config_type: query.config_type,
                is_blacklist: true,
            })
        }
        false => {
            let list = get_whitelist(query.config_type);
            Json(ListResponse {
                entries: list.clone(),
                total: list.len(),
                config_type: query.config_type,
                is_blacklist: false,
            })
        }
    }
}

pub async fn add_to_list_handler(query: Query<ListQuery>) -> Result<StatusCode, StatusCode> {
    let query = query.0;

    let text = query.text.ok_or(StatusCode::BAD_REQUEST)?;
    if text.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if matches!(query.config_type, ListConfigType::Regex) {
        if regex::Regex::new(&text).is_err() {
            tracing::warn!("Invalid regex pattern: {}", text);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    tracing::info!(
        "Adding '{}' to {} (type: {:?})",
        text,
        if query.is_blacklist {
            "blacklist"
        } else {
            "whitelist"
        },
        query.config_type
    );

    match query.is_blacklist {
        true => add_domain_to_blacklist(&text, query.config_type)
            .map(|_| StatusCode::CREATED)
            .map_err(|e| {
                tracing::error!("Failed to add to blacklist: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }),
        false => add_domain_to_whitelist(&text, query.config_type)
            .map(|_| StatusCode::CREATED)
            .map_err(|e| {
                tracing::error!("Failed to add to whitelist: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }),
    }
}

pub async fn remove_from_list_handler(query: Query<ListQuery>) -> Result<StatusCode, StatusCode> {
    let query = query.0;

    let text = query.text.ok_or(StatusCode::BAD_REQUEST)?;
    if text.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if matches!(query.config_type, ListConfigType::Regex) {
        if regex::Regex::new(&text).is_err() {
            tracing::warn!("Invalid regex pattern: {}", text);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    tracing::info!(
        "Removing '{}' from {} (type: {:?})",
        text,
        if query.is_blacklist {
            "blacklist"
        } else {
            "whitelist"
        },
        query.config_type
    );

    match query.is_blacklist {
        true => remove_domain_from_blacklist(&text, query.config_type)
            .map(|_| StatusCode::OK)
            .map_err(|e| {
                tracing::error!("Failed to remove of blacklist: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }),
        false => remove_domain_from_whitelist(&text, query.config_type)
            .map(|_| StatusCode::OK)
            .map_err(|e| {
                tracing::error!("Failed to remove of blacklist: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }),
    }
}

#[derive(Deserialize)]
pub struct UpdateAdListQuery {
    pub hard: Option<bool>,
}

pub async fn update_ad_list_handler(
    Query(query): Query<UpdateAdListQuery>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!("Updating ad list...");

    let updated_filter_path = CONFIG_PATH.join("filter.updated.toml");

    let result = if query.hard.unwrap_or(false) {
        tracing::info!("Performing hard update (replace)");
        replace_from_file(updated_filter_path)
    } else {
        tracing::info!("Performing soft update (merge)");
        merge_from_file(updated_filter_path)
    };

    match result {
        Ok(_) => {
            tracing::info!("Ad list updated successfully");
            Ok(StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Failed to update ad list: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct IsDomainInQuery {
    pub is_blacklist: bool,
}

#[derive(Serialize)]
pub struct IsDomainInResponse {
    pub found: bool,
}

pub async fn is_domain_in(
    Path(domain): Path<String>,
    Query(query): Query<IsDomainInQuery>,
) -> Json<IsDomainInResponse> {
    let found = match query.is_blacklist {
        true => is_domain_blacklisted(&domain),
        false => is_domain_whitelisted(&domain),
    };

    Json(IsDomainInResponse { found })
}
