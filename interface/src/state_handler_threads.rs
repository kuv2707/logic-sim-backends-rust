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
    utils::EmitterReceiverPair,
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
            CircuitUpdateOps::Connect(er_pair) => {
                let res = ckt.connect(
                    er_pair.receiver.1.id,
                    er_pair.receiver.1.pin,
                    er_pair.emitter.1.id,
                );
                if res.is_ok() {
                    ui_sender.send(UiUpdateOps::Connect(er_pair)).unwrap();
                }
                res
            }
            CircuitUpdateOps::Disconnect(er_pair) => {
                let res = ckt.disconnect(
                    er_pair.receiver.1.id,
                    er_pair.receiver.1.pin,
                    er_pair.emitter.1.id,
                );
                if res.is_ok() {
                    ui_sender.send(UiUpdateOps::Disconnect(er_pair)).unwrap();
                }
                res
            }
            CircuitUpdateOps::Remove(id) => {
                let res = ckt.remove_component(id);
                res
            }
            CircuitUpdateOps::SetComponentLabel(id, old_label, new_label) => {
                // old_label is for undo support
                ckt.set_component_label(id, &new_label)
            }
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
    ckt_sender: Sender<CircuitUpdateOps>,
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
                // between the remove_component being called on the ckt
                // and this code executing (removing the id from display_data)
                // if the user drags, a drag event would be sent, but that would
                // be sequenced after this code, so there's no chance of a
                // transient inconsistent state causing the app to crash.
                clear_screen(&mut ds.screen);
                mark_obstacles(ds);
                let dparams = ds
                    .display_data
                    .remove(&id)
                    .expect("Can't remove it as it is already absent! STALE UI!!");

                let mut remove_list = Vec::new();
                for er_pair in ds.wires.keys() {
                    if dparams.contents.contains(&er_pair.emitter.1.id)
                        || dparams.contents.contains(&er_pair.receiver.1.id)
                    {
                        remove_list.push(*er_pair);
                    }
                }
                for rem_key in remove_list {
                    ds.wires.remove(&rem_key);
                }
                for remid in dparams.contents {
                    ckt_sender.send(CircuitUpdateOps::Remove(remid)).unwrap();
                }
            }
            UiUpdateOps::Connect(er_pair) => {
                let pts = find_path(ds, &er_pair);
                ds.wires.insert(
                    er_pair,
                    Wire {
                        pts,
                        emitter: er_pair.emitter,
                        width: 2.0, // todo: make bolder when component is selected
                    },
                );
            }
            UiUpdateOps::Disconnect(er_pair) => {
                ds.wires.remove(&er_pair);
            }
            UiUpdateOps::Select(id) => {
                // todo: select, deselect single or multiple elems.
            }
        }
    }
}

fn update_wires(ds: &mut DisplayState) {
    let keys = ds
        .wires
        .keys()
        .cloned()
        .collect::<Vec<EmitterReceiverPair>>();
    for er_pair in keys {
        let newpts = find_path(ds, &er_pair);
        ds.wires.get_mut(&er_pair).unwrap().pts = newpts;
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
        for oloc in &dd.outputs_rel {
            let oloc = dd.logical_loc + oloc.loc_rel;
            ds.screen[oloc.y as usize][oloc.x as usize] = UnitArea::VACANT;
        }
        for iloc in &dd.inputs_rel {
            let iloc = dd.logical_loc + iloc.loc_rel;
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

fn find_path(ds: &DisplayState, er_pair: &EmitterReceiverPair) -> Vec<Pos2> {
    let oploc = ds.display_data.get(&er_pair.emitter.0).unwrap();
    let oploc = oploc.logical_loc + er_pair.emitter.1.loc_rel;

    let iploc = ds.display_data.get(&er_pair.receiver.0).unwrap();
    let iploc = iploc.logical_loc + er_pair.receiver.1.loc_rel;

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
