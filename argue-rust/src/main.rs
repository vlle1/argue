use axum::{
    routing::{get, get_service},
    Router,
};
use tower_http::services::{ServeDir, ServeFile};

mod openai;
mod model;
mod socket_handler;

#[tokio::main]
async fn main() {
    use dotenv::dotenv;
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let static_service = ServeDir::new("./argue-react/build")
        .fallback(ServeFile::new("./argue-react/build/index.html"));
    let router = Router::new()
        .route("/ws", get(socket_handler::ws_route_handler))
        .nest_service("/", get(get_service(static_service)));
    //output the address
    //println!("WS: Listening on: ws://{}/ws", SOCKET_ADRESS);
    let ws_port = std::env::var("WS_PORT").unwrap();
    axum::Server::bind(&format!("0.0.0.0:{}", ws_port).parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
