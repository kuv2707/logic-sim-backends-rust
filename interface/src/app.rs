use std::{
    cmp::{max, min},
    collections::HashMap,
    f32,
    sync::Arc,
    thread,
};

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
    consts::{DEFAULT_SCALE, GRID_UNIT_SIZE, WINDOW_HEIGHT, WINDOW_WIDTH},
    draw_conns::{a_star_get_pts, Screen, UnitArea},
    update_ops::{SyncState, UpdateOps},
};

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

pub struct SimulatorUI {
    ckt: Arc<Mutex<BCircuit>>,
    pub display_data: HashMap<ID, DisplayData>,
    pub sender: Sender<UpdateOps>,
    sync: Arc<Mutex<SyncState>>,
    pub from: Option<ID>,
    pub available_comp_defns: Vec<(String, usize)>,
    pub screen: Screen,
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
                    // println!("{} {} {}", emitter_id, receiver_id, pin);
                    ckt.connect(receiver_id, pin, emitter_id)
                }
                UpdateOps::Disconnect(emitter_id, (receiver_id, pin)) => {
                    // println!("{} {} {}", emitter_id, receiver_id, pin);
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
            screen: make_screen(),
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
                    let loc = egui::pos2(40.0 + 80.0 * i as f32, 100.0) / GRID_UNIT_SIZE;
                    let input_locs_rel = (0..*n_inp + 1)
                        .map(|i| {
                            if i == 0 {
                                // clock
                                vec2(50.0, 90.0)
                            } else {
                                vec2(10.0, spc * i as f32)
                            }
                        })
                        .collect();
                    self.display_data.insert(
                        id,
                        //todo: add various sizes for components and scale these fields accordingly
                        DisplayData {
                            logical_loc: loc,
                            output_loc_rel: vec2(90.0, 50.0),
                            input_locs_rel,
                            id,
                            is_clocked: gate.clock_manager.is_some(),
                            scale: DEFAULT_SCALE,
                            size: (10.0, 10.0).into(),
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
        // algo:
        // generate map with obstacles
        // iterate through conns to get (start, end) pairs
        // call A* for all those pairs
        // render

        // todo: change screen array state as components are added/removed/moved
        // instead of rebuilding each time
        let mut screen = make_screen();

        for (id, disp_data) in &self.display_data {
            let p1 = disp_data.logical_loc;
            let p2 = p1 + vec2(disp_data.size.x, disp_data.size.y);
            for x in (max((p1.x-1.0) as i32, 0))..(min(p2.x as i32 + 1, WINDOW_WIDTH as i32)) {
                for y in
                    (max((p1.y - 1.0) as i32, 0))..(min(p2.y as i32 + 1, WINDOW_HEIGHT as i32))
                {
                    screen[y as usize][x as usize] = UnitArea::Unvisitable;
                }
            }
        }
        // print_screen(&screen);
        let stroke = Stroke::new(2.0, Color32::WHITE);
        for gt in ckt.components() {
            let gate = gt.borrow();
            let from = self.display_data.get(&gate.id).unwrap();
            let from = from.logical_loc * GRID_UNIT_SIZE + from.output_loc_rel;
            for (id, pin) in gate.get_output_receivers() {
                let rec_dd = self.display_data.get(id).unwrap();
                let to = rec_dd.logical_loc * GRID_UNIT_SIZE + rec_dd.input_locs_rel[*pin];
                // pt.line(vec![from, to], stroke);

                let pts = a_star_get_pts(
                    (
                        (from.x / GRID_UNIT_SIZE) as i32 + 3,
                        (from.y / GRID_UNIT_SIZE) as i32,
                    ),
                    (
                        (to.x / GRID_UNIT_SIZE) as i32 - 3,
                        (to.y / GRID_UNIT_SIZE) as i32,
                    ),
                    &screen,
                );
                pt.line(
                    pts.iter()
                        .map(|p| pos2(p.0 as f32 * GRID_UNIT_SIZE, p.1 as f32 * GRID_UNIT_SIZE))
                        .collect(),
                    stroke,
                );
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

fn make_screen() -> Screen {
    [[UnitArea::VACANT; WINDOW_WIDTH as usize]; WINDOW_HEIGHT as usize]
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
