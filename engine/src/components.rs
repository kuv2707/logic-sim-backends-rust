use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    fmt::{self},
};

use crate::{
    clock_manager::ClockManager,
    types::{BinaryLogicReducer, CompType, CLOCK_PIN, ID, NULL, PIN, UNASSIGNED},
    utils::form_expr,
};
#[derive(Clone)]
pub struct ComponentDefParams {
    pub name: String,
    pub label: String,
    pub comp_type: CompType,
    pub eval: BinaryLogicReducer,
    pub default_inputs: u16,
    pub symbol: String,
}

pub struct Gate {
    pub name: String,
    pub id: ID,
    pub comp_type: CompType,
    pub label: String, //todo: shift to comp_type
    eval: BinaryLogicReducer,
    pub state: bool,
    pub(crate) output_recvlist: HashSet<(ID, PIN)>,
    #[deprecated(note = "Compute from vector sizes of `self.input_pin_values`")]
    n_inp: usize,
    pub symbol: String,
    input_pin_values: Vec<bool>,
    pub input_pin_sources: Vec<ID>,
    pub input_pin_exprs: Vec<String>,
    pub state_expr: String,
    pub clock_manager: Option<ClockManager>,
}

impl Gate {
    pub fn from_params(p: ComponentDefParams) -> Gate {
        let n_inp = p.default_inputs as usize + 1;
        let mut c = Gate {
            name: p.name.clone(),
            id: UNASSIGNED,
            comp_type: p.comp_type,
            label: p.label.to_owned(),
            eval: p.eval,
            state: false,
            output_recvlist: HashSet::new(),
            n_inp,
            symbol: p.symbol.clone(),

            // 0th pin is the clock pin
            input_pin_values: vec![false; n_inp],
            input_pin_sources: vec![NULL; n_inp],
            input_pin_exprs: vec![String::new(); n_inp],
            state_expr: String::new(),

            clock_manager: None,
        };
        if n_inp > 0 {
            // not doing for inputs etc
            c.state = (c.eval)(&c.input_pin_values, false); // sound initial assumption
        }
        if c.comp_type == CompType::Sequential {
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
            eval: |_, old| old, // can only be externally changed
            default_inputs: 0,
            symbol: lab.to_owned(),
            comp_type: CompType::Input,
        });
        c.state = init;
        c.state_expr = lab.to_string();
        c
    }
    pub fn num_inputs(&self) -> usize {
        self.input_pin_exprs.len()
    }
    pub fn set_state(&mut self, state: bool) {
        self.state = state;
    }
    pub fn has_independent_state(&self) -> bool {
        // input and clocked components have independent state
        self.comp_type == CompType::Input || self.comp_type == CompType::Sequential
    }
    pub fn link_output_receiver(&mut self, receiver_id: ID, pin: PIN) {
        self.output_recvlist.insert((receiver_id, pin));
    }
    pub fn unlink_output_receiver(&mut self, receiver_id: ID, pin: PIN) -> Result<(), String> {
        let had = self.output_recvlist.remove(&(receiver_id, pin));
        if !had {
            Err(format!(
                "No connection b/w id_{} and id_{} pin_{}",
                self.id, receiver_id, pin
            ))
        } else {
            Ok(())
        }
    }
    pub fn get_output_receivers(&self) -> &HashSet<(ID, PIN)> {
        return &self.output_recvlist;
    }
    pub fn set_input_pin_connection(&mut self, pin: PIN, emitter: &Gate) -> Result<(), String> {
        if pin >= self.num_inputs() {
            return Err(format!(
                "Only have {} input pins in {}, can't access pin_{}",
                self.num_inputs(),
                self.name,
                pin,
            ));
        }
        if self.input_pin_sources[pin] != NULL {
            return Err(format!("Please disconnect it first!"));
        }
        // we do allow setting CLOCK_PIN`th index for non clocked compos
        // they are simply never used
        self.input_pin_values[pin as usize] = emitter.state;
        self.input_pin_sources[pin as usize] = emitter.id;
        self.input_pin_exprs[pin as usize].push_str(&emitter.state_expr);

        Ok(())
    }
    pub fn clear_input_pin_connection(&mut self, pin: PIN) -> Result<(), String> {
        if pin >= self.num_inputs() {
            return Err(format!(
                "Only have {} input pins in {}, can't clear pin_{}",
                self.num_inputs(),
                self.name,
                pin,
            ));
        }
        // we do allow setting CLOCK_PIN`th index for non clocked compos
        // they are simply never used
        // println!("{} inppin_{} val {}", self.label, pin, false);
        self.input_pin_values[pin] = false;
        self.input_pin_sources[pin] = NULL;
        self.input_pin_exprs[pin].clear();

        Ok(())
    }
    pub(crate) fn set_pin_val(&mut self, pin: PIN, val: bool) {
        if pin == CLOCK_PIN {
            if let Some(cm) = &mut self.clock_manager {
                cm.push(val);
            }
        }
        self.input_pin_values[pin] = val;
    }
    pub fn set_pin_expr(&mut self, pin: PIN, val: &str) {
        self.input_pin_exprs[pin].replace_range(.., &val);
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
    let new_state = match &mut c.clock_manager {
        Some(mag) => {
            // clocked component
            if mag.clock_triggered() {
                mag.reset_clock_hist();
                (c.eval)(&c.input_pin_values, c.state)
            } else {
                c.state
            }
        }
        None => (c.eval)(&c.input_pin_values, c.state),
    };

    if optimize && new_state == c.state {
        return;
    }
    c.state = new_state;
    // println!("{} {} : {}", c.name, c.label, c.state);
    for (id, pin) in &c.output_recvlist {
        let mut ele = mp
            .get(id)
            .expect(&format!("Expected id_{} to be present", id))
            .borrow_mut();

        // println!("from {} to {} {} : {}", c.label, ele.label, pin, c.state);
        ele.set_pin_val(*pin, c.state);

        // optimization to the exec_queue. If there are same id's
        // in succession, we don't need to run update for each.
        // Just updating once suffices.
        if exec_q.is_empty() || *exec_q.back().unwrap() != *id {
            exec_q.push_back(*id);
        }
    }
}

pub(crate) fn set_expressions(c: &mut Gate, mp: &HashMap<i32, RefCell<Gate>>, exec_q: &mut VecDeque<ID>) {
    match c.comp_type {
        CompType::Combinational => c.state_expr = form_expr(&c.input_pin_exprs, &c.symbol),
        _ => {
            c.state_expr = c.label.clone();
        }
    }
    for (id, pin) in &c.output_recvlist {
        let mut ele = mp
            .get(id)
            .expect(&format!("Expected id_{} to be present", id))
            .borrow_mut();

        // println!("from {} to {} {} : {}", c.label, ele.label, pin, c.state);
        ele.set_pin_expr(*pin, c.state_expr.as_str());

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
            self.input_pin_exprs.join("  \n")
        )
    }
}
