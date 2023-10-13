use aws_sdk_s3 as s3;
use axum::body::boxed;
use axum::http::StatusCode;
use axum::response::{AppendHeaders, Response};
use axum::{body::StreamBody, extract::Path, http::header, routing::get, Extension, Router};
use s3::Client;
use tokio_util::io::ReaderStream;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

async fn download_object(
    Path(file): Path<String>,
    Extension(s3_client): Extension<Client>,
) -> Result<Response, StatusCode> {
    let bucket = std::env::var("AWS_S3_BUCKET").unwrap();

    let object = s3_client.get_object().bucket(bucket).key(file);

    let data = object.send().await.unwrap();

    let stream = data.body.into_async_read();

    let reader_stream = ReaderStream::new(stream);

    let body = StreamBody::new(reader_stream);

    let _headers = AppendHeaders([
        (header::CONTENT_TYPE, data.content_type),
        (header::CONTENT_DISPOSITION, data.content_disposition),
        (header::ACCEPT_RANGES, data.accept_ranges),
    ]);

    let response = Response::builder().body(boxed(body)).unwrap();

    Ok(response)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let cors_layer = CorsLayer::permissive();

    let aws_configuration = aws_config::load_from_env().await;

    let aws_s3_client = s3::Client::new(&aws_configuration);

    let app = Router::new()
        .route("/:file", get(download_object))
        .layer(cors_layer)
        .layer(Extension(aws_s3_client));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("starting server on port: {}", addr.port());
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to start server");
}
