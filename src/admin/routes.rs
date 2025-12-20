use axum::{
    Router,
    routing::{get, put},
};

use super::handlers::{
    add_to_list_handler, get_config_handler, get_health_handler, get_list_handler, is_domain_in,
    remove_from_list_handler, update_ad_list_handler, update_config_handler,
};

pub fn create_config_routes() -> Router {
    Router::new().route(
        "/config",
        get(get_config_handler).put(update_config_handler),
    )
}

pub fn create_health_routes() -> Router {
    Router::new().route("/health", get(get_health_handler))
}

pub fn create_list_routes() -> Router {
    Router::new()
        .route(
            "/list",
            get(get_list_handler)
                .post(add_to_list_handler)
                .delete(remove_from_list_handler),
        )
        .route("/list/update-ads", put(update_ad_list_handler))
        .route("/list/{domain}", get(is_domain_in))
}
