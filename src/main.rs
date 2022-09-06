use axum::response::{IntoResponse, Json};
use axum::{routing::get, Router};
use hyper::StatusCode;
use time::UtcOffset;
use tower_http::compression::CompressionLayer;
use tower_http::cors;
use tower_http::cors::Any;

async fn get_data() -> impl IntoResponse {
    let date = time::OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(-6, 0, 0).unwrap());

    let resp = reqwest::get(format!(
        "https://baseballsavant.mlb.com/schedule?date={}-{}-{}",
        date.year(),
        date.month() as u8,
        date.day()
    ))
    .await
    .unwrap()
    .json::<serde_json::Value>()
    .await
    .unwrap();

    (StatusCode::OK, Json(resp))
}

async fn sup() -> impl IntoResponse {
    (StatusCode::OK, "sup x2")
}

#[tokio::main]
async fn main() {
    let cors = cors::CorsLayer::new().allow_methods(Any).allow_origin(Any);

    // Wrap an `axum::Router` with our state, CORS, Tracing, & Compression layers
    let app = Router::new()
        .route("/", get(get_data))
        .route("/sup", get(sup))
        .layer(CompressionLayer::new())
        .layer(cors);

    #[cfg(debug_assertions)]
    {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    // If we compile in release mode, use the Lambda Runtime
    #[cfg(not(debug_assertions))]
    {
        // To run with AWS Lambda runtime, wrap in our `LambdaLayer`
        let app = tower::ServiceBuilder::new()
            .layer(axum_aws_lambda::LambdaLayer::default())
            .service(app);

        lambda_http::run(app).await.unwrap();
    }
}
