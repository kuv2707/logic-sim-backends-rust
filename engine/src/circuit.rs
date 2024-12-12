use crate::{
    components::{
        evaluate_component_expression, update_component_state, ComponentDefParams, Gate, Output,
    },
    table::Table,
    types::{ComponentActor, COMPONENT_NOT_DEFINED, ID},
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
};

pub struct BCircuit {
    component_definitions: HashMap<String, ComponentDefParams>,
    components: HashMap<ID, RefCell<Gate>>,
    inputs: HashSet<String>,
    outputs: HashMap<String, Output>,
    last_id: ID,
    pub exec_queue: VecDeque<ID>,
}

impl BCircuit {
    pub fn new() -> BCircuit {
        let mut c = BCircuit {
            component_definitions: HashMap::new(),
            components: HashMap::new(),
            inputs: HashSet::new(),
            outputs: HashMap::new(),
            last_id: 0,
            exec_queue: VecDeque::new(),
        };
        define_common_gates(&mut c);
        c
    }
    pub fn run(&mut self) {
        self.graph_act(update_component_state);
    }
    pub fn graph_act(&mut self, runnable: ComponentActor) {
        // traverse in breadth-first fashion, starting from inputs
        // and calls the specified function on the components
        // todo: add clocked components (circuit states)
        for (id, c) in &self.components {
            if c.borrow().name.eq("Input") {
                self.exec_queue.push_back(*id);
            }
        }
        while !self.exec_queue.is_empty() {
            let id = self.exec_queue.pop_front().unwrap();
            let mut k = self.components.get(&id).unwrap().borrow_mut();
            // println!("acting on {}", k.name);
            runnable(&mut k, &self.components, &mut self.exec_queue);
        }
    }
    pub fn define_gate(&mut self, p: ComponentDefParams) {
        self.component_definitions
            .insert(String::from(p.name.clone()), ComponentDefParams::from(p));
    }
    pub fn add_component(&mut self, typ: &str) -> ID {
        let id = self.new_id();
        let comp = self.make_component(typ);
        match comp {
            Ok(mut c) => {
                c.id = id;
                self.components.insert(id, RefCell::new(c));
                return id;
            }
            Err(e) => e,
        }
    }
    pub fn get_component(&mut self, id: &ID) -> Option<&mut RefCell<Gate>> {
        return self.components.get_mut(id);
    }
    fn make_component(&mut self, typ: &str) -> Result<Gate, ID> {
        let def = self.component_definitions.get(typ);
        if def.is_none() {
            return Err(COMPONENT_NOT_DEFINED);
        }
        let def = def.unwrap();
        return Ok(Gate::from_params(def));
    }
    pub fn register_input(&mut self, label: &str, init_val: bool) -> ID {
        let mut inp = Gate::make_input(label, init_val);
        let id = self.new_id();
        inp.id = id;
        self.components.insert(id, RefCell::new(inp));
        self.inputs.insert(label.to_string());
        return id;
    }
    pub fn connect(&mut self, receiver_id: ID, pin: u16, emitter_id: ID) -> Result<(), String> {
        let receiver = self.components.remove(&receiver_id);
        if receiver.is_none() {
            return Err("Invalid receiver id".to_string());
        }
        let receiver = receiver.unwrap();

        let emitter = self.components.remove(&emitter_id);
        if emitter.is_none() {
            return Err("Invalid emitter id".to_string());
        }
        let emitter = emitter.unwrap();

        if pin > receiver.borrow().n_inp {
            return Err(format!(
                "There are only {} pins, can't access {}",
                receiver.borrow().n_inp,
                pin
            ));
        }

        emitter.borrow_mut().add_notify(receiver_id, pin);

        self.components.insert(receiver_id, receiver);
        self.components.insert(emitter_id, emitter);

        return Ok(());
    }
    pub fn track_output(&mut self, comp_id: ID, lab: &str) -> bool {
        if self.components.get(&comp_id).is_none() {
            return false;
        }
        self.outputs
            .insert(lab.to_string(), (comp_id, lab.to_string()));
        true
    }
    pub fn compile(&mut self) {
        // 1. we generate boolean expression for each component
        // in the circuit.
        // 2. we check for dangerous loops.
        // 3. we create input/output expression for each component
        self.graph_act(evaluate_component_expression);
    }
    pub fn gen_truth_table(&self) -> Table {
        todo!()
    }
    pub fn state(&self, id: ID) -> Option<bool> {
        match self.components.get(&id) {
            Some(a) => Some(a.borrow().state),
            None => None,
        }
    }
    pub fn new_id(&mut self) -> ID {
        self.last_id += 1;
        return self.last_id;
    }
}

fn define_common_gates(c: &mut BCircuit) {
    c.define_gate(ComponentDefParams {
        name: "NAND".to_string(),
        comp_type: 'g',
        eval: |v| {
            // println!("{:?}", v);
            return !(v.iter().fold(true, |a, b| a && *b));
        },
        default_inputs: 2,
        symbol: "!.".to_string(),
    });
    c.define_gate(ComponentDefParams {
        name: "AND".to_string(),
        comp_type: 'g',
        eval: |v| {
            return v.iter().fold(true, |a, b| a && *b);
        },
        default_inputs: 2,
        symbol: ".".to_string(),
    });
    c.define_gate(ComponentDefParams {
        name: "OR".to_string(),
        comp_type: 'g',
        eval: |v| {
            return v.iter().fold(false, |a, b| a || *b);
        },
        default_inputs: 2,
        symbol: "+".to_string(),
    });
    c.define_gate(ComponentDefParams {
        name: "NOT".to_string(),
        comp_type: 'g',
        eval: |v| {
            return !v[0];
        },
        default_inputs: 1,
        symbol: "!".to_string(),
    });
}
