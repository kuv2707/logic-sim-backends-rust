use std::collections::{HashMap, HashSet};

use bsim_engine::{
    circuit::BCircuit,
    types::{ID, NULL},
};
use pratt_parser::{lexer::Lexer, nodes::NodeType, pratt::Parser};

pub struct ModuleCreationData {
    pub inputs: HashMap<String, ID>,
    pub outputs: HashMap<String, ID>,
    pub contents: HashSet<ID>,
}

pub fn get_logic_unit(ckt: &mut BCircuit, s: &str) -> Result<ModuleCreationData, String> {
    let mut lex = Lexer::new();
    let toks = lex.lexify(s);
    let mut p = Parser::new(toks);
    let nodes = p.parse();

    let mut inp_ids: HashMap<String, ID> = HashMap::new();
    let mut out_ids: HashMap<String, ID> = HashMap::new();
    let mut all_ids = HashSet::new();
    for node in nodes {
        // node.traverse(0);
        let output = traverse_add(&node, ckt, &mut inp_ids, &mut out_ids, &mut all_ids);
        match output {
            Ok(oid) => {}
            Err(e) => return Err(e),
        }
    }

    if out_ids.len() == 0 {
        return Err("No outputs created!".into());
    }

    // todo: replace with clock receiver
    // inp_ids.insert("CLK".into(), NULL);
    Ok(ModuleCreationData {
        inputs: inp_ids,
        outputs: out_ids,
        contents: all_ids,
    })
}

fn traverse_add(
    root: &NodeType,
    ckt: &mut BCircuit,
    inp_ids: &mut HashMap<String, ID>,
    out_ids: &mut HashMap<String, ID>,
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
                    let parent = match traverse_add(&childs[1], ckt, inp_ids, out_ids, all_ids) {
                        Err(e) => return Err(e),
                        Ok(k) => k,
                    };
                    out_ids.insert(name.to_string(), bfr);
                    ckt.connect(bfr, 1, parent).unwrap();
                    Ok(bfr)
                }
                '!' => {
                    // not
                    let n = ckt.add_component("NOT", "").unwrap();
                    all_ids.insert(n);
                    let parent = match traverse_add(&childs[0], ckt, inp_ids, out_ids, all_ids) {
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
                    let left = match traverse_add(&childs[0], ckt, inp_ids, out_ids, all_ids) {
                        Err(e) => return Err(e),
                        Ok(k) => k,
                    };
                    let right = match traverse_add(&childs[1], ckt, inp_ids, out_ids, all_ids) {
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
            match inp_ids.get(sym) {
                Some(id) => Ok(*id),
                None => {
                    // add a buffer with this label
                    let bfr = ckt.add_component("BFR", sym).unwrap();
                    all_ids.insert(bfr);
                    inp_ids.insert(sym.to_string(), bfr);
                    Ok(bfr)
                }
            }
        }
    }
}
