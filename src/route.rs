use crate::{
    Body, BodyExt, Bytes, Deserialize, Duration, FromRequest, IntoResponse, Json, Level,
    MatchedPath, Next, PgPool, Request, Response, Router, State, StatusCode, TraceLayer,
    Validation,
    controller::user_accounts,
    dao::user_accounts::{UserAccountsDao, new_user_accounts_dao},
    debug, decode, error, event, get,
    init::KEYS,
    jwt::Claims,
    middleware, patch, post, put,
    response::{self, ErrCode, Resp},
};

pub fn new_route(pool: PgPool) -> Router {
    let users_route = Router::new()
        .route("/test", post(user_accounts::test_user))
        .route("/list", post(user_accounts::list))
        .route("/{user_id}", get(user_accounts::one))
        .route("/register", post(user_accounts::register))
        .route("/login", post(user_accounts::login));

    let api = Router::new()
        .nest("/api/v1/users", users_route)
        .layer(
            TraceLayer::new_for_http()
                .on_request(|req: &Request<_>, _span: &tracing::Span| {
                    event!(
                        Level::INFO,
                        "Incoming request: method={}, uri={}, matched_path={}",
                        req.method(),
                        req.uri(),
                        req.extensions()
                            .get::<MatchedPath>()
                            .map(|matched_path| matched_path.as_str())
                            .unwrap_or("unknown")
                    );
                })
                .on_response(
                    |response: &axum::response::Response,
                     latency: Duration,
                     _span: &tracing::Span| {
                        if response.status() != StatusCode::OK {
                            event!(
                                Level::ERROR,
                                "Response completed: status={}, latency={:?}",
                                response.status(),
                                latency
                            );
                        }
                    },
                ),
        )
        .layer(middleware::from_fn_with_state(
            pool.clone(),
            user_info_middleware,
        ))
        .with_state(pool);

    let app = api.route("/", get(async || "hello, world"));

    app.fallback(async || (StatusCode::NOT_FOUND, "not found!"))
}

#[derive(Deserialize)]
struct UserId {
    user_id: i32,
}

async fn user_info_middleware(
    State(pool): State<PgPool>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    // 分离请求的 parts 和 body
    let (parts, body) = req.into_parts();

    // 收集 body 字节
    let (bytes, user) = match body.collect().await {
        Ok(collected) => {
            let bytes = collected.to_bytes();
            // 创建临时请求用于 JSON 提取
            let temp_body = Body::from(bytes.clone());
            let temp_req = Request::from_parts(parts.clone(), temp_body);

            // 提取 JSON 并查询用户信息
            let user = if let Ok(Json(user_id)) = Json::<UserId>::from_request(temp_req, &()).await
            {
                let dao = new_user_accounts_dao(pool);
                dao.select_by_id(user_id.user_id).await.ok()
            } else {
                None
            };
            (bytes, user)
        }
        Err(_) => (Bytes::new(), None),
    };

    let req = Request::from_parts(parts, Body::from(bytes));

    // 将 UserInfo 插入到请求的 extensions 中
    let mut req = req;
    req.extensions_mut().insert(user);
    next.run(req).await
}

async fn auth_middleware(mut req: Request, next: Next) -> Box<dyn IntoResponse> {
    let token = match req.headers().get("authorization") {
        Some(header) => {
            let str = match header.to_str() {
                Ok(s) => s,
                Err(_) => {
                    return Box::new(response::make_response::<Resp<()>>(Err(
                        ErrCode::UnAuthorized,
                    )));
                }
            };
            match str.strip_prefix("Bearer ") {
                Some(t) => t,
                None => {
                    return Box::new(response::make_response::<Resp<()>>(Err(
                        ErrCode::UnAuthorized,
                    )));
                }
            }
        }
        None => {
            return Box::new(response::make_response::<Resp<()>>(Err(
                ErrCode::UnAuthorized,
            )));
        }
    };

    debug!("Extracted token: {}", token);

    let claims = match decode::<Claims>(token, &KEYS.decoding, &Validation::default()) {
        Ok(data) => data.claims,
        Err(e) => {
            error!("JWT decode error: {}", e);
            return Box::new(response::make_response::<Resp<()>>(Err(
                ErrCode::InvalidToken,
            )));
        }
    };

    // Insert claims into request extensions for downstream handlers
    req.extensions_mut().insert(claims);

    // Proceed to the next middleware or handler
    Box::new(next.run(req).await)
}
