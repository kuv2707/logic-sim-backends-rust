use std::collections::{HashMap, HashSet};

use bsim_engine::{
    circuit::BCircuit,
    types::{ID, NULL},
};
use pratt_parser::{lexer::Lexer, nodes::NodeType, pratt::Parser};

pub struct ModuleCreationData {
    pub inputs: Vec<ID>,
    pub outputs: Vec<ID>,
    pub contents: HashSet<ID>,
}

pub fn get_logic_unit(ckt: &mut BCircuit, s: &str) -> Result<ModuleCreationData, String> {
    let mut lex = Lexer::new();
    let toks = lex.lexify(s);
    let mut p = Parser::new(toks);
    let nodes = p.parse();

    let mut inps: HashMap<String, ID> = HashMap::new();
    let mut out_ids: Vec<ID> = Vec::new();
    let mut all_ids = HashSet::new();
    for node in nodes {
        let output = traverse_add(&node, ckt, &mut inps, &mut all_ids);
        match output {
            Ok(oid) => {
                out_ids.push(oid);
            }
            Err(e) => return Err(e),
        }
    }

    if out_ids.len() == 0 {
        return Err("No outputs created!".into());
    }

    let mut inp_ids: Vec<i32> = inps.values().map(|e| *e).collect();
    // todo: replace with clock receiver
    inp_ids.insert(0, NULL);
    Ok(ModuleCreationData {
        inputs: inp_ids,
        outputs: out_ids,
        contents: all_ids,
    })
}

fn traverse_add(
    root: &NodeType,
    ckt: &mut BCircuit,
    inps: &mut HashMap<String, ID>,
    all_ids: &mut HashSet<ID>,
) -> Result<ID, String> {
    match root {
        NodeType::Operator(sym, childs) => {
            match sym {
                '=' => {
                    let name = match &childs[0] {
                        NodeType::Operator(_, _) => return Err("LHS should be a label".into()),
                        NodeType::Literal(sym, _) => sym,
                    };
                    let bfr = ckt.add_component("BFR", name).unwrap();
                    all_ids.insert(bfr);
                    let parent = match traverse_add(&childs[1], ckt, inps, all_ids) {
                        Err(e) => return Err(e),
                        Ok(k) => k,
                    };

                    ckt.connect(bfr, 1, parent).unwrap();
                    Ok(bfr)
                }
                '!' => {
                    // not
                    let n = ckt.add_component("NOT", "").unwrap();
                    all_ids.insert(n);
                    let parent = match traverse_add(&childs[0], ckt, inps, all_ids) {
                        Err(e) => return Err(e),
                        Ok(k) => k,
                    };
                    // `parent` should be a valid component id
                    // and there should be no problem connecting
                    ckt.connect(n, 1, parent).unwrap();
                    Ok(n)
                }
                '+' | '.' | '*' => {
                    let comp = ckt
                        .add_component(
                            match sym {
                                '+' => "OR",
                                '.' => "AND",
                                '*' => "XOR",
                                _ => "UNKNOWN", // todo: make compiler not need this
                            },
                            "",
                        )
                        .unwrap();
                    all_ids.insert(comp);
                    let left = match traverse_add(&childs[0], ckt, inps, all_ids) {
                        Err(e) => return Err(e),
                        Ok(k) => k,
                    };
                    let right = match traverse_add(&childs[1], ckt, inps, all_ids) {
                        Err(e) => return Err(e),
                        Ok(k) => k,
                    };
                    ckt.connect(comp, 1, left).unwrap();
                    ckt.connect(comp, 2, right).unwrap();
                    Ok(comp)
                }
                _ => Err(format!("Undefined operator {}", sym)),
            }
        }
        NodeType::Literal(sym, _) => {
            match inps.get(sym) {
                Some(id) => Ok(*id),
                None => {
                    // add a buffer with this label
                    let bfr = ckt.add_component("BFR", sym).unwrap();
                    all_ids.insert(bfr);
                    inps.insert(sym.to_string(), bfr);
                    Ok(bfr)
                }
            }
        }
    }
}
