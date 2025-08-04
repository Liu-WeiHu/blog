use axum::{
    RequestPartsExt, Router,
    extract::{FromRequest, FromRequestParts, MatchedPath, Path, Query, Request, State},
    http::{self, StatusCode, request::Parts},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
};

use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::LazyLock;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tracing::{Level, event};

use jsonwebtoken::{DecodingKey, EncodingKey, Validation, decode};
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
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
