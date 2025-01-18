use std::{collections::HashMap, f32, sync::Arc, thread};

use bsim_engine::{
    circuit::BCircuit,
    types::{CLOCK_PIN, ID},
};
use crossbeam::channel::{self, Sender};
use egui::{
    mutex::Mutex, pos2, vec2, Button, Color32, Layout, Painter, Pos2, Stroke, Ui, Vec2, Widget,
};

use crate::{
    component_ui::paint_component,
    update_ops::{SyncState, UpdateOps},
};

pub struct DisplayData {
    pub loc: Pos2,
    pub output_loc: Vec2,
    pub input_locs: Vec<Vec2>,
    pub id: ID,
    pub is_clocked: bool,
    pub scale: f32,
}

pub struct SimulatorUI {
    ckt: Arc<Mutex<BCircuit>>,
    pub display_data: HashMap<ID, DisplayData>,
    pub sender: Sender<UpdateOps>,
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

        let (sender, receiver) = channel::unbounded();
        let am = Arc::new(Mutex::new(ckt));
        let update_am = am.clone();

        let sync_state = SyncState::Synced;
        let sync = Arc::new(Mutex::new(sync_state));
        let sync_c = sync.clone();
        thread::spawn(move || loop {
            let rec = receiver.recv().unwrap();
            let mut ckt = update_am.lock();
            let result = match rec {
                UpdateOps::SetState(id, val) => ckt.set_component_state(id, val),
                UpdateOps::Connect(emitter_id, (receiver_id, pin)) => {
                    println!("{} {} {}", emitter_id, receiver_id, pin);
                    ckt.connect(receiver_id, pin, emitter_id)
                }
                UpdateOps::Disconnect(emitter_id, (receiver_id, pin)) => {
                    println!("{} {} {}", emitter_id, receiver_id, pin);
                    ckt.disconnect(receiver_id, pin, emitter_id)
                }
                UpdateOps::Remove(id) => ckt.remove_component(id),
            };

            let mut s = sync.lock();
            *s = match result {
                Ok(()) => SyncState::NotSynced,
                Err(e) => SyncState::Error(e),
            };
        });
        let sim = Self {
            ckt: am,
            display_data: HashMap::new(),
            sender,
            sync: sync_c,
            from: None,
            available_comp_defns,
        };
        sim
    }
    fn ui(&mut self, ui: &mut Ui) {
        let mut ckt = self.ckt.lock();
        let mut sync = self.sync.lock();
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, Color32::from_rgb(80, 60, 60));

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
                    let spc = 100.0 / (n_inp + 1) as f32;
                    let loc = egui::pos2(40.0 + 80.0 * i as f32, 50.0);
                    self.display_data.insert(
                        id,
                        //todo: add various sizes for components and scale these fields accordingly
                        DisplayData {
                            loc,
                            output_loc: vec2(90.0, 50.0),
                            input_locs: (0..*n_inp)
                                .map(|i| vec2(10.0, spc * (i + 1) as f32))
                                .collect(),
                            id,
                            is_clocked: gate.clock_manager.is_some(),
                            scale: 5.0,
                        },
                    );
                }
            }
        });

        self.draw_connections(&ckt, ui.painter());

        for gate in ckt.components() {
            let disp_data = self.display_data.get_mut(&gate.borrow().id).unwrap();
            for ev in paint_component(disp_data, ui, &mut gate.borrow_mut(), &mut self.from) {
                self.emit_event(ev);
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
    fn draw_connections(&self, ckt: &BCircuit, pt: &Painter) {
        for gate in ckt.components() {
            let d_data_from = self.display_data.get(&gate.borrow().id).unwrap();
            let from = d_data_from.loc + d_data_from.output_loc;
            for (nx_id, pin) in gate.borrow().get_output_receivers() {
                let d_data_to = self.display_data.get(nx_id).unwrap();
                let to = d_data_to.loc
                    + if *pin != CLOCK_PIN {
                        d_data_to.input_locs[*pin as usize - 1]
                    } else {
                        vec2(50.0, 90.0)
                    };
                let mut pts = Vec::<Pos2>::new();
                pts.reserve(6);
                pts.push(from);
                pts.push(from + vec2(20.0, 0.));
                let m1 = pos2(f32::max((from.x + to.x) / 2.0, from.x + 20.0), from.y);
                pts.push(m1);
                let m2 = pos2(f32::min((from.x + to.x) / 2.0, to.x - 20.0), to.y);

                if m1.x != m2.x {
                    let half = 0.5;
                    let m3 = pos2(m1.x, f32::min((m1.y + m2.y) * half, d_data_from.loc.y));
                    let m4 = pos2(
                        m2.x,
                        f32::max((m1.y + m2.y) * half, d_data_to.loc.y + 100.0), // todo: bottom_right not working
                    );
                    pts.push(m3);
                    if m3.y != m4.y {
                        pts.push(pos2((m3.x + m4.x) * half, m3.y));
                        pts.push(pos2((m3.x + m4.x) * half, m4.y));
                    }
                    pts.push(m4);
                }
                pts.push(m2);
                pts.push(to - vec2(20.0, 0.));
                pts.push(to);
                //todo: replace with some path finding algorithm
                pt.line(pts, Stroke::new(2.0, Color32::BLACK));
            }
        }
    }
    pub fn emit_event(&self, ev: UpdateOps) {
        if let Err(err) = self.sender.send(ev) {
            println!("{}", err.to_string());
        }
    }
}

impl eframe::App for SimulatorUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });
    }
}
