use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt,
};

use crate::{
    clock_manager::ClockManager,
    types::{BinaryLogicReducer, CLOCK_PIN, ID, UNASSIGNED},
    utils::form_expr,
};
#[derive(Clone)]
pub struct ComponentDefParams {
    pub name: String,
    pub label: String,
    pub comp_type: char,
    pub eval: BinaryLogicReducer,
    pub default_inputs: u16,
    pub symbol: String,
}

pub struct Gate {
    pub name: String,
    pub id: ID,
    pub comp_type: char, // 'g' or 'm'
    pub label: String,
    eval: BinaryLogicReducer,
    pub state: bool,
    output_recvlist: Vec<(ID, u16)>,
    pub n_inp: u16,
    pub symbol: String,
    in_pins: Vec<bool>,
    in_exprs: Vec<String>,
    pub state_expr: String,
    pub clock_manager: Option<ClockManager>,
}

impl Gate {
    pub fn from_params(p: ComponentDefParams) -> Gate {
        let mut c = Gate {
            name: p.name.clone(),
            id: UNASSIGNED,
            comp_type: p.comp_type,
            label: p.label.to_owned(),
            eval: p.eval,
            state: false,
            output_recvlist: Vec::new(),
            n_inp: p.default_inputs,
            symbol: p.symbol.clone(),
            in_pins: vec![false; p.default_inputs as usize],
            in_exprs: vec![String::new(); p.default_inputs as usize],
            state_expr: String::new(),
            clock_manager: None,
        };
        if c.n_inp > 0 {
            // not doing for inputs etc
            c.state = (c.eval)(&c.in_pins, false); // sound initial assumption
        }
        if c.comp_type == 'm' {
            c.clock_manager = Some(ClockManager::new());
            c.state_expr = p.label + "(t)"
        }
        c
    }
    pub fn make_input(lab: &str, init: bool) -> Gate {
        // the eval function will not be called on input elements
        let mut c = Gate::from_params(ComponentDefParams {
            name: String::from("Input"),
            label: lab.to_owned(),
            eval: |_, _| true,
            default_inputs: 0,
            symbol: lab.to_owned(),
            comp_type: 'i',
        });
        c.state = init;
        c.state_expr = lab.to_string();
        c
    }
    pub fn add_next(&mut self, target_id: ID, n_pin: u16) {
        self.output_recvlist.push((target_id, n_pin));
    }

    pub fn set_pin_val(&mut self, pin: &u16, val: bool) -> Result<(), String> {
        if *pin == CLOCK_PIN {
            match &mut self.clock_manager {
                Some(k) => {
                    // println!("trig {}",self.label);
                    k.push(val)
                }
                None => return Err(String::from("This is not a clocked component")),
            }
        } else {
            self.in_pins[*pin as usize - 1] = val;
        }
        Ok(())
    }
    pub fn set_pin_expr(&mut self, pin: &u16, val: String) -> Result<(), String> {
        if *pin == CLOCK_PIN {
            match &mut self.clock_manager {
                Some(k) => {
                    // println!("trig {}",self.label);
                    k.clk_expr(val)
                }
                None => return Err(String::from("This is not a clocked component")),
            }
        } else {
            self.in_exprs[*pin as usize - 1] = val;
        }
        Ok(())
    }
}

// change the value of input pins connected to this component's
// output and schedule an update for that component as a whole.
// if this component's state didn't change when updated, then it
// will not schedule updates for its neighbours.

pub fn power_on_component(
    c: &mut Gate,
    mp: &HashMap<i32, RefCell<Gate>>,
    exec_q: &mut VecDeque<ID>,
) {
    state_update(c, mp, exec_q, false);
}

pub fn update_component_state(
    c: &mut Gate,
    mp: &HashMap<i32, RefCell<Gate>>,
    exec_q: &mut VecDeque<ID>,
) {
    state_update(c, mp, exec_q, true);
}

fn state_update(
    c: &mut Gate,
    mp: &HashMap<i32, RefCell<Gate>>,
    exec_q: &mut VecDeque<ID>,
    optimize: bool,
) {
    match &mut c.clock_manager {
        Some(mag) => {
            // clocked component
            if mag.clock_triggered() {
                mag.reset_clock_hist();
                c.state = (c.eval)(&c.in_pins, c.state);
            }
        }
        None => 'optimizer: {
            if c.name.eq("Input") {
                break 'optimizer;
            }
            let old_state = c.state;
            let new_state = (c.eval)(&c.in_pins, old_state);
            if optimize && new_state == old_state {
                // buggy optimization: all components should
                // trigger updates to their neighbours at least
                // once. todo: think sth else
                return;
            }
            c.state = new_state;
        }
    }
    // println!("{} {} : {}",c.name, c.symbol, c.state);
    for (id, pin) in &c.output_recvlist {
        let ele = mp.get(id);
        if ele.is_none() {
            eprintln!("No element with id {}", id);
        }
        let mut ele = ele.unwrap().borrow_mut();
        // println!("from {} to {} {} : {}",c.label, ele.label, pin, c.state);
        ele.set_pin_val(pin, c.state).unwrap();
        // optimization to the exec_queue. If there are same id's
        // in succession, we don't need to run update for each.
        // Just updating once suffices.
        if exec_q.is_empty() || *exec_q.back().unwrap() != *id {
            exec_q.push_back(*id);
        }
    }
}

pub fn evaluate_component_expression(
    c: &mut Gate,
    mp: &HashMap<i32, RefCell<Gate>>,
    exec_q: &mut VecDeque<ID>,
) {
    if !c.name.eq("Input") && c.clock_manager.is_none() {
        let old_expr = &c.state_expr;
        let new_expr = form_expr(&c.in_exprs, &c.symbol);
        if new_expr.eq(old_expr) {
            return;
        }
        c.state_expr = new_expr;
    }
    for (id, pin) in &c.output_recvlist {
        let ele = mp.get(id);
        if ele.is_none() {
            eprintln!("No element with id {}", id);
        }
        let mut ele = ele.unwrap().borrow_mut();
        ele.set_pin_expr(pin, c.state_expr.clone()).unwrap();
        // optimization to the exec_queue. If there are same id's
        // in succession, we don't need to run update for each.
        // Just updating once suffices.
        if exec_q.is_empty() || *exec_q.back().unwrap() != *id {
            exec_q.push_back(*id);
        }
    }
}

impl fmt::Display for Gate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state_str = if self.state {
            "\x1b[32mON\x1b[0m" // Green text for ON
        } else {
            "\x1b[31mOFF\x1b[0m" // Red text for OFF
        };

        write!(
            f,
            "{} ({} input{}) - Symbol: \x1b[33m{}\x1b[0m - State: {}\n{}",
            self.name,
            self.n_inp,
            if self.n_inp == 1 { "" } else { "s" },
            self.symbol,
            state_str,
            self.in_exprs.join("  \n")
        )
    }
}
