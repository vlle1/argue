use std::fmt::Display;

use generational_arena::{Arena, Index};
use serde::Serialize;

use super::{StatementDTO, TreeStateDTO};

#[derive(Debug, Serialize)]
pub enum ProofError {
    NoSuchNode(Index),
    RemoveRoot,
    AddExistingLink { child: Index, parent: Index },
    RemoveNonExistentLink { child: Index, parent: Index },
}

impl Display for ProofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProofError::NoSuchNode(i) => write!(f, "No node with index {:?}.", i),
            ProofError::RemoveRoot => write!(f, "Tried to remove the root node."),
            ProofError::AddExistingLink { child, parent } => write!(
                f,
                "Tried to add an existing link from {:?} to {:?}.",
                child, parent
            ),
            ProofError::RemoveNonExistentLink { child, parent } => write!(
                f,
                "Tried to remove a non-existent link from {:?} to {:?}.",
                child, parent
            ),
        }
    }
}

struct StatementNode {
    statement: String,
    children: Vec<Index>,
    parents: Vec<Index>,
    state: ProofState,
}

impl StatementNode {
    fn new(statement: String) -> Self {
        Self {
            statement,
            children: Vec::new(),
            parents: Vec::new(),
            state: ProofState::None,
        }
    }
    fn is_proven(&self) -> bool {
        self.state.is_proven()
    }
    fn is_implied(&self) -> bool {
        self.state.is_implied()
    }
}

#[derive(Serialize, Clone)]
pub enum ProofState {
    DirectlyProven,
    None,
    ImpliedUnproven, // gpt accepts that it is a consequence
    ImpliedProven,
}
impl ProofState {
    fn is_proven(&self) -> bool {
        match self {
            ProofState::DirectlyProven => true,
            ProofState::ImpliedProven => true,
            _ => false,
        }
    }
    fn is_implied(&self) -> bool {
        match self {
            ProofState::ImpliedProven => true,
            ProofState::ImpliedUnproven => true,
            _ => false,
        }
    }
}

pub struct TreeState {
    arena: Arena<StatementNode>,
    root: Index,
}

