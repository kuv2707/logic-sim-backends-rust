use crate::{
    bootstrap::bootstrap_ckt,
    components::{
        power_on_component, set_expressions, update_component_state, ComponentDefParams, Gate,
    },
    table::{bitwise_counter, Table},
    types::{CompType, ComponentActor, ID, NULL, PIN},
};
use std::{
    cell::RefCell,
    collections::{hash_map::Values, HashMap, HashSet, VecDeque},
};

pub struct BCircuit {
    pub component_definitions: HashMap<String, ComponentDefParams>,
    // todo: Maybe using RC<Refcell<>> will ease connection management
    // since then each gate would only hold a reference to neighbouring
    // gates instead of separately storing state, expr, src.
    components: HashMap<ID, RefCell<Gate>>,

    // actual input components reside in the `components`
    inputs: HashMap<String, ID>,
    pub outputs: HashSet<ID>,
    last_id: ID,
    pub exec_queue: VecDeque<ID>,
    active: bool,
    clk: Option<ID>,
}

impl BCircuit {
    pub fn new() -> BCircuit {
        let mut c = BCircuit {
            component_definitions: HashMap::new(),
            components: HashMap::new(),
            // label_map: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashSet::new(),
            last_id: 0,
            exec_queue: VecDeque::new(),
            active: false,
            clk: None,
        };
        bootstrap_ckt(&mut c);

        c
    }
    pub fn clear_circuit(&mut self) {
        self.components.clear();
        self.inputs.clear();
        self.outputs.clear();
        self.last_id = 0;
        self.exec_queue.clear();
        self.clk = None;
    }
    pub fn components(&self) -> &HashMap<ID, RefCell<Gate>> {
        return &self.components;
    }
    pub fn set_component_label(&mut self, id: ID, lab: &str) -> Result<(), String> {
        let c = self.components.get(&id);
        match c {
            Some(c) => {
                c.borrow_mut().label.replace_range(std::ops::RangeFull, lab);
                self.graph_act(set_expressions, &vec![id]);
                Ok(())
            }
            None => Err(format!("Component with id_{} not found", id)),
        }
    }
    pub fn clock(&mut self, id: ID) {
        self.clk = Some(id)
    }
    pub fn pulse_clock(&mut self) {
        if self.clk.is_none() {
            return;
        }
        let clk = self.clk.unwrap();
        let curr_state = self.state(clk).unwrap();
        self.set_component_state(clk, !curr_state).unwrap();
        self.set_component_state(clk, curr_state).unwrap();
    }
    pub fn refresh(&mut self) {
        self.graph_act(update_component_state, &self.all_inputs_and_states());
    }
    pub fn power_on(&mut self) {
        self.active = true;
        self.graph_act(power_on_component, &self.all_inputs_and_states());
        println!("POWER ON");
    }

    fn graph_act(&mut self, runnable: ComponentActor, inits: &Vec<ID>) {
        // traverse in breadth-first fashion, starting from received `inits`
        // and calls the specified function until queue vacates.
        for k in inits {
            self.exec_queue.push_back(*k);
        }

        let mut iters = 0;
        // todo: puede ser mejor que esto.
        let max_iters = 5000;

        while !self.exec_queue.is_empty() && iters < max_iters {
            let id = self.exec_queue.pop_front().unwrap();
            let mut k = self.components.get(&id).unwrap().borrow_mut();
            runnable(&mut k, &self.components, &mut self.exec_queue);
            iters += 1;
        }
    }
    pub fn define_gate(&mut self, p: ComponentDefParams) {
        self.component_definitions
            .insert(String::from(p.name.clone()), ComponentDefParams::from(p));
    }

