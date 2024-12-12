use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt,
};

use crate::{
    types::{BinaryLogicReducer, ID, UNASSIGNED},
    utils::form_expr,
};

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
    in_exprs: Vec<String>,
    pub state_expr: String,
}

impl Component {
    pub fn from_params(p: &ComponentDefParams) -> Component {
        let mut c = Component {
            name: p.name.clone(),
            id: UNASSIGNED,
            eval: p.eval,
            state: false,
            output_notifylist: Vec::new(),
            n_inp: p.default_inputs,
            symbol: p.symbol.clone(),
            in_pins: vec![false; p.default_inputs as usize],
            in_exprs: vec![String::new(); p.default_inputs as usize],
            state_expr: String::new(),
        };
        if c.n_inp > 0 {
            // not doing for inputs etc
            c.state = (c.eval)(&c.in_pins); // sound initial assumption
        }
        c
    }
    pub fn make_input(lab: &str, init: bool) -> Component {
        // the eval function will not be called on input elements
        let mut c = Component::from_params(&ComponentDefParams {
            name: String::from("Input"),
            eval: |_| true,
            default_inputs: 0,
            symbol: lab.to_string(),
            comp_type: 'i',
        });
        c.state = init;
        c.state_expr = lab.to_string();
        c
    }
    pub fn add_notify(&mut self, target_id: ID, n_pin: u16) {
        self.output_notifylist.push((target_id, n_pin));
    }

    pub fn set_pin_val(&mut self, pin: &u16, val: bool) {
        self.in_pins[*pin as usize - 1] = val;
    }
    pub fn set_pin_expr(&mut self, pin: &u16, val: String) {
        self.in_exprs[*pin as usize - 1] = val;
    }
}

// change the value of input pins connected to this component's
// output and schedule an update for that component as a whole.
// if this component's state didn't change when updated, then it
// will not schedule updates for its neighbours.
pub fn update_component_state(
    c: &mut Component,
    mp: &HashMap<i32, RefCell<Component>>,
    exec_q: &mut VecDeque<ID>,
) {
    if !c.name.eq("Input") {
        let old_state = c.state;
        let new_state = (c.eval)(&c.in_pins);
        if new_state == old_state {
            // buggy optimization: all components should
            // trigger updates to their neighbours at least 
            // once. todo: think sth else
            // return;
        }
        c.state = new_state;
    }
    // println!("{} {} : {}",c.name, c.symbol, c.state);
    for (id, pin) in &c.output_notifylist {
        let ele = mp.get(id);
        if ele.is_none() {
            eprintln!("No element with id {}", id);
        }
        let ele = ele.unwrap();
        ele.borrow_mut().set_pin_val(pin, c.state);
        // optimization to the exec_queue. If there are same id's
        // in succession, we don't need to run update for each.
        // Just updating once suffices.
        exec_q.push_back(*id);
        if exec_q.is_empty() || *exec_q.back().unwrap() != *id {
        }
    }
}

pub fn evaluate_component_expression(
    c: &mut Component,
    mp: &HashMap<i32, RefCell<Component>>,
    exec_q: &mut VecDeque<ID>,
) {
    if !c.name.eq("Input") {
        let old_expr = &c.state_expr;
        let new_expr = form_expr(&c.in_exprs, &c.symbol);
        if new_expr.eq(old_expr) {
            return;
        }
        c.state_expr = new_expr;
    }
    for (id, pin) in &c.output_notifylist {
        let ele = mp.get(id);
        if ele.is_none() {
            eprintln!("No element with id {}", id);
        }
        let ele = ele.unwrap();
        ele.borrow_mut().set_pin_expr(pin, c.state_expr.clone());
        // optimization to the exec_queue. If there are same id's
        // in succession, we don't need to run update for each.
        // Just updating once suffices.
        if exec_q.is_empty() || *exec_q.back().unwrap() != *id {
            exec_q.push_back(*id);
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
