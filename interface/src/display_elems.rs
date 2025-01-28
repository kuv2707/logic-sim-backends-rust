use std::collections::HashMap;

use bsim_engine::types::{ID, PIN};
use egui::{Color32, Context, Pos2, Vec2};

use crate::consts::{DEFAULT_SCALE, GRID_UNIT_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnitArea {
    VACANT,
    Unvisitable,
}

pub type Screen = [[UnitArea; WINDOW_WIDTH as usize]; WINDOW_HEIGHT as usize];

pub struct Wire {
    pub emitter_id: ID,
    pub pts: Vec<Pos2>,
    pub width: f32,
}

#[derive(Clone)]
pub struct DisplayData {
    // top left corner of the component
    // coords of the top left block on the grid where
    // this component begins, not the px values
    pub logical_loc: Pos2,
    pub name: String,
    pub label: String,
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
    pub ctx: Context,
    pub clk_t: u64,
}

fn make_screen() -> Screen {
    [[UnitArea::VACANT; WINDOW_WIDTH as usize]; WINDOW_HEIGHT as usize]
}

impl DisplayState {
    pub fn init_display_state(clk_id: ID, ctx: Context) -> Self {
        let mut this = Self {
            display_data: HashMap::new(),
            screen: make_screen(),
            wires: HashMap::new(),
            ctx,
            clk_t: 1000,
        };
        // pre-add clock
        let size: Vec2 = (8.0, 4.0).into();
        this.display_data.insert(
            clk_id,
            DisplayData {
                logical_loc: (1., 18.0).into(),
                output_loc_rel: (size.x, size.y/2.0).into(),
                input_locs_rel: vec![],
                id: clk_id,
                is_clocked: false,
                scale: DEFAULT_SCALE,
                size,
                name: "CLK".into(),
                label: "CLK".into(),
            },
        );
        this
    }
}