    fn make_component(&mut self, typ: &str, label: &str) -> Result<Gate, String> {
        let def = self.component_definitions.get(typ);
        if def.is_none() {
            return Err(format!("Component type not defined {}", typ));
        }
        let mut def = def.unwrap().clone();
        def.label = label.to_string();
        return Ok(Gate::from_params(def));
    }
    pub fn add_component(&mut self, typ: &str, label: &str) -> Result<ID, String> {
        let id = self.new_id();
        let comp = self.make_component(typ, label);
        match comp {
            Ok(mut c) => {
                c.id = id;
                self.components.insert(id, RefCell::new(c));
                return Ok(id);
            }
            Err(e) => Err(e),
        }
    }
    pub fn remove_component(&mut self, id: ID) -> Result<(), String> {
        if !self.components.contains_key(&id) {
            return Err(format!("Component with id {} not found", id));
        }
        // algo:
        // 1. Remove its entry from all its input sources. They don't need state update.
        // Disconnect it from all its output receivers, then update them.
        // Remove this comp. from map.

        // 1.
        let srcs: Vec<(PIN, ID)> = {
            let c = self.components.get(&id).unwrap().borrow();
            c.input_pin_sources.iter().cloned().enumerate().collect()
        };
        for (pin, inp_src_id) in srcs {
            if inp_src_id == NULL {
                continue;
            }
            match self.disconnect(id, pin, inp_src_id) {
                Err(e) => return Err(format!("Could not remove id_{} completely: {}", id, e)),
                Ok(()) => {}
            }
        }

        let orlist: Vec<(ID, PIN)> = self
            .components
            .get(&id)
            .unwrap()
            .borrow()
            .output_recvlist
            .iter()
            .cloned()
            .collect();
        for (rec_id, pin) in &orlist {
            // calling do_disconnect to save on bfs. We can run it once at the end
            match self.do_disconnect(*rec_id, *pin, id) {
                Err(e) => return Err(format!("Could not remove id_{} completely: {}", id, e)),
                Ok(()) => {}
            }
        }
        self.components.remove(&id);
        self.graph_act(
            update_component_state,
            &orlist.iter().map(|a| a.0).collect(),
        );
        Ok(())
    }
    pub fn add_input(&mut self, label: &str, init_val: bool) -> ID {
        // todo: unique label enforcement for inputs and clocked comps
        let mut inp = Gate::make_input(label, init_val);
        let id = self.new_id();
        inp.id = id;
        self.components.insert(id, RefCell::new(inp));
        self.inputs.insert(label.to_string(), id);
        return id;
    }
    pub fn get_component(&self, id: &ID) -> Option<&RefCell<Gate>> {
        return self.components.get(id);
    }
    pub fn set_component_state(&mut self, id: ID, val: bool) -> Result<(), String> {
        if !self.active {
            return Err(format!("Power on the circuit first!"));
        }

        let childs = match self.components.get(&id) {
            //todo: might panic if component is connected to itself
            Some(k) => {
                k.borrow_mut().set_state(val);
                for (id, pin) in k.borrow().get_output_receivers() {
                    self.components
                        .get(id)
                        .unwrap()
                        .borrow_mut()
                        .set_pin_val(*pin, val);
                }
                k.borrow()
                    .get_output_receivers()
                    .iter()
                    .map(|a| a.0)
                    .collect()
            }
            None => return Err(format!("No element with id_{}", id)),
        };

        self.graph_act(update_component_state, &childs);
        Ok(())
    }
    pub fn connect(&mut self, receiver_id: ID, pin: PIN, emitter_id: ID) -> Result<(), String> {
        let res = self.do_connect(receiver_id, pin, emitter_id);
        if res.is_err() {
            return res;
        }
        self.graph_act(update_component_state, &vec![receiver_id]);
        self.graph_act(set_expressions, &vec![receiver_id]);
        Ok(())
    }
    fn do_connect(&mut self, receiver_id: ID, pin: PIN, emitter_id: ID) -> Result<(), String> {
        if !self.components.contains_key(&receiver_id) {
            return Err(format!("No receiver with id {}", receiver_id));
        }
        if !self.components.contains_key(&emitter_id) {
            return Err(format!("No emitter with id {}", emitter_id));
        }
        if receiver_id == emitter_id {
            return Err(format!(
                "id_{} Self connection is not supported, please use a buffer.",
                receiver_id
            ));
        }

        let receiver = self.components.get(&receiver_id).unwrap();
        let emitter = self.components.get(&emitter_id).unwrap();

        if pin >= receiver.borrow().num_inputs() {
            return Err(format!(
                "There are only {} pins, can't access {}",
                receiver.borrow().num_inputs(),
                pin
            ));
        }

        // algo:
        // - emitter stores (id, pin) to be used to propagate its state
        // - receiver's input `pin` is set to emitter's current state and
        // - stores emitter.id as input source at that pin
        // - receiver propagates its new state further

        // todo: If emitter and receiver are same, we'd need to handle this
        // separately.

        emitter.borrow_mut().link_output_receiver(receiver_id, pin);

        receiver
            .borrow_mut()
            .set_input_pin_connection(pin, &emitter.borrow())
    }
    pub fn disconnect(&mut self, receiver_id: ID, pin: PIN, emitter_id: ID) -> Result<(), String> {
        let res = self.do_disconnect(receiver_id, pin, emitter_id);
        if res.is_err() {
            return res;
        }
        self.graph_act(update_component_state, &vec![receiver_id]);
        self.graph_act(set_expressions, &vec![receiver_id]);
        Ok(())
    }
    fn do_disconnect(&mut self, receiver_id: ID, pin: PIN, emitter_id: ID) -> Result<(), String> {
        if !self.components.contains_key(&receiver_id) {
            return Err(format!("No receiver with id {}", receiver_id));
        }
        if !self.components.contains_key(&emitter_id) {
            return Err(format!("No emitter with id {}", emitter_id));
        }

        let receiver = self.components.get(&receiver_id).unwrap();
        let emitter = self.components.get(&emitter_id).unwrap();

        let unlink_result = emitter
            .borrow_mut()
            .unlink_output_receiver(receiver_id, pin);
        if unlink_result.is_err() {
            return unlink_result;
        }

        receiver.borrow_mut().clear_input_pin_connection(pin)
    }
    pub fn track_output(&mut self, comp_id: ID) -> bool {
        if self.components.get(&comp_id).is_none() {
            return false;
        }
        self.outputs.insert(comp_id);
        true
    }
    pub fn compile(&mut self) {
        // 1. we generate boolean expression for each component
        // in the circuit.
        // 2. we check for dangerous loops.
        // todo
    }
    pub fn all_inputs_and_states(&self) -> Vec<ID> {
        // excludes clk
        // todo: add clocked components (circuit states)
        let mut q = Vec::<ID>::new();
        for (id, c) in &self.components {
            let ct = c.borrow().comp_type;
            if ct == CompType::Input || ct == CompType::Sequential {
                if c.borrow().id == self.clk.unwrap_or(NULL) {
                    continue;
                }
                q.push(*id);
            }
        }
        q
    }
    pub fn gen_truth_table(&mut self) -> Table<char> {
        let mut t = Table::<char>::new();
        let inps = self.all_inputs_and_states();
        let outs = &self.outputs.iter().map(|v| *v).collect::<Vec<ID>>();

        let mut cols = inps
            .iter()
            .map(|id| self.get_component(id).unwrap().borrow().state_expr.clone())
            .collect::<Vec<String>>();
        cols.sort();
        let sorted_out_labels = &mut outs
            .iter()
            .map(|id| self.get_component(id).unwrap().borrow().label.clone())
            .collect::<Vec<String>>();
        sorted_out_labels.sort();
        cols.append(sorted_out_labels);

        t.set_columns(cols).unwrap();
        for ct in bitwise_counter(inps.len()) {
            let idx = t.add_row();
            let mut i = 0;
            for id in &inps {
                let mut in_el = self.components.get(id).unwrap().borrow_mut();
                in_el.set_state(ct[i]);
                t.set_val_at(
                    idx,
                    &in_el.state_expr.as_str(),
                    (in_el.state as u8 + '0' as u8) as char,
                );

                i += 1;
            }
            self.graph_act(update_component_state, &inps);
            self.pulse_clock();
            for id in &self.outputs {
                let out_el = self.components.get(id).unwrap().borrow_mut();
                t.set_val_at(
                    idx,
                    out_el.label.as_str(),
                    (out_el.state as u8 + '0' as u8) as char,
                );
            }
        }
        return t;
    }
    pub fn state(&self, id: ID) -> Option<bool> {
        match self.components.get(&id) {
            Some(a) => {
                let val = a.borrow().state.to_string().chars().nth(0).unwrap();
                if val == 'f' {
                    Some(false)
                } else {
                    Some(true)
                }
            }
            None => None,
        }
    }
    pub fn new_id(&mut self) -> ID {
        self.last_id += 1;
        return self.last_id;
    }
}

