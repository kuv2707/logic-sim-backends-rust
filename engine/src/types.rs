use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
};

use crate::components::Gate;

// logic reducer receives a vector of inputs and current state
pub type BinaryLogicReducer = fn(&Vec<bool>, bool) -> bool;
pub type ComponentActor = fn(&mut Gate, &HashMap<i32, RefCell<Gate>>, &mut VecDeque<ID>);

pub type ID = i32;
pub type PIN = usize;
pub const NULL: ID = -1;
pub const UNASSIGNED: ID = -2;

pub const CLOCK_PIN: PIN = 0;
pub const OUTPUT_PIN: PIN = usize::MAX;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CompType {
    Combinational,
    Sequential,
    Input,
}
