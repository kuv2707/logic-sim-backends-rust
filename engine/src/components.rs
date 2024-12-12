use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt,
};

use crate::types::{BinaryLogicReducer, ID, UNASSIGNED};

pub struct ComponentDefParams {
    pub name: String,
    pub comp_type: char,
    pub eval: BinaryLogicReducer,
    pub default_inputs: u16,
    pub symbol: String,
}

pub struct Component {
    pub name: String,
    pub id: ID,
    eval: BinaryLogicReducer,
    pub state: bool,
    output_notifylist: Vec<(ID, u16)>,
    pub n_inp: u16,
    pub symbol: String,
    in_pins: Vec<bool>,
}

impl Component {
    pub fn from_params(p: &ComponentDefParams) -> Component {
        let mut c = Component {
            name: p.name.clone(),
            id: UNASSIGNED,
            eval: p.eval,
            state: false,
            output_notifylist: Vec::new(),
            n_inp: 2, // deal with p.default_inputs etc later
            symbol: p.symbol.clone(),
            in_pins: vec![false; p.default_inputs as usize],
        };
        c.state = (c.eval)(&c.in_pins); // sound initial assumption
        c
    }
    pub fn make_input(lab: &str, init: bool) -> Component {
        // the eval function will not be called on input elements
        return Component {
            name: String::from("Input"),
            id: UNASSIGNED,
            eval: |_| true,
            state: init,
            output_notifylist: Vec::new(),
            n_inp: 0,
            symbol: lab.to_string(),
            in_pins: Vec::new(),
        };
    }
    pub fn add_notify(&mut self, target_id: ID, n_pin: u16) {
        self.output_notifylist.push((target_id, n_pin));
    }

    pub fn set_pin_val(&mut self, pin: &u16, val: bool) {
        self.in_pins[*pin as usize - 1] = val;
    }

    // change the value of input pins connected to this component's
    // output and schedule an update for that component as a whole.
    // if this component's state didn't change when updated, then it
    // will not schedule updates for its neighbours.
    pub fn update(&mut self, mp: &HashMap<i32, RefCell<Component>>, exec_q: &mut VecDeque<ID>) {
        if !self.name.eq("Input") {
            let old_state = self.state;
            let new_state = (self.eval)(&self.in_pins);
            if new_state == old_state {
                return;
            }
            self.state = new_state;
        }
        for (id, pin) in &self.output_notifylist {
            let ele = mp.get(id);
            if ele.is_none() {
                eprintln!("No element with id {}", id);
            }
            let ele = ele.unwrap();
            ele.borrow_mut().set_pin_val(pin, self.state);
            // optimization to the exec_queue. If there are same id's
            // in succession, we don't need to run update for each.
            // Just updating once suffices.
            if exec_q.is_empty() || *exec_q.back().unwrap() != *id {
                exec_q.push_back(*id);
            }
        }
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str = if self.state {
            "\x1b[32mON\x1b[0m" // Green text for ON
        } else {
            "\x1b[31mOFF\x1b[0m" // Red text for OFF
        };

        write!(
            f,
            "{} ({} input{}) - Symbol: \x1b[33m{}\x1b[0m - State: {}",
            self.name,
            self.n_inp,
            if self.n_inp == 1 { "" } else { "s" },
            self.symbol,
            state_str
        )
    }
}

pub type Output = (ID, String);
