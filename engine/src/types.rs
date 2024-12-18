use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
};

use crate::components::Gate;

// logic reducer receives a vector of inputs and current state
pub type BinaryLogicReducer = fn(&Vec<bool>, bool) -> bool;
pub type ComponentActor = fn(&mut Gate, &HashMap<i32, RefCell<Gate>>, &mut VecDeque<ID>);

pub type ID = i32;
pub const COMPONENT_NOT_DEFINED: ID = -1;
pub const UNASSIGNED: ID = -2;

pub const CLOCK_PIN: u16 = u16::MAX;

#[derive(Clone, PartialEq)]
pub enum CompType {
    Combinational, Sequential, Input
}