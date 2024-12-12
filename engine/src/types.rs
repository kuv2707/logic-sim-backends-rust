use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
};

use crate::components::Gate;

pub type BinaryLogicReducer = fn(&Vec<bool>) -> bool;
pub type ComponentActor = fn(&mut Gate, &HashMap<i32, RefCell<Gate>>, &mut VecDeque<ID>);

pub type ID = i32;
pub const COMPONENT_NOT_DEFINED: ID = -1;
pub const UNASSIGNED: ID = -2;
