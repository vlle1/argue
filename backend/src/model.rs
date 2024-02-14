use std::env;
use std::result::Result;
use std::time::Instant;

use crate::model::ClientMessage::*;

use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use generational_arena::Index;
use serde::{Deserialize, Serialize};

use self::proof::{ProofError, ProofState, TreeState};
mod proof;

#[derive(Serialize)]
pub struct StatementDTO {
    id: Index,
    statement: String,
    state: ProofState,
    parents: Vec<Index>,
    children: Vec<Index>,
}

#[derive(Serialize)]
pub struct TreeStateDTO {
    statements: Vec<StatementDTO>,
    root: Index,
}

#[derive(Serialize)]
pub enum ServerMessage {
    NewNodeId(Index),
    GameState(TreeStateDTO),
    Comment { id: Index, comment: String, success: bool },
    Win,
    AICooldown { seconds: u64 },
    Error(ProofError),
}
#[derive(Deserialize, Serialize)]
pub enum ClientMessage {
    GetGameState,
    Add { statement: String },
    Delete { id: Index },
    Edit { id: Index, statement: String },
    Link { premise: Index, conclusion: Index },
    Unlink { premise: Index, conclusion: Index },
    ProveDirect { id: Index },
    ProveImplication { id: Index },
}

/// handles communication between client and server.
pub struct Messenger {
    pub sender: SplitSink<WebSocket, Message>,
}

impl Messenger {
    async fn send(&mut self, msg: ServerMessage) {
        let _ = self
            .sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap()))
            .await;
    }
    async fn send_cooldown(&mut self, seconds: u64) {
        let _ = self.send(ServerMessage::AICooldown { seconds }).await;
    }
    async fn send_tree(&mut self, tree: &TreeState) {
        //push game state to client(s)
        let tree_dto: TreeStateDTO = tree.as_dto();
        let _ = self.send(ServerMessage::GameState(tree_dto)).await;
    }
    async fn msg(&mut self, id: Index, comment: String, success: bool) {
        //append message to node
        let _ = self.send(ServerMessage::Comment { id, comment, success }).await;
    }
    async fn msg_win(&mut self) {
        let _ = self.send(ServerMessage::Win).await;
    }
    /* Methods to (in future) only reply to the client that triggered some command */
    async fn reply(&mut self, msg: ServerMessage) {
        self.send(msg).await;
    }
    async fn reply_tree(&mut self, tree: &TreeState) {
        let tree_dto: TreeStateDTO = tree.as_dto();
        let _ = self.reply(ServerMessage::GameState(tree_dto)).await;
    }
}

pub struct GameState {
    tree: TreeState,
    ai: AI,
    messenger: Messenger,
}

impl GameState {
    pub fn new(root_statement: String, messenger: Messenger) -> Self {
        Self {
            tree: TreeState::new(root_statement),
            ai: AI {
                cooldown_until: Instant::now(),
                max_ai_cooldown_seconds: env::var("MAX_AI_COOLDOWN_SECONDS")
                    .expect("MAX_AI_COOLDOWN_SECONDS not in env")
                    .parse::<u64>()
                    .expect("MAX_AI_COOLDOWN_SECONDS must be a number."),
            },
            messenger,
        }
    }

    /// handle incoming messages from client(s). Returns a message to be sent only to the sender.
    pub async fn on_incoming_message(&mut self, incoming_message: ClientMessage) {
        //remember if we want to push the tree (as long as no error happens)
        let state_change = &mut match incoming_message {
            Add { .. } | Delete { .. } | Link { .. } | Unlink { .. } | Edit { .. } => true,
            _ => false,
        };

        //handle incoming messages from client(s)
        let result: Result<(), ProofError> = match incoming_message {
            Add { statement } => {
                let id = self.tree.add_node(statement);
                self.messenger.reply(ServerMessage::NewNodeId(id)).await;
                Ok(())
            }
            GetGameState => {
                self.messenger.reply_tree(&self.tree).await;
                Ok(())
            }
            Link { premise, conclusion } => self.tree.link(conclusion, premise),
            Unlink { premise, conclusion } => self.tree.unlink(conclusion, premise),
            Delete { id } => self.tree.remove_node(id),
            Edit { id, statement } => self.tree.change_node_statement(id, statement),
            ProveDirect { id } => self.prove_direct(id, state_change).await,
            ProveImplication { id } => self.prove_implication(id, state_change).await,
        };
        if let Err(e) = result {
            self.messenger.reply(ServerMessage::Error(e)).await;
        } else {
            if *state_change {
                self.messenger.send_tree(&self.tree).await;
                if self.tree.proof_complete() {
                    self.messenger.msg_win().await;
                }
            }
        }
    }