impl TreeState {
    pub fn new(root_statement: String) -> Self {
        let root = StatementNode::new(root_statement);
        let mut arena = Arena::new();
        let root_id = arena.insert(root);
        Self {
            arena,
            root: root_id,
        }
    }
    pub fn as_dto(&self) -> TreeStateDTO {
        let mut statements = Vec::<StatementDTO>::new();
        for (id, node) in self.arena.iter() {
            statements.push(StatementDTO {
                id,
                statement: node.statement.clone(),
                state: node.state.clone(),
                parents: node.parents.clone(),
                children: node.children.clone(),
            });
        }
        TreeStateDTO {
            statements,
            root: self.root,
        }
    }
    pub fn proof_complete(&self) -> bool {
        self.is_proven(self.root).expect("Root must exist.")
    }
    pub fn is_proven(&self, id: Index) -> Result<bool, ProofError> {
        let n = self.get_node(id)?;
        Ok(n.is_proven())
    }
    pub fn get_statement(&self, id: Index) -> Result<&str, ProofError> {
        Ok(&self.get_node(id)?.statement)
    }
    pub fn get_premises(&self, id: Index) -> Result<Vec<&str>, ProofError> {
        let node = self.get_node(id)?;
        Ok(node
            .children
            .iter()
            .map(|&child| self.get_statement(child).unwrap())
            .collect())
    }
    pub fn add_node(&mut self, statement: String) -> Index {
        let node = StatementNode::new(statement);
        self.arena.insert(node)
    }
    /// remove any node. affects all ancestors.
    pub fn remove_node(&mut self, id: Index) -> Result<(), ProofError> {
        if id == self.root {
            return Err(ProofError::RemoveRoot);
        }
        let node = self.arena.remove(id).ok_or(ProofError::NoSuchNode(id))?;
        for parent_id in node.parents {
            let _ = self.unlink(parent_id, id);
        }
        for child_id in node.children {
            let _ = self.unlink(id, child_id);
        }
        Ok(())
    }
    /// Change statement of a node. Does affect proof state.
    pub fn change_node_statement(
        &mut self,
        id: Index,
        new_statement: String,
    ) -> Result<(), ProofError> {
        let node = self.get_node_mut(id)?;
        node.statement = new_statement;
        self.set_proof_state(id, ProofState::None);
        Ok(())
    }
    /// Create implication-link. Affects parent state.
    pub fn link(&mut self, parent_id: Index, child_id: Index) -> Result<(), ProofError> {
        if parent_id == child_id {
            let node = self.get_node_mut(parent_id)?;
            node.parents.push(child_id);
            node.children.push(parent_id);
            return Ok(());
        }
        let (parent, child) = self.get2_node_mut(parent_id, child_id)?;
        if parent.children.contains(&child_id) {
            return Err(ProofError::AddExistingLink {
                parent: parent_id,
                child: child_id,
            });
        }
        parent.children.push(child_id);
        child.parents.push(parent_id);
        if parent.is_implied() {
            // implication stays in place, but truth value might change.
            self.on_child_change(parent_id);
        }
        Ok(())
    }
    /// Remove implication-link. Affects parent state.
    pub fn unlink(&mut self, parent_id: Index, child_id: Index) -> Result<(), ProofError> {
        if parent_id == child_id {
            let node = self.get_node_mut(parent_id)?;
            node.parents.retain(|&x| x != child_id);
            node.children.retain(|&x| x != parent_id);
            return Ok(());
        }
        let (parent, child) = self.get2_node_mut(parent_id, child_id)?;
        if !parent.children.contains(&child_id) {
            return Err(ProofError::RemoveNonExistentLink {
                parent: parent_id,
                child: child_id,
            });
        }
        parent.children.retain(|&x| x != child_id);
        child.parents.retain(|&x| x != parent_id);
        if parent.is_implied() {
            self.set_proof_state(parent_id, ProofState::None);
        }
        Ok(())
    }
    /// AI accepts a statement by itself
    pub fn set_directly_proven(&mut self, id: Index) {
        self.set_proof_state(id, ProofState::DirectlyProven)
    }
    /// AI accepts a statement as a consequence its children
    pub fn set_implied(&mut self, id: Index) {
        let new = {
            if self.check_if_all_children_true(id) {
                ProofState::ImpliedProven
            } else {
                ProofState::ImpliedUnproven
            }
        };
        self.set_proof_state(id, new);
    }
    fn get_node(&self, id: Index) -> Result<&StatementNode, ProofError> {
        self.arena.get(id).ok_or(ProofError::NoSuchNode(id))
    }
    fn get_node_mut(&mut self, id: Index) -> Result<&mut StatementNode, ProofError> {
        self.arena.get_mut(id).ok_or(ProofError::NoSuchNode(id))
    }
    fn get2_node_mut(
        &mut self,
        id1: Index,
        id2: Index,
    ) -> Result<(&mut StatementNode, &mut StatementNode), ProofError> {
        match self.arena.get2_mut(id1, id2) {
            (Some(a), Some(b)) => Ok((a, b)),
            (None, _) => Err(ProofError::NoSuchNode(id1)),
            _ => Err(ProofError::NoSuchNode(id2)),
        }
    }
    /// trickle up the proof state.
    fn set_proof_state(&mut self, id: Index, new_state: ProofState) {
        let node = self.get_node_mut(id).unwrap();
        let old_truth = node.is_proven();
        node.state = new_state;
        let new_truth = node.is_proven();
        if old_truth != new_truth {
            let parents = &node.parents.clone();
            for &parent in parents {
                self.on_child_change(parent);
            }
        }
    }

    /// recheck this node
    fn on_child_change(&mut self, id: Index) {
        let node = self.get_node(id).unwrap();
        if node.is_implied() {
            let new = {
                if self.check_if_all_children_true(id) {
                    ProofState::ImpliedProven
                } else {
                    ProofState::ImpliedUnproven
                }
            };
            self.set_proof_state(id, new);
        }
    }

    fn check_if_all_children_true(&mut self, id: Index) -> bool {
        let parent = self.get_node(id).unwrap();
        // check all children
        for &child_id in parent.children.iter() {
            let child = self.get_node(child_id).unwrap();
            if !child.is_proven() {
                return false;
            }
        }
        true
    }
}