mod tests {
    use crate::{
        circuit::BCircuit,
        types::{CLOCK_PIN, NULL},
    };
    #[test]
    fn add_component() {
        let mut c = BCircuit::new();
        c.power_on();
        let a = c.add_component("AND", "A").unwrap();
        let b = c.add_component("JK", "Q1").unwrap();
        assert!(c.get_component(&a).is_some());
        assert!(c.get_component(&b).is_some());
        assert!(c.get_component(&a).unwrap().borrow().name == "AND");

        assert!(c
            .add_component("GIBBERISH", "A")
            .is_err_and(|a| a.contains("GIBBERISH")));
    }

    #[test]
    fn connect_disconnect() {
        let mut c = BCircuit::new();
        c.power_on();
        let a = c.add_input("A", true);
        let clk = c.add_input("CLK", false);
        let q1 = c.add_component("JK", "Q1").unwrap();

        assert!(c.connect(q1, CLOCK_PIN, clk).is_ok());
        assert!(c.components.get(&q1).unwrap().borrow().input_pin_sources[CLOCK_PIN] == clk);
        assert!(c
            .components
            .get(&clk)
            .unwrap()
            .borrow()
            .output_recvlist
            .contains(&(q1, CLOCK_PIN)));

        assert!(c.connect(q1, 1, a).is_ok());
        assert!(c.components.get(&q1).unwrap().borrow().input_pin_sources[1] == a);
        assert!(c
            .components
            .get(&a)
            .unwrap()
            .borrow()
            .output_recvlist
            .contains(&(q1, 1)));

        assert!(c.connect(q1, 2, a).is_ok());
        assert!(c.components.get(&q1).unwrap().borrow().input_pin_sources[2] == a);
        assert!(c
            .components
            .get(&a)
            .unwrap()
            .borrow()
            .output_recvlist
            .contains(&(q1, 2)));

        assert!(c.connect(q1, 3, a).is_err());
        assert_eq!(
            c.components.get(&a).unwrap().borrow().output_recvlist.len(),
            2
        );

        // disconnect input pin
        assert!(c.disconnect(q1, 1, a).is_ok());
        assert!(c.components.get(&q1).unwrap().borrow().input_pin_sources[1] == NULL);
        assert!(!c
            .components
            .get(&a)
            .unwrap()
            .borrow()
            .output_recvlist
            .contains(&(q1, 1)));

        // disconnect CLOCK
        assert!(c.disconnect(q1, CLOCK_PIN, clk).is_ok());
        assert!(c.components.get(&q1).unwrap().borrow().input_pin_sources[CLOCK_PIN] == NULL);
        assert!(!c
            .components
            .get(&a)
            .unwrap()
            .borrow()
            .output_recvlist
            .contains(&(q1, CLOCK_PIN)));

        // connecting to self
        let q2 = c.add_component("JK", "Q2").unwrap();
        assert!(c.connect(q2, 1, q2).is_err());
    }

