use std::sync::Arc;

use axum::http::header;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderName, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json,
    Router,
};
use log::info;
use poise::serenity_prelude::{GuildId, UserId};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

use botox::cache::CacheHttpImpl;

pub struct AppState {
    pub cache_http: CacheHttpImpl,
    pub pool: PgPool,
}

pub async fn setup_server(pool: PgPool, cache_http: CacheHttpImpl) {
    let shared_state = Arc::new(AppState { pool, cache_http });

    let app = Router::new()
        .route("/api/stats", get(stats))
        .route("/api/commands", get(commands))
        .route("/:gid", get(create_login))
        .route("/confirm-login", get(confirm_login))
        .with_state(shared_state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let bind_addr = std::env::var("API_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:4950".to_string());
    let addr = bind_addr.parse().expect("Invalid API_BIND_ADDR");

    info!("Starting server on {}", addr);

    if let Err(e) = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
    {
        panic!("server error: {}", e);
    }
}

async fn stats() -> Json<serde_json::Value> {
    Json(json!({
        "status": "online",
        "name": "Sentinel",
        "service": "discord-moderation-safety",
        "version": crate::stats::VERSION,
        "commit": crate::stats::GIT_SHA,
        "semver": crate::stats::GIT_SEMVER,
        "capabilities": ["audit monitoring", "thresholds", "automatic response", "owner alerts"],
    }))
}

async fn commands() -> Json<serde_json::Value> {
    Json(json!({
        "name": "Sentinel",
        "commands": [
            { "name": "help", "group": "Essentials", "description": "Browse Sentinel commands and usage." },
            { "name": "simplehelp", "group": "Essentials", "description": "Get a compact command reference." },
            { "name": "ping", "group": "Essentials", "description": "Check whether Sentinel is responding." },
            { "name": "stats", "group": "Essentials", "description": "View Sentinel build and runtime details." },
            { "name": "setup", "group": "Server", "description": "Initialize Sentinel protection for a server." },
            { "name": "settings", "group": "Server", "description": "Configure audit notification channel and color." },
            { "name": "limits guide", "group": "Protection", "description": "Understand thresholds, windows, and responses." },
            { "name": "limits add", "group": "Protection", "description": "Create a threshold for a moderation event." },
            { "name": "limits view", "group": "Protection", "description": "Review active protection thresholds." },
            { "name": "limits hit", "group": "Protection", "description": "Review triggered thresholds." },
            { "name": "limits remove", "group": "Protection", "description": "Remove a configured threshold." },
            { "name": "actions view", "group": "Audit", "description": "Review recorded moderation actions." },
            { "name": "perms", "group": "Administration", "description": "Manage Sentinel administrators." }
        ]
    }))
}

enum ServerError {
    Error(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::Error(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    }
}

async fn create_login(Path(gid): Path<UserId>) -> Redirect {
    // Redirect user to the login page
    let url = format!("https://discord.com/api/oauth2/authorize?client_id={}&redirect_uri={}/confirm-login&scope={}&state={}&response_type=code", crate::config::CONFIG.client_id, crate::config::CONFIG.frontend_url, "identify", gid);

    Redirect::temporary(&url)
}

#[derive(Deserialize)]
struct AccessToken {
    access_token: String,
}

#[derive(Deserialize)]
struct ConfirmLogin {
    code: String,
    state: GuildId,
}

async fn confirm_login(
    State(app_state): State<Arc<AppState>>,
    data: Query<ConfirmLogin>,
) -> Result<([(HeaderName, &'static str); 2], String), ServerError> {
    // Create access token from code
    let client = reqwest::Client::new();

    let access_token = client
        .post("https://discord.com/api/v10/oauth2/token")
        .form(&json!({
            "client_id": crate::config::CONFIG.client_id,
            "client_secret": crate::config::CONFIG.client_secret,
            "grant_type": "authorization_code",
            "code": data.code,
            "redirect_uri": format!("{}/confirm-login", crate::config::CONFIG.frontend_url),
        }))
        .send()
        .await
        .map_err(|_| ServerError::Error("Could not send request to get access token".to_string()))?
        .error_for_status()
        .map_err(|e| ServerError::Error(format!("Could not get access token: {}", e)))?;

    let access_token = access_token
        .json::<AccessToken>()
        .await
        .map_err(|_| ServerError::Error("Could not deserialize response".to_string()))?;

    // Get user from access token
    let user = client
        .get("https://discord.com/api/v10/users/@me")
        .header(
            "Authorization",
            format!("Bearer {}", access_token.access_token),
        )
        .send()
        .await
        .map_err(|_| ServerError::Error("Could not send request to get user".to_string()))?
        .error_for_status()
        .map_err(|_| ServerError::Error("Get User failed!".to_string()))?;

    let user = user
        .json::<serenity::model::user::User>()
        .await
        .map_err(|_| ServerError::Error("Could not deserialize response".to_string()))?;

    // Check that user is a guild admin
    crate::utils::is_guild_admin(
        &app_state.cache_http,
        &app_state.pool,
        data.state,
        user.id.to_string(),
    )
    .await
    .map_err(|e| ServerError::Error(e.to_string()))?;

    // Find all actions
    let actions = crate::core::Action::guild(&app_state.pool, data.state)
        .await
        .map_err(|e| ServerError::Error(e.to_string()))?;

    // Convert to json
    let actions = serde_json::to_string(&actions)
        .map_err(|_| ServerError::Error("Could not serialize actions".to_string()))?;

    let headers = [
        (
            header::CONTENT_TYPE,
            "application/octet-stream; charset=utf-8",
        ),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"data.json\"",
        ),
    ];

    Ok((headers, actions))
}
