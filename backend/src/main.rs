use axum::routing::{get, get_service};
use axum::Router;
use tower_http::services::ServeDir;

mod ai;
mod config;
mod model;
mod openai;
mod routes;
mod socket_handler;

pub use config::*;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    use dotenv::dotenv;
    dotenv().ok();

    tracing_subscriber::fmt().with_max_level(tracing::Level::TRACE).init();

    let config_path = std::env::var("CONFIG_FILE").unwrap_or("argue.toml".into());
    let config: Config = std::fs::read_to_string(config_path)
        .map(|s| toml::from_str(&s).unwrap())
        .unwrap_or_default();

    let listener = tokio::net::TcpListener::bind(config.address).await?;
    let static_service = ServeDir::new(config.serve_dir);

    let app = Router::new()
        .route("/api/create", get(routes::create_game))
        // .route("/ws", get(socket_handler::ws_route_handler))
        .nest_service("/", get(get_service(static_service)));

    axum::serve(listener, app).await
}
