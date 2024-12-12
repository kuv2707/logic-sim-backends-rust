use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
};

use crate::components::Component;

pub type BinaryLogicReducer = fn(&Vec<bool>) -> bool;
pub type ComponentActor = fn(&mut Component, &HashMap<i32, RefCell<Component>>, &mut VecDeque<ID>);

pub type ID = i32;
pub const COMPONENT_NOT_DEFINED: ID = -1;
pub const UNASSIGNED: ID = -2;
