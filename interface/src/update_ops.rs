use bsim_engine::types::{ID, PIN};

#[derive(Debug)]
pub enum UpdateOps {
    SetState(ID, bool),
    Connect(ID, (ID, PIN)),
    Disconnect(ID, (ID, PIN)),
    Remove(ID),
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
