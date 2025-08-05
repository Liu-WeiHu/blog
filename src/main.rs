use axum::{
    Extension, RequestPartsExt, Router,
    body::Body,
    extract::{FromRequest, FromRequestParts, Json, MatchedPath, Path, Query, Request, State},
    http::{StatusCode, request::Parts},
    middleware::{self, Next},
    response::IntoResponse,
    response::Response,
    routing::{get, patch, post, put},
};

use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use bytes::Bytes;
use chrono::NaiveDateTime;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::LazyLock;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tracing::{Level, debug, event, info};

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
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
