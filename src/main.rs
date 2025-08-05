use axum::{
    Extension, Router,
    body::Body,
    extract::{FromRequest, Json, MatchedPath, Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};

use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, NaiveDateTime, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower_http::trace::TraceLayer;
use tracing::{Level, debug, error, event, info};

use std::sync::LazyLock;
use std::time::Duration;

mod controller;
mod dao;
mod init;
mod jwt;
mod model;
mod pagination;
mod response;
mod route;
mod service;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let pool = init::get_db_pool().await;
    let app = route::new_route(pool);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
