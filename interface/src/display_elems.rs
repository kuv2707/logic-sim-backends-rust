use std::collections::HashMap;

use bsim_engine::types::{ID, PIN};
use egui::{Color32, Pos2, Vec2};

use crate::consts::{WINDOW_HEIGHT, WINDOW_WIDTH};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnitArea {
    VACANT,
    Unvisitable,
}

pub type Screen = [[UnitArea; WINDOW_WIDTH as usize]; WINDOW_HEIGHT as usize];

pub struct Wire {
    pub pts: Vec<Pos2>,
    pub col: Color32,
    pub width: f32,
}

#[derive(Clone)]
pub struct DisplayData {
    // top left corner of the component
    // coords of the top left block on the grid where
    // this component begins, not the px values
    pub logical_loc: Pos2,
    // these are relative to loc but are in px
    pub output_loc_rel: Vec2,
    pub input_locs_rel: Vec<Vec2>,
    pub id: ID,
    pub is_clocked: bool,
    pub scale: f32,
    // number of grid blocks it extends to
    pub size: Vec2,
}

pub struct DisplayState {
    pub display_data: HashMap<ID, DisplayData>,
    pub screen: Screen,
    pub wires: HashMap<(ID, (ID, PIN)), Wire>,
}

fn make_screen() -> Screen {
    [[UnitArea::VACANT; WINDOW_WIDTH as usize]; WINDOW_HEIGHT as usize]
}

impl DisplayState {
    pub fn new() -> Self {
        Self {
            display_data: HashMap::new(),
            screen: make_screen(),
            wires: HashMap::new(),
        }
    }
}
