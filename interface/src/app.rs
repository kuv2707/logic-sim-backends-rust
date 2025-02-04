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
    component_ui::{add_pin_btn, paint_component, PIN_BTN_SIZE},
    consts::{DEFAULT_SCALE, GREEN_COL, GRID_UNIT_SIZE, RED_COL},
    display_elems::{CompDisplayData, DisplayState, Screen, UnitArea, Wire},
    logic_units::{get_logic_unit, ModuleCreationData},
    state_handlers::{ckt_communicate, toggle_clock, ui_update},
    update_ops::{CircuitUpdateOps, SyncState, UiUpdateOps},
    utils::{CompIO, EmitterReceiverPair},
};

pub struct SimulatorUI {
    ckt: BCircuit,
    pub display_state: DisplayState,
    pub ckt_evts: VecDeque<CircuitUpdateOps>,
    pub ui_evts: VecDeque<UiUpdateOps>, // todo: shift to display_state
    pub from: Option<(egui::Id, CompIO)>, //todo: shift to display_state
    pub available_comp_defns: Vec<(String, usize)>,
}

impl SimulatorUI {
    pub fn new(ctx: Context) -> Self {
        let mut ckt = BCircuit::new();
        ckt.compile();
        ckt.power_on();
        let clk_id = ckt.add_input("CLK", false);
        ckt.clock(clk_id);

        let mut available_comp_defns: Vec<(String, usize)> = ckt
            .component_definitions
            .values()
            .map(|v| (v.name.clone(), v.default_inputs as usize))
            .collect();
        available_comp_defns.sort();

        let sync_state = SyncState::Synced;

        let display_state = DisplayState::init_display_state(clk_id, ctx);

        let sim = Self {
            ckt,
            display_state,
            ckt_evts: VecDeque::new(),
            ui_evts: VecDeque::new(),
            from: None,
            available_comp_defns,
        };
        sim
    }
    fn ui(&mut self, ui: &mut Ui) {
        draw_bg(ui, &self.display_state.screen);
        let display_data = &self.display_state.display_data;
        let ui_sender = &mut self.ui_evts;
        ui.horizontal(|ui| {
            for (i, (name, n_inp)) in self.available_comp_defns.iter().enumerate() {
                let button = egui::Button::new(name).min_size(Vec2::new(80.0, 40.0));
                let response = button.ui(ui);
                if response.clicked() {
                    let id = match name.as_str() {
                        "Input" => self.ckt.add_input("", false),
                        _ => self.ckt.add_component(name, "").unwrap(),
                    };
                    let gate = self.ckt.get_component(&id).unwrap().borrow();
                    let loc = egui::pos2(40.0 + 80.0 * i as f32, 100.0) / GRID_UNIT_SIZE;
                    let size: Vec2 = (8.0, 8.0).into();
                    let spc = size.y / (n_inp + 1) as f32;
                    let inputs_rel = (0..*n_inp + 1)
                        .map(|i| {
                            CompIO {
                                id,
                                pin: i,
                                loc_rel: if i == 0 {
                                    // clock
                                    vec2(size.x / 2.0, size.y)
                                } else {
                                    vec2(0.0, spc * i as f32)
                                },
                                label: String::new(),
                            }
                        })
                        .collect();
                    let mut contents = HashSet::new();
                    contents.insert(id);
                    let data = CompDisplayData {
                        id: egui::Id::new(id),
                        logical_loc: loc,
                        name: name.into(),
                        label: gate.label.clone(),
                        outputs_rel: vec![CompIO {
                            id,
                            pin: 1,
                            loc_rel: vec2(size.x, size.y / 2.0),
                            label: String::new(),
                        }],
                        inputs_rel,
                        is_clocked: gate.clock_manager.is_some(),
                        scale: DEFAULT_SCALE,
                        size,
                        is_module: false,
                        state_indicator_ref: Some(id),
                        contents,
                    };

                    send_event(ui_sender, UiUpdateOps::AddComponent(data));
                }
            }

            let response = ui.text_edit_singleline(&mut self.display_state.module_expr_input);
            if response.lost_focus() {
                match get_disp_data_from_modctx(get_logic_unit(
                    &mut self.ckt,
                    &self.display_state.module_expr_input,
                )) {
                    Ok(data) => {
                        send_event(ui_sender, UiUpdateOps::AddComponent(data));
                    }
                    Err(e) => {
                        // todo: show msg that expr was bad
                    }
                }
                self.display_state.module_expr_input.clear();
            }
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
        // drawing components and handling evts
        let mut ckt_evts = Vec::new();
        let mut ui_evts = Vec::new();
        for (id, disp_data) in self.display_state.display_data.iter_mut() {
            paint_component(
                disp_data,
                ui,
                &self.ckt,
                &mut self.from,
                &mut ckt_evts,
                &mut ui_evts,
            );
        }
        for evt in ckt_evts {
            send_event(&mut self.ckt_evts, evt);
        }
        for evt in ui_evts {
            send_event(&mut self.ui_evts, evt);
        }

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
}

pub fn send_event<T>(sender: &mut VecDeque<T>, evt: T) {
    sender.push_back(evt);
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

fn get_disp_data_from_modctx(
    res: Result<ModuleCreationData, String>,
) -> Result<CompDisplayData, String> {
    match res {
        Ok(ctx) => {
            let ipins = ctx.inputs;
            let opins = ctx.outputs;
            let contents = ctx.contents;
            let size: Vec2 = (8.0, 2.0 * (max(ipins.len(), opins.len())) as f32).into();

            let i_gap = size.y / (ipins.len() + 1) as f32;
            let o_gap = size.y / (opins.len() + 1) as f32;
            let data = CompDisplayData {
                id: egui::Id::new(contents.iter().next().unwrap()),
                logical_loc: (7.0, 7.0).into(),
                name: "module".into(),
                label: "".into(),
                outputs_rel: opins
                    .iter()
                    .enumerate()
                    .map(|(i, id)| CompIO {
                        id: *id.1,
                        pin: 1,
                        loc_rel: vec2(size.x, o_gap * (i + 1) as f32),
                        label: id.0.to_string(),
                    })
                    .collect(),
                inputs_rel: ipins
                    .iter()
                    .enumerate()
                    .map(|(i, id)| CompIO {
                        id: *id.1,
                        pin: 1,
                        loc_rel: vec2(0.0, i_gap * (i + 1) as f32),
                        label: id.0.to_string(),
                    })
                    .collect(),
                is_clocked: true, // todo
                is_module: true,
                scale: DEFAULT_SCALE,
                size,
                state_indicator_ref: None,
                contents,
            };
            Ok(data)
        }
        Err(e) => Err(e),
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
