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
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Config::from_env()?;
    let app_state = State::new(&config)?;

    let listener = TcpListener::bind("0.0.0.0:6651").await?;
    let app = Router::new()
        .route("/", post(service::webhook))
        .with_state(app_state);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
