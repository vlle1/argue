use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade, State,
    },
    response::Response
};
use futures_util::SinkExt;
use tokio::sync::Mutex;

use crate::model::{ClientMessage, GameState, messenger::PlayerId};
use crate::model::messenger::{SingleMessenger, BroadcastMessenger};
pub struct AppState {
    game_states: HashMap<String, Arc<Mutex<GameState>>>,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            game_states: HashMap::new(),
        }
    }
    pub fn get_or_create_game_state(&self, root_statement: String) -> Arc<Mutex<GameState>> {
        match self.game_states.get(&root_statement).map(|x| x.clone()) {
            Some(game_state) => game_state,
            None => {
                let mut game_state = Arc::new(Mutex::new(GameState::new(root_statement.clone(), BroadcastMessenger::new())));
            self.game_states.insert(root_statement, game_state );
            game_state
            },
        }
    }
}

pub async fn ws_route_handler(ws: WebSocketUpgrade, State(state): State<Arc<Mutex<AppState>>>, ) -> Response {
    println!("ws_route_handler");
    ws.on_failed_upgrade(|error| {
        println!("failed to upgrade: {}", error);
        Response::builder().status(500).body("failed to upgrade").unwrap();
    }).on_upgrade(|socket| handle_socket(socket, "".into(), true, state))//FIXME
}

/// one method call = one websocket connection
async fn handle_socket(socket: WebSocket, root_statement: String, private: bool, state: Arc<Mutex<AppState>>) {
    use futures_util::stream::StreamExt;
    println!("handle socket");
    let (mut sender, mut receiver) = socket.split();
    //gamestate
    let (mut game_state, player_id) : (GameState, PlayerId) = if private {
        let messenger = SingleMessenger::new(sender);
        (GameState::new(root_statement.to_string(), messenger), 0)
    } else {
        let mut game_state: GameState = state.lock().await.get_or_create_game_state(root_statement);
        let player_id = (game_state.messenger as BroadcastMessenger).add_player(sender); //TODO generics, um hier casten zu vermeiden?
        (game_state, player_id)
    };
    
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
        if let Some(response) = game_state.on_incoming_message(client_msg, player_id).await {
            sender.lock().await.send(Message::Text(serde_json::to_string(&response).unwrap()));
        }
    };
    //connection closed?
    if !private {
        (game_state.messenger as BroadcastMessenger).remove_player(player_id);
    }
}
