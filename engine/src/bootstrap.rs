use crate::{circuit::BCircuit, components::ComponentDefParams, types::CompType};

pub(crate) fn bootstrap_ckt(c: &mut BCircuit) {
    c.define_gate(ComponentDefParams {
        name: "Input".into(),
        label: String::new(),
        comp_type: CompType::Input,
        eval: |_, old| old,
        default_inputs: 0,
        symbol: "".into(),
    });
    c.define_gate(ComponentDefParams {
        name: "NAND".into(),
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return !(v[1] && v[2]);
            // return !(v.iter().fold(true, |a, b| a && *b));
        },
        default_inputs: 2,
        symbol: "!.".into(),
    });

    c.define_gate(ComponentDefParams {
        name: "AND".into(),
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return v[1] && v[2];
            // return v.iter().fold(true, |a, b| a && *b);
        },
        default_inputs: 2,
        symbol: ".".into(),
    });

    c.define_gate(ComponentDefParams {
        name: "OR".into(),
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return v[1] || v[2];
            // return v.iter().fold(false, |a, b| a || *b);
        },
        default_inputs: 2,
        symbol: "+".into(),
    });

    c.define_gate(ComponentDefParams {
        name: "XOR".into(),
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return v[1] != v[2];
        },
        default_inputs: 2,
        symbol: "+".into(),
    });

    c.define_gate(ComponentDefParams {
        name: "NOT".into(),
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return !v[1];
        },
        default_inputs: 1,
        symbol: "!".into(),
    });

    c.define_gate(ComponentDefParams {
        name: "BFR".into(), // buffer
        label: String::new(),
        comp_type: CompType::Combinational,
        eval: |v, _| {
            return v[1];
        },
        default_inputs: 1,
        symbol: "".into(),
    });

    c.define_gate(ComponentDefParams {
        name: "JK".into(),
        label: String::new(),
        comp_type: CompType::Sequential,
        eval: |v, q| {
            let j = v[1];
            let k = v[2];
            (j && !q) || (!k && q)
        },
        default_inputs: 2,
        symbol: "JK".into(),
    });
}
