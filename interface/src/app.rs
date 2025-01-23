use std::{
    cmp::{max, min},
    collections::HashMap,
    f32,
    sync::{Arc, Mutex},
    thread,
};

use bsim_engine::{
    circuit::BCircuit,
    types::{CLOCK_PIN, ID, PIN},
};
use crossbeam::channel::{self, Sender};
use egui::{pos2, vec2, Button, Color32, Layout, Painter, Pos2, Stroke, Ui, Vec2, Widget};

use crate::{
    component_ui::{paint_component, PIN_BTN_SIZE},
    consts::{DEFAULT_SCALE, GRID_UNIT_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH},
    display_elems::{DisplayData, DisplayState, Screen, UnitArea, Wire},
    path_find::a_star_get_pts,
    state_handler_threads::{ckt_communicate, ui_update},
    update_ops::{CircuitUpdateOps, SyncState, UiUpdateOps},
};

pub struct SimulatorUI {
    ckt: Arc<Mutex<BCircuit>>,
    pub display_state: Arc<Mutex<DisplayState>>,
    pub ckt_sender: Sender<CircuitUpdateOps>,
    pub ui_sender: Sender<UiUpdateOps>,
    sync: Arc<Mutex<SyncState>>,
    pub from: Option<ID>,
    pub available_comp_defns: Vec<(String, usize)>,
}

impl SimulatorUI {
    pub fn new() -> Self {
        let mut ckt = BCircuit::new();
        ckt.compile();
        ckt.power_on();
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
        let display_state = Arc::new(Mutex::new(DisplayState::new()));

        thread::spawn(ckt_communicate(
            ckt_receiver,
            ckt.clone(),
            sync.clone(),
            ui_sender.clone(),
        ));
        thread::spawn(ui_update(ui_receiver, display_state.clone()));

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
                        _ => ckt.add_component(name, "A").unwrap(),
                    };
                    let gate = ckt.get_component(&id).unwrap().borrow();
                    let spc = 10.0 / (n_inp + 1) as f32;
                    let loc = egui::pos2(40.0 + 80.0 * i as f32, 100.0) / GRID_UNIT_SIZE;
                    let size = (10.0, 10.0).into();
                    let input_locs_rel = (0..*n_inp + 1)
                        .map(|i| {
                            if i == 0 {
                                // clock
                                vec2(5.0, 10.0)
                            } else {
                                vec2(0.0, spc * i as f32)
                            }
                        })
                        .collect();
                    let data = DisplayData {
                        logical_loc: loc,
                        output_loc_rel: vec2(10.0, 5.0),
                        input_locs_rel,
                        id,
                        is_clocked: gate.clock_manager.is_some(),
                        scale: DEFAULT_SCALE,
                        size,
                    };
                    {
                        // display_data.insert(
                        //     id,
                        //     //todo: add various sizes for components and scale these fields accordingly
                        //     data,
                        // );
                        send_event(ui_sender, UiUpdateOps::AddComponent(data));
                    }
                }
            }
        });
        // print_screen(&display_state.screen);
        self.draw_connections(&display_state.wires, ui.painter());

        for (id, disp_data) in display_state.display_data.iter_mut() {
            let gate = ckt.get_component(id).unwrap();
            let res = paint_component(disp_data, ui, &mut gate.borrow_mut(), &mut self.from);
            for evt in res.0 {
                send_event(&self.ckt_sender, evt);
            }
            for evt in res.1 {
                send_event(&self.ui_sender, evt);
            }
        }
        ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
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
        });
        if !sync.is_error() {
            *sync = SyncState::Synced;
        }
    }
    fn draw_connections(&self, wires: &HashMap<(ID, (ID, PIN)), Wire>, pt: &Painter) {
        for wire in wires.values() {
            // cloning might be bad!
            pt.line(
                wire.pts
                    .iter()
                    .map(|k| {
                        *k * GRID_UNIT_SIZE + 0.0 * vec2(PIN_BTN_SIZE / 2.0, PIN_BTN_SIZE / 2.0)
                    })
                    .collect(),
                Stroke::new(wire.width, wire.col),
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
