use bsim_engine::types::{ID, PIN};

#[derive(Debug)]
pub enum UpdateOps {
    SetState(ID, bool),
    Connect(ID, (ID, PIN)),
    Disconnect(ID, (ID, PIN)),
    Remove(ID),
}

pub struct SyncState {
    pub synced: bool,
}