    #[test]
    fn state_change_ripple() {
        // State of a component changes through two ways:
        // 1. A parent component changed value at a pin and bfs state update was performed.
        // 2. A parent component was removed/disconnected.
        let mut c = BCircuit::new();
        c.power_on();
        let i = c.add_input("A", false);
        let n1 = c.add_component("NOT", "B").unwrap();
        let n2 = c.add_component("NOT", "C").unwrap();

        c.connect(n1, 1, i).unwrap();
        c.connect(n2, 1, n1).unwrap();

        assert_eq!(c.state(i).unwrap(), false);
        assert_eq!(c.state(n1).unwrap(), true);
        assert_eq!(c.state(n2).unwrap(), false);

        c.set_component_state(i, true).unwrap();

        assert_eq!(c.state(i).unwrap(), true);
        assert_eq!(c.state(n1).unwrap(), false);
        assert_eq!(c.state(n2).unwrap(), true);

        // if an and gate has its input disconnected, it turns off (not ON)
        c.remove_component(i).unwrap();
        assert_eq!(c.state(n1).unwrap(), false);
        assert_eq!(c.state(n2).unwrap(), true);

        c.disconnect(n2, 1, n1).unwrap();
        assert_eq!(c.state(n2).unwrap(), false);
    }

    #[test]
    fn clock_change_ripple() {
        let mut c = BCircuit::new();

        // 2 bit async up counter
        let one = c.add_input("1", true);
        let clk = c.add_input("clk", false);
        let q = c.add_component("JK", "Q1").unwrap();
        let qq = c.add_component("JK", "Q2").unwrap();
        let n = c.add_component("NOT", "!Q1").unwrap();

        c.connect(q, 1, one).unwrap();
        c.connect(q, 2, one).unwrap();

        c.connect(qq, 1, one).unwrap();
        c.connect(qq, 2, one).unwrap();

        c.connect(q, CLOCK_PIN, clk).unwrap();
        c.connect(qq, CLOCK_PIN, n).unwrap();
        c.connect(n, 1, q).unwrap();

        c.clock(clk);

        // THIS TEST IS NONDETERMINISTIC!??
        c.power_on();

        assert_eq!((c.state(q).unwrap(), c.state(qq).unwrap()), (false, false));
        c.pulse_clock();
        assert_eq!((c.state(q).unwrap(), c.state(qq).unwrap()), (true, false));
        c.pulse_clock();
        assert_eq!((c.state(q).unwrap(), c.state(qq).unwrap()), (false, true));
        c.pulse_clock();
        assert_eq!((c.state(q).unwrap(), c.state(qq).unwrap()), (true, true));
    }

