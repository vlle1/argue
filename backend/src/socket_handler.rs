use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::Response;
use futures_util::stream::StreamExt;

use crate::model::{ClientMessage, GameState, Messenger};

pub async fn ws_route_handler(ws: WebSocketUpgrade) -> Response { ws.on_upgrade(|socket| handle_socket(socket)) }

/// one method call = one websocket connection
async fn handle_socket(socket: WebSocket) {
    let (sender, mut receiver) = socket.split();

    //gamestate
    let messenger = Messenger { sender };
    let mut game_state = GameState::new("".to_string(), messenger);
    while let Some(result) = receiver.next().await {
        let msg = match result {
            Ok(Message::Text(msg)) => msg,
            Ok(Message::Binary(_) | Message::Ping(_) | Message::Pong(_)) => continue,
            Ok(Message::Close(_)) | Err(_) => return,
        };

        let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&msg) else {
            eprintln!("received invalid message: {}", msg);
            continue;
        };
        game_state.on_incoming_message(client_msg).await;
    }
}
