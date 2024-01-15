use axum::{Router, routing::{get, get_service}};
use tower_http::services::{ServeDir, ServeFile};


mod api;
mod model;
mod socket_handler;

const PORT: &str = "8100";

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt()
    .with_max_level(tracing::Level::TRACE)
    .init();

    let static_service =
        ServeDir::new("../argue-react/build").fallback(ServeFile::new("../argue-react/build/index.html"));
    let router = Router::new()
        .route("/ws", get(socket_handler::ws_route_handler))
        .nest_service("/", get(get_service(static_service)));
    //output the address
    //println!("WS: Listening on: ws://{}/ws", SOCKET_ADRESS);
    axum::Server::bind(&format!("0.0.0.0:{}", PORT).parse().unwrap())
    .serve(router.into_make_service())
    .await
    .unwrap();
}
