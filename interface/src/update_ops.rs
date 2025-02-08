use bsim_engine::types::{ID, PIN};

use crate::{display_elems::{CompDisplayData}, utils::EmitterReceiverPair};

pub enum CircuitUpdateOps {
    SetState(ID, bool),
    Connect(EmitterReceiverPair),
    Disconnect(EmitterReceiverPair),
    Remove(ID),
    SetComponentLabel(ID, String, String),
}
#[derive(PartialEq, Eq)]
pub enum SyncState {
    Synced,
    Error(String),
    NotSynced,
}

impl SyncState {
    pub fn is_synced(&self) -> bool {
        matches!(self, SyncState::Synced)
    }
    pub fn is_error(&self) -> bool {
        matches!(self, SyncState::Error(_))
    }
    pub fn error_msg(&self) -> &str {
        if let SyncState::Error(msg) = self {
            msg
        } else {
            ""
        }
    }
}


pub enum UiUpdateOps {
    Dragged,
    AddComponent(CompDisplayData),
    RemoveComponent(egui::Id),
    Connect(EmitterReceiverPair),
    Disconnect(EmitterReceiverPair),
    Select(egui::Id),
}

pub enum StateUpdateOps {
    UiOp(UiUpdateOps),
    CktOp(CircuitUpdateOps),
}