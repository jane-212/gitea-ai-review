use axum::Router;
use axum::response::IntoResponse;
use axum::routing::post;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:6651").await?;
    let router = Router::new().route("/review", post(root));
    axum::serve(listener, router).await?;

    Ok(())
}

async fn root() -> impl IntoResponse {
    "this is review"
}