    pub async fn prove_direct(&mut self, id: Index, tree_changed: &mut bool) -> Result<(), ProofError> {
        self.messenger.send_cooldown(self.ai.max_ai_cooldown_seconds).await;
        match self.ai.check_statement(self.tree.get_statement(id)?).await {
            Ok(explanation) => {
                self.tree.set_directly_proven(id);
                *tree_changed = true;
                self.messenger.msg(id, explanation, true).await;
            }
            Err(explanation) => {
                self.messenger.msg(id, explanation, false).await;
            }
        }
        Ok(())
    }
    pub async fn prove_implication(&mut self, id: Index, tree_changed: &mut bool) -> Result<(), ProofError> {
        self.messenger.send_cooldown(self.ai.max_ai_cooldown_seconds).await;
        let conclusion = self.tree.get_statement(id)?;
        let premises = self.tree.get_premises(id)?;
        if premises.len() == 0 {
            self.messenger
                .msg(
                    id,
                    "You need to add at least one premise to prove an implication.".to_string(),
                    false,
                )
                .await;
            return Ok(());
        }
        match self.ai.check_implication(&premises, conclusion).await {
            Ok(explanation) => {
                self.tree.set_implied(id);
                *tree_changed = true;
                self.messenger.msg(id, explanation, true).await;
            }
            Err(explanation) => {
                self.messenger.msg(id, explanation, false).await;
            }
        }
        Ok(())
    }
}

struct AI {
    cooldown_until: Instant,
    max_ai_cooldown_seconds: u64,
}

const SYSTEM_MESSAGE_DIRECT: &str = "The User will give you a statement. Begin your answer with '[TRUE]', if you consider the statement to be objectively correct. If not, begin your answer with '[FALSE]' and then provide an explanation.\n
Important:\n
- Always use this format for your answer.\n
- Explain very briefly but exact, in one sentence.";
const SYSTEM_MESSAGE_IMPLICATION: &str = "The User will give you a list of assumptions and a statement. Begin your answer with '[TRUE]', if you consider the statement to be a logical consequence of the assumptions. If not, begin your answer with '[FALSE]' and tell why (f.ex. which assumptions are missing).\n
Important:\n
- Always use this format for your answer.\n
- Explain very briefly but exact, in one sentence.";
const ÎMPLICATION_PRE: &str = "Assume, the following assumptions would all be true:\n";
const IMPLICATION_MID: &str = "Now, under this assumption, evaluate if the following statement is a consequence:\n";
impl AI {
    fn check_cooldown(&mut self) -> Result<(), String> {
        if self.cooldown_until > Instant::now() {
            return Err(format!(
                "AI is on cooldown for the next {} second(s).",
                self.cooldown_until.duration_since(Instant::now()).as_secs()
            ));
        }
        //set cooldown for the next 15 seconds.
        self.cooldown_until = Instant::now() + std::time::Duration::from_secs(self.max_ai_cooldown_seconds);

        Ok(())
    }
    fn parse_ai_result(&mut self, ai_result: Result<String, reqwest::Error>) -> Result<String, String> {
        let result = match ai_result {
            Ok(msg) => msg,
            Err(e) => {
                return Err(format!(
                    "Server: Internal Error while consulting AI - maybe no more money? :( - {:?}",
                    e
                ));
            }
        };
        if result.starts_with("[TRUE]") {
            //todo: reset cooldown if true, and emit message to client
            //self.cooldown_until = Instant::now();
            Ok(result[6..].to_string())
        } else if result.starts_with("[FALSE]") {
            Err(result[7..].to_string())
        } else {
            Err(result.to_string())
        }
    }
    async fn check_statement(&mut self, statement: &str) -> Result<String, String> {
        self.check_cooldown()?;

        self.parse_ai_result(openai::request(SYSTEM_MESSAGE_DIRECT.to_string(), statement.to_string()).await)
    }
    async fn check_implication(&mut self, premises: &[&str], conclusion: &str) -> Result<String, String> {
        self.check_cooldown()?;
        let user_message = format!(
            "{}{}{}{}",
            ÎMPLICATION_PRE,
            premises.join("\n"),
            IMPLICATION_MID,
            conclusion
        );
        self.parse_ai_result(openai::request(SYSTEM_MESSAGE_IMPLICATION.to_string(), user_message).await)
    }
}
