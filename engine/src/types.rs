pub type BinaryLogicReducer = fn(&Vec<bool>) -> bool;

pub type ID = i32;
pub const COMPONENT_NOT_DEFINED: ID = -1;
pub const UNASSIGNED: ID = -2;
