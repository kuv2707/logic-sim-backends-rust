use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet, VecDeque},
    f32,
    ops::RangeInclusive,
    sync::{Arc, Mutex},
    thread,
};

use bsim_engine::{
    circuit::BCircuit,
    types::{CLOCK_PIN, ID, PIN},
};
use egui::{
    pos2, vec2, Align, Button, Color32, Context, FontId, Label, Layout, Painter, Pos2, Rect,
    Slider, Stroke, TextEdit, Ui, Vec2, Widget,
};

use crate::{
    component_ui::{add_pin_btn, paint_component, paint_components, PIN_BTN_SIZE},
    consts::{DEFAULT_SCALE, GREEN_COL, GRID_UNIT_SIZE, RED_COL},
    display_elems::{CompDisplayData, DisplayState, Screen, UnitArea, Wire},
    logic_units::{get_logic_unit, ModuleCreationData},
    state_handlers::{ckt_communicate, toggle_clock, ui_update},
    top_bar::{Modulator, TopBarOption},
    update_ops::{CircuitUpdateOps, StateUpdateOps, SyncState, UiUpdateOps},
    utils::{CompIO, EmitterReceiverPair},
};

macro_rules! receive_evts {
    ($a:ident, $b:expr) => {
        for k in $b {
            match k {
                StateUpdateOps::UiOp(ui_update_ops) => $a.ui_evts.push_back(ui_update_ops),
                StateUpdateOps::CktOp(circuit_update_ops) => {
                    $a.ckt_evts.push_back(circuit_update_ops)
                }
            };
        }
    };
}

pub struct SimulatorUI {
    pub ckt: BCircuit,
    pub display_state: DisplayState,
    pub ckt_evts: VecDeque<CircuitUpdateOps>,
    pub ui_evts: VecDeque<UiUpdateOps>, // todo: shift to display_state
}

impl SimulatorUI {
    pub fn new(ctx: Context) -> Self {
        let mut ckt = BCircuit::new();
        ckt.compile();
        ckt.power_on();
        let clk_id = ckt.add_input("CLK", false);
        ckt.clock(clk_id);

        let mut display_state = DisplayState::init_display_state(clk_id, ctx);

        let mut available_comp_names: Vec<String> = ckt
            .component_definitions
            .values()
            .map(|v| v.name.clone())
            .collect();
        available_comp_names.sort();
        for name in available_comp_names {
            display_state
                .top_bar_opts
                .push(TopBarOption::AddComponent { name });
        }
        display_state
            .top_bar_opts
            .push(TopBarOption::AddModuleFromText {
                typed_text: String::new(),
                modulator: Modulator::Expressions,
            });
        display_state
            .top_bar_opts
            .push(TopBarOption::AddModuleFromText {
                typed_text: "3x8".into(),
                modulator: Modulator::Decoder,
            });
        let sim = Self {
            ckt,
            display_state,
            ckt_evts: VecDeque::new(),
            ui_evts: VecDeque::new(),
        };
        sim
    }
    fn ui(&mut self, ui: &mut Ui) {
        draw_bg(ui, &self.display_state.screen);
        ui.horizontal(|ui| {
            for opt in &mut self.display_state.top_bar_opts {
                receive_evts!(self, opt.render(&mut self.ckt, ui));
            }

            // todo: move to settings
            let clk_freq =
                Slider::new(&mut self.display_state.clk_t, RangeInclusive::new(10, 1000));
            let r = ui.add(clk_freq);
            r.on_hover_text(format!(
                "Clock toggles every {} frames.",
                self.display_state.clk_t
            ))
        });
        // print_screen(&display_state.screen);
        self.draw_connections(&self.ckt, &self.display_state.wires, ui.painter());

        ui.style_mut().text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(8.0, egui::FontFamily::Monospace),
        );

        receive_evts!(
            self,
            paint_components(&mut self.display_state, &self.ckt, ui)
        );

        ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
            let btn = Button::new(if self.display_state.sync.is_synced() {
                "Synced"
            } else {
                self.display_state.sync.error_msg()
            })
            .fill(if self.display_state.sync.is_synced() {
                GREEN_COL
            } else {
                RED_COL
            });
            ui.add(btn);
            ui.add(Label::new(&format!(
                "No. of ckt components -> {}",
                self.ckt.components().len()
            )))
        });
        if !self.display_state.sync.is_error() {
            self.display_state.sync = SyncState::Synced;
        }
    }
    fn draw_connections(
        &self,
        ckt: &BCircuit,
        wires: &HashMap<EmitterReceiverPair, Wire>,
        pt: &Painter,
    ) {
        for wire in wires.values() {
            let col = if ckt
                .components()
                .get(&wire.emitter.1.id)
                .unwrap()
                .borrow()
                .state
            {
                GREEN_COL
            } else {
                RED_COL
            };
            pt.line(
                wire.pts.iter().map(|k| *k * GRID_UNIT_SIZE).collect(),
                Stroke::new(wire.width, col),
            );
        }
    }
    pub fn send_event(&mut self, evt: StateUpdateOps) {
        match evt {
            StateUpdateOps::UiOp(ui_update_ops) => self.ui_evts.push_back(ui_update_ops),
            StateUpdateOps::CktOp(circuit_update_ops) => {
                self.ckt_evts.push_back(circuit_update_ops)
            }
        }
    }
}

impl eframe::App for SimulatorUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
            ckt_communicate(
                &mut self.ckt_evts,
                &mut self.ckt,
                &mut self.display_state.sync,
                &mut self.ui_evts,
            );
            ui_update(
                &mut self.ui_evts,
                &mut self.display_state,
                &mut self.ckt_evts,
            );
            self.display_state.render_cnt += 1;
            // todo
            toggle_clock(&mut self.ckt, &mut self.display_state);
            self.display_state.ctx.request_repaint();
        });
    }
}

fn draw_bg(ui: &mut Ui, s: &Screen) {
    let p = ui.painter();
    p.rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(34, 37, 42));
    let stroke = Stroke::new(1.0, Color32::from_rgb(52, 56, 65));
    for y in 0..s.len() {
        p.line(
            vec![(0, y), (s[0].len(), y)]
                .iter()
                .map(|v| pos2(v.0 as f32, v.1 as f32) * GRID_UNIT_SIZE)
                .collect(),
            stroke,
        );
    }
    for x in 0..s[0].len() {
        p.line(
            vec![(x, 0), (x, s.len())]
                .iter()
                .map(|v| pos2(v.0 as f32, v.1 as f32) * GRID_UNIT_SIZE)
                .collect(),
            stroke,
        );
    }
}

fn print_screen(s: &Screen) {
    for row in s {
        for unit in row {
            print!("{}", if *unit == 0 { " " } else { "#" });
        }
        println!();
    }
}
