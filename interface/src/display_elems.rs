use std::collections::{HashMap, HashSet};

use bsim_engine::types::{ID, PIN};
use egui::{Color32, Context, Id, Pos2, Vec2};

use crate::{
    consts::{DEFAULT_SCALE, GRID_UNIT_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH},
    utils::{CompIO, EmitterReceiverPair},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnitArea {
    VACANT,
    Unvisitable,
}

pub type Screen = [[UnitArea; WINDOW_WIDTH as usize]; WINDOW_HEIGHT as usize];

pub struct Wire {
    pub emitter: (egui::Id, CompIO),
    pub pts: Vec<Pos2>,
    pub width: f32,
}

// a component is built of various gates, and the I/O pins
// of the component are mapped to the individual gates by
// some internal logic of the component

// when used for output, `pin` is useless

pub struct CompDisplayData {
    // top left corner of the component
    // coords of the top left block on the grid where
    // this component begins, not the px values
    pub id: egui::Id,
    pub logical_loc: Pos2,
    pub name: String,
    pub label: String,
    // these are relative to loc but are in px
    pub outputs_rel: Vec<CompIO>,
    // pub input_locs_rel: Vec<Vec2>,
    pub inputs_rel: Vec<CompIO>,
    pub is_clocked: bool,
    pub is_module: bool,
    pub scale: f32,
    // number of grid blocks it extends to
    pub size: Vec2,
    pub state_indicator_ref: Option<ID>, // the color of this module (red/green) is decided by this ID
    pub contents: HashSet<ID>,
}

pub struct DisplayState {
    pub display_data: HashMap<egui::Id, CompDisplayData>,
    pub screen: Screen,
    pub wires: HashMap<EmitterReceiverPair, Wire>,
    pub ctx: Context,
    pub clk_t: u64,
    pub module_expr_input: String,
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
            module_expr_input: String::new(),
        };
        // pre-add clock
        let size: Vec2 = (8.0, 4.0).into();
        let id = egui::Id::new(clk_id);
        let mut contents = HashSet::new();
        contents.insert(clk_id);
        this.display_data.insert(
            id,
            CompDisplayData {
                id,
                logical_loc: (1., 18.0).into(),
                outputs_rel: vec![CompIO {
                    id: clk_id,
                    pin: 1,
                    loc_rel: (size.x, size.y / 2.0).into(),
                }],
                inputs_rel: vec![],
                is_clocked: false,
                scale: DEFAULT_SCALE,
                size,
                name: "CLK".into(),
                label: "CLK".into(),
                is_module: false,
                state_indicator_ref: Some(clk_id),
                contents,
            },
        );
        this
    }
}
