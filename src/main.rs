mod ai_client;
mod api_response;
mod app_state;
mod config;
mod error;
mod service;

use app_state::State;
use axum::Router;
use axum::routing::post;
use config::Config;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Config::from_env()?;
    let app_state = State::new(&config)?;

    let listener = TcpListener::bind("0.0.0.0:6651").await?;
    let app = Router::new()
        .route("/", post(service::webhook))
        .with_state(app_state);
    axum::serve(listener, app).await?;

    Ok(())
}
