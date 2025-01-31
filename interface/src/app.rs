use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet},
    f32,
    sync::{Arc, Mutex},
    thread,
};

use bsim_engine::{
    circuit::BCircuit,
    types::{CLOCK_PIN, ID, PIN},
};
use crossbeam::channel::{self, Sender};
use egui::{
    pos2, vec2, Align, Button, Color32, Context, FontId, Label, Layout, Painter, Pos2, Rect,
    Stroke, TextEdit, Ui, Vec2, Widget,
};

use crate::{
    component_ui::{add_pin_btn, paint_component, PIN_BTN_SIZE},
    consts::{DEFAULT_SCALE, GRID_UNIT_SIZE},
    display_elems::{CompDisplayData, DisplayState, Screen, UnitArea, Wire},
    logic_units::{get_logic_unit, ModuleCreationData},
    state_handler_threads::{ckt_communicate, toggle_clock, ui_update},
    update_ops::{CircuitUpdateOps, SyncState, UiUpdateOps},
    utils::{CompIO, EmitterReceiverPair},
};

pub struct SimulatorUI {
    ckt: Arc<Mutex<BCircuit>>,
    pub display_state: Arc<Mutex<DisplayState>>,
    pub ckt_sender: Sender<CircuitUpdateOps>,
    pub ui_sender: Sender<UiUpdateOps>,
    sync: Arc<Mutex<SyncState>>,          // todo: shift to display_state
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

        let (ckt_sender, ckt_receiver) = channel::unbounded();
        let ckt = Arc::new(Mutex::new(ckt));

        let sync_state = SyncState::Synced;
        let sync = Arc::new(Mutex::new(sync_state));

        let (ui_sender, ui_receiver) = channel::unbounded();
        let display_state = Arc::new(Mutex::new(DisplayState::init_display_state(clk_id, ctx)));

        thread::Builder::new()
            .name("ckt-module-communicate".into())
            .spawn(ckt_communicate(
                ckt_receiver,
                ckt.clone(),
                sync.clone(),
                ui_sender.clone(),
            ))
            .expect("Failed to spawn thread");
        thread::Builder::new()
            .name("ui-update".into())
            .spawn(ui_update(
                ui_receiver,
                display_state.clone(),
                ckt_sender.clone(),
            ))
            .expect("Failed to spawn thread");
        thread::Builder::new()
            .name("ckt-clock-toggle".into())
            .spawn(toggle_clock(ckt.clone(), display_state.clone(), clk_id))
            .expect("Failed to spawn thread");

        let sim = Self {
            ckt,
            display_state,
            ckt_sender,
            ui_sender,
            sync,
            from: None,
            available_comp_defns,
        };
        sim
    }
    fn ui(&mut self, ui: &mut Ui) {
        let mut ckt = self.ckt.lock().unwrap();
        let mut sync = self.sync.lock().unwrap();

        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(80, 60, 60));

        let display_state = &mut self.display_state.lock().unwrap();

        let display_data = &display_state.display_data;
        let ui_sender = &mut self.ui_sender;
        ui.horizontal(|ui| {
            for (i, (name, n_inp)) in self.available_comp_defns.iter().enumerate() {
                let button = egui::Button::new(name).min_size(Vec2::new(80.0, 40.0));
                let response = button.ui(ui);
                if response.clicked() {
                    let id = match name.as_str() {
                        "Input" => ckt.add_input("", false),
                        _ => ckt.add_component(name, "").unwrap(),
                    };
                    let gate = ckt.get_component(&id).unwrap().borrow();
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

            let response = ui.text_edit_singleline(&mut display_state.module_expr_input);
            if response.lost_focus() {
                match get_disp_data_from_modctx(get_logic_unit(
                    &mut ckt,
                    &display_state.module_expr_input,
                )) {
                    Ok(data) => {
                        send_event(ui_sender, UiUpdateOps::AddComponent(data));
                    }
                    Err(e) => {
                        // todo: show msg that expr was bad
                    }
                }
                display_state.module_expr_input.clear();
            }
        });
        // print_screen(&display_state.screen);
        self.draw_connections(&ckt, &display_state.wires, ui.painter());

        ui.style_mut().text_styles.insert(
            egui::TextStyle::Body,
            FontId::new(8.0, egui::FontFamily::Monospace),
        );
        // drawing components and handling evts
        let mut ckt_evts = Vec::new();
        let mut ui_evts = Vec::new();
        for (id, disp_data) in display_state.display_data.iter_mut() {
            paint_component(
                disp_data,
                ui,
                &ckt,
                &mut self.from,
                &mut ckt_evts,
                &mut ui_evts,
            );
        }
        for evt in ckt_evts {
            send_event(&self.ckt_sender, evt);
        }
        for evt in ui_evts {
            send_event(&self.ui_sender, evt);
        }

        ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
            let btn = Button::new(if sync.is_synced() {
                "Synced"
            } else {
                sync.error_msg()
            })
            .fill(if sync.is_synced() {
                egui::Color32::from_rgb(34, 139, 34) // Darker green
            } else {
                egui::Color32::from_rgb(139, 0, 0) // Darker red
            });
            ui.add(btn);
            ui.add(Label::new(&format!(
                "No. of ckt components -> {}",
                ckt.components().len()
            )))
        });
        if !sync.is_error() {
            *sync = SyncState::Synced;
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
                egui::Color32::from_rgb(144, 238, 144)
            } else {
                egui::Color32::from_rgb(255, 102, 102)
            };
            pt.line(
                wire.pts.iter().map(|k| *k * GRID_UNIT_SIZE).collect(),
                Stroke::new(wire.width, col),
            );
        }
    }
}

pub fn send_event<T>(sender: &Sender<T>, evt: T) {
    if let Err(err) = sender.send(evt) {
        println!("{}", err.to_string());
    }
}

impl eframe::App for SimulatorUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
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
            let size: Vec2 = (8.0, 8.0).into();

            let i_gap = size.y / (ipins.len() + 1) as f32;
            let o_gap = size.y / (opins.len() + 1) as f32;

            let data = CompDisplayData {
                id: egui::Id::new(ipins[0]),
                logical_loc: (7.0, 7.0).into(),
                name: "module".into(),
                label: "".into(),
                outputs_rel: opins
                    .iter()
                    .enumerate()
                    .map(|(i, id)| CompIO {
                        id: *id,
                        pin: 1,
                        loc_rel: vec2(size.x, o_gap * (i + 1) as f32),
                    })
                    .collect(),
                inputs_rel: ipins
                    .iter()
                    .enumerate()
                    .map(|(i, id)| CompIO {
                        id: *id,
                        pin: 1,
                        loc_rel: vec2(0.0, i_gap * (i + 1) as f32),
                    })
                    .collect(),
                is_clocked: false, // todo
                is_module: true,
                scale: DEFAULT_SCALE,
                size,
                state_indicator_ref: Some(opins[0]),
                contents,
            };
            Ok(data)
        }
        Err(e) => Err(e),
    }
}

fn print_screen(s: &Screen) {
    for row in s {
        for unit in row {
            match unit {
                UnitArea::VACANT => print!(" "),
                UnitArea::Unvisitable => print!("#"),
            }
        }
        println!();
    }
}
