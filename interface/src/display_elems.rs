use std::collections::{HashMap, HashSet};

use bsim_engine::types::{ID, PIN};
use egui::{Context, Id, Pos2, Rect, Vec2};

use crate::{
    consts::{DEFAULT_SCALE, GRID_UNIT_SIZE},
    top_bar::TopBarOption,
    update_ops::SyncState,
    utils::{CompIO, EmitterReceiverPair},
};

pub type Weight = usize;
pub const OCCUPIED_WEIGHT: Weight = 999999;

pub struct Screen {
    width: usize,
    height: usize,
    pub weights: Vec<Vec<Weight>>,
}
impl Screen {
    pub fn new(width: usize, height: usize) -> Self {
        let weights = vec![vec![0; width]; height];
        Screen {
            weights,
            width,
            height,
        }
    }
    pub fn at(&self, x: usize, y: usize) -> Option<&Weight> {
        if x < self.width && y < self.height {
            Some(&self.weights[y][x])
        } else {
            None
        }
    }

    pub fn logical_width(&self) -> usize {
        self.width
    }

    pub fn logical_height(&self) -> usize {
        self.height
    }
    pub fn resize_to(&mut self, r: Rect) {
        let neww = r.width() as usize;
        let newh = r.height() as usize;
        self.weights.resize_with(newh, || vec![0; neww]);
        for row in &mut self.weights {
            row.resize(neww, 0);
        }
        self.width = neww;
        self.height = newh;
    }
}

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
    pub scale: f32,
    // number of grid blocks it extends to
    pub size: Vec2,
    pub state_indicator_ref: Option<ID>, // the color of this module (red/green) is decided by this ID
    pub contents: HashSet<ID>,
}

pub struct DisplayState {
    pub comp_display_data: HashMap<egui::Id, CompDisplayData>,
    pub screen: Screen,
    pub wires: HashMap<EmitterReceiverPair, Wire>,
    pub ctx: Context,
    pub module_expr_input: String,
    pub sync: SyncState,
    pub render_cnt: u64,
    pub clk_t: u64,
    pub top_bar_opts: Vec<TopBarOption>,
    pub connect_candidate: Option<(egui::Id, CompIO)>,
}

impl DisplayState {
    pub fn init_display_state(clk_id: ID, ctx: Context) -> Self {
        let mut this = Self {
            comp_display_data: HashMap::new(),
            screen: Screen::new(140, 80),
            wires: HashMap::new(),
            ctx,
            module_expr_input: String::new(),
            sync: SyncState::Synced,
            render_cnt: 0,
            clk_t: 60,
            top_bar_opts: Vec::new(),
            connect_candidate: None,
        };
        // pre-add clock
        let size: Vec2 = (8.0, 4.0).into();
        let id = egui::Id::new(clk_id);
        let mut contents = HashSet::new();
        contents.insert(clk_id);
        this.comp_display_data.insert(
            id,
            CompDisplayData {
                id,
                logical_loc: (1., 18.0).into(),
                outputs_rel: vec![CompIO {
                    id: clk_id,
                    pin: 1,
                    loc_rel: (size.x, size.y / 2.0).into(),
                    label: String::new(),
                }],
                inputs_rel: vec![],
                is_clocked: false,
                scale: DEFAULT_SCALE,
                size,
                name: "CLK".into(),
                label: "CLK".into(),
                state_indicator_ref: Some(clk_id),
                contents,
            },
        );
        this
    }
}
