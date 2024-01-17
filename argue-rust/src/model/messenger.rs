use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use generational_arena::Index;
use std::collections::HashMap;

use super::ServerMessage;
use super::TreeStateDTO;
use super::proof::TreeState;

pub type PlayerId = u64;
pub type Sender = SplitSink<WebSocket, Message>;

/// handles communication between client and server.
pub struct BroadcastMessenger {
    sender: HashMap<PlayerId, Sender>,
    next_id: PlayerId,
}

impl BroadcastMessenger {
    pub fn new() -> Self {
        Self {
            sender: HashMap::new(),
            next_id: 0,
        }
    }
    pub fn add_player(&mut self, sender: Sender) -> PlayerId {
        self.sender.insert(self.next_id, sender);
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    pub fn remove_player(&mut self, player_id: PlayerId) {
        self.sender.remove(&player_id);
    }
    pub async fn send(&mut self, msg: ServerMessage) {
        let m: Message = Message::Text(serde_json::to_string(&msg).unwrap());
        let send_all = self
            .sender
            .values_mut()
            .map(|sender| sender.send(m.to_owned()));
        let _ = futures_util::future::join_all(send_all).await;
    }
    pub async fn reply(&mut self, msg: ServerMessage, player_id: PlayerId) {
        match self.sender.get_mut(&player_id) {
            Some(sender) => {
                let m: Message = Message::Text(serde_json::to_string(&msg).unwrap());
                let _ = sender.send(m).await;
            }
            None => {
                println!("Reply error: Player {} not found", player_id);
            }
        }
    }
    pub async fn send_cooldown(&mut self, seconds: u64) {
        let _ = self.send(ServerMessage::AICooldown { seconds }).await;
    }
    pub async fn send_tree(&mut self, tree: &TreeState) {
        //push game state to client(s)
        let tree_dto: TreeStateDTO = tree.as_dto();
        let _ = self.send(ServerMessage::GameState(tree_dto)).await;
    }
    pub async fn msg(&mut self, id: Index, comment: String, success: bool) {
        //append message to node
        let _ = self
            .send(ServerMessage::Comment {
                id,
                comment,
                success,
            })
            .await;
    }
    pub async fn msg_win(&mut self) {
        let _ = self.send(ServerMessage::Win).await;
    }
    pub async fn reply_tree(&mut self, tree: &TreeState, player_id: PlayerId) {
        let tree_dto: TreeStateDTO = tree.as_dto();
        let _ = self
            .reply(ServerMessage::GameState(tree_dto), player_id)
            .await;
    }
}