    #[test]
    fn remove_component() {
        let mut c = BCircuit::new();
        c.power_on();
        let i = c.add_input("A", false);
        let n1 = c.add_component("NOT", "B").unwrap();
        let n2 = c.add_component("NOT", "C").unwrap();

        c.connect(n1, 1, i).unwrap();
        c.connect(n2, 1, n1).unwrap();
        assert_eq!(c.state(n2).unwrap(), false);
        c.remove_component(n1).unwrap();
        assert_eq!(c.components.get(&n1).is_none(), true);
        assert_eq!(c.state(n2).unwrap(), true);
    }

    #[test]
    fn sample_comb_ckt() {
        let mut c = BCircuit::new();
        let i1 = c.add_input("A", !false);
        let i2 = c.add_input("B", false);
        let i3 = c.add_input("C", true);

        let n1 = c.add_component("NOT", "not1").unwrap();
        let n2 = c.add_component("NOT", "not2").unwrap();
        let or = c.add_component("OR", "F1").unwrap();

        let a1 = c.add_component("AND", "AB").unwrap();
        let a2 = c.add_component("AND", "").unwrap();
        let a3 = c.add_component("AND", "").unwrap();

        c.connect(n1, 1, i1).unwrap();
        c.connect(n2, 1, i2).unwrap();

        c.connect(a1, 1, i1).unwrap();
        c.connect(a1, 2, i2).unwrap();

        c.connect(a2, 1, n1).unwrap();
        c.connect(a2, 2, n2).unwrap();

        c.connect(a3, 1, a2).unwrap();
        c.connect(a3, 2, i3).unwrap();

        c.connect(or, 1, a3).unwrap();
        c.connect(or, 2, a1).unwrap();

        c.track_output(or);
        c.compile();
        c.power_on();
        println!("{}", c.components.get(&or).unwrap().borrow().state_expr);
    }
    #[test]
    fn latch() {
        let mut c = BCircuit::new();
        c.power_on();

        let q = c.add_component("NAND", "q").unwrap();
        let nq = c.add_component("NAND", "nq").unwrap();
        let is = c.add_input("S", true);
        let ir = c.add_input("R", false);

        c.connect(q, 1, is).unwrap();
        c.connect(nq, 2, ir).unwrap();

        c.connect(q, 2, nq).unwrap();
        c.connect(nq, 1, q).unwrap();
    }
}
