use crate::{circuit::BCircuit, components::ComponentDefParams, types::CompType};

pub(crate) fn bootstrap_ckt(c: &mut BCircuit) {
    c.define_gate(ComponentDefParams {
        name: "NAND".to_string(),
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return !(v[1] && v[2]);
            // return !(v.iter().fold(true, |a, b| a && *b));
        },
        default_inputs: 2,
        symbol: "!.".to_string(),
    });

    c.define_gate(ComponentDefParams {
        name: "AND".to_string(),
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return v[1] && v[2];
            // return v.iter().fold(true, |a, b| a && *b);
        },
        default_inputs: 2,
        symbol: ".".to_string(),
    });

    c.define_gate(ComponentDefParams {
        name: "OR".to_string(),
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return v[1] || v[2];
            // return v.iter().fold(false, |a, b| a || *b);
        },
        default_inputs: 2,
        symbol: "+".to_string(),
    });

    c.define_gate(ComponentDefParams {
        name: "XOR".to_string(),
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return v[1] != v[2];
        },
        default_inputs: 2,
        symbol: "+".to_string(),
    });

    c.define_gate(ComponentDefParams {
        name: "NOT".to_string(),
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return !v[1];
        },
        default_inputs: 1,
        symbol: "!".to_string(),
    });

    c.define_gate(ComponentDefParams {
        name: "BFR".to_string(), // buffer
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return v[1];
        },
        default_inputs: 1,
        symbol: "".to_string(),
    });

    c.define_gate(ComponentDefParams {
        name: "JK".to_string(),
        label: String::new(),
        comp_type: CompType::Sequential,
        eval: |v, q| {
            let j = v[1];
            let k = v[2];
            (j && !q) || (!k && q)
        },
        default_inputs: 2,
        symbol: "JK".to_string(),
    });
}
