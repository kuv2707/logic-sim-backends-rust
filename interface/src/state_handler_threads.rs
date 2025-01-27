use std::{
    cmp::{max, min},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use bsim_engine::{
    circuit::BCircuit,
    types::{ID, PIN},
};
use crossbeam::channel::{Receiver, Sender};
use egui::{pos2, vec2, Color32, Pos2};

use crate::{
    consts::{WINDOW_HEIGHT, WINDOW_WIDTH},
    display_elems::{DisplayState, Screen, UnitArea, Wire},
    path_find::a_star_get_pts,
    update_ops::{CircuitUpdateOps, SyncState, UiUpdateOps},
};

pub fn toggle_clock(
    ckt: Arc<Mutex<BCircuit>>,
    display_state: Arc<Mutex<DisplayState>>,
    clk_id: ID,
) -> impl Fn() {
    move || loop {
        let delay;
        {
            // put in a scope to release the locks before the thread sleeps.
            let mut ckt = ckt.lock().unwrap();
            // .pulse_clock();
            let new_state = !ckt.state(clk_id).unwrap();
            ckt.set_component_state(clk_id, new_state).unwrap();
            let ds = display_state.lock().unwrap();
            ds.ctx.request_repaint();
            delay = ds.clk_t;
        }
        thread::sleep(Duration::from_millis(delay));
    }
}

pub fn ckt_communicate(
    receiver: Receiver<CircuitUpdateOps>,
    update_am: Arc<Mutex<BCircuit>>,
    sync: Arc<Mutex<SyncState>>,
    ui_sender: Sender<UiUpdateOps>,
) -> impl Fn() {
    move || loop {
        let rec = receiver.recv().unwrap();
        let mut ckt = update_am.lock().unwrap();
        let result = match rec {
            CircuitUpdateOps::SetState(id, val) => ckt.set_component_state(id, val),
            CircuitUpdateOps::Connect(emitter_id, (receiver_id, pin)) => {
                let res = ckt.connect(receiver_id, pin, emitter_id);
                if res.is_ok() {
                    ui_sender
                        .send(UiUpdateOps::Connect(receiver_id, (emitter_id, pin)))
                        .unwrap();
                }
                res
            }
            CircuitUpdateOps::Disconnect(emitter_id, (receiver_id, pin)) => {
                let res = ckt.disconnect(receiver_id, pin, emitter_id);
                if res.is_ok() {
                    ui_sender
                        .send(UiUpdateOps::Disconnect(receiver_id, (emitter_id, pin)))
                        .unwrap();
                }
                res
            }
            CircuitUpdateOps::Remove(id) => {
                let res = ckt.remove_component(id);
                res
            },
        };

        let mut s = sync.lock().unwrap();
        *s = match result {
            Ok(()) => SyncState::NotSynced,
            Err(e) => SyncState::Error(e),
        };
    }
}

pub fn ui_update(
    receiver: Receiver<UiUpdateOps>,
    display_state: Arc<Mutex<DisplayState>>,
) -> impl Fn() {
    move || loop {
        let rec = receiver.recv().unwrap();
        let ds = &mut display_state.lock().unwrap();

        match rec {
            UiUpdateOps::AddComponent(dd) => {
                ds.display_data.insert(dd.id, dd);
                mark_obstacles(ds);
            }
            UiUpdateOps::Dragged => {
                clear_screen(&mut ds.screen);
                mark_obstacles(ds);
                update_wires(ds);
            }
            UiUpdateOps::RemoveComponent(id) => {
                ds.display_data.remove(&id);
                clear_screen(&mut ds.screen);
                mark_obstacles(ds);
                let mut remove_list = Vec::new();
                for wire in ds.wires.keys() {
                    if wire.0 == id || wire.1.0 == id {
                        remove_list.push(*wire);
                    }
                }
                for rem_key in remove_list {
                    ds.wires.remove(&rem_key);
                }
            }
            UiUpdateOps::Connect(rec_id, (send_id, pin)) => {
                let pts = find_path(ds, &send_id, &rec_id, pin);
                ds.wires.insert(
                    (send_id, (rec_id, pin)),
                    Wire {
                        pts,
                        col: Color32::WHITE, // todo: replace with sth else if component is selected
                        width: 2.0,          // todo: make bolder when component is selected
                    },
                );
            }
            UiUpdateOps::Disconnect(rec_id, (send_id, pin)) => {
                ds.wires.remove(&(send_id, (rec_id, pin)));
            }
            UiUpdateOps::Select(id) => {
                // todo: select, deselect single or multiple elems.
                // let keys = ds
                //     .wires
                //     .keys()
                //     .filter(|k| k.0 == id)
                //     .cloned()
                //     .collect::<Vec<(ID, (ID, PIN))>>();
                // for (send_id, (recv_id, pin)) in keys {
                //     let w = ds.wires.get_mut(&(send_id, (recv_id, pin))).unwrap();
                //     w.width = 5.0;
                //     w.col = Color32::YELLOW;
                // }
            }
        }
    }
}

fn update_wires(ds: &mut DisplayState) {
    let keys = ds.wires.keys().cloned().collect::<Vec<(ID, (ID, PIN))>>();
    for (send_id, (recv_id, pin)) in keys {
        let newpts = find_path(ds, &send_id, &recv_id, pin);
        ds.wires.get_mut(&(send_id, (recv_id, pin))).unwrap().pts = newpts;
    }
}

fn mark_obstacles(ds: &mut DisplayState) {
    for (_, dd) in &ds.display_data {
        let p1 = dd.logical_loc;
        let p2 = p1 + vec2(dd.size.x, dd.size.y);
        for x in (max((p1.x - 1.0) as i32, 0))..(min(p2.x as i32 + 1, WINDOW_WIDTH as i32)) {
            for y in (max((p1.y - 1.0) as i32, 0))..(min(p2.y as i32 + 1, WINDOW_HEIGHT as i32)) {
                ds.screen[y as usize][x as usize] = UnitArea::Unvisitable;
            }
        }
        let oloc = dd.logical_loc + dd.output_loc_rel;
        ds.screen[oloc.y as usize][oloc.x as usize] = UnitArea::VACANT;
        for iloc in &dd.input_locs_rel {
            let iloc = dd.logical_loc + *iloc;
            ds.screen[iloc.y as usize][iloc.x as usize - 1] = UnitArea::VACANT;
            ds.screen[iloc.y as usize][iloc.x as usize] = UnitArea::VACANT;
        }
    }
}

fn clear_screen(s: &mut Screen) {
    for row in s.iter_mut() {
        for unit in row.iter_mut() {
            *unit = UnitArea::VACANT;
        }
    }
}

fn find_path(ds: &DisplayState, send_id: &ID, recv_id: &ID, pin: PIN) -> Vec<Pos2> {
    let oploc = ds.display_data.get(send_id).unwrap();
    let oploc = oploc.logical_loc + oploc.output_loc_rel;

    let iploc = ds.display_data.get(recv_id).unwrap();
    let iploc = iploc.logical_loc + iploc.input_locs_rel[pin];

    a_star_get_pts(
        (oploc.x as i32, oploc.y as i32),
        (iploc.x as i32, iploc.y as i32),
        // (5,5),(3,10),
        &ds.screen,
    )
    .iter()
    .map(|f| pos2(f.0 as f32, f.1 as f32))
    .collect()
}
