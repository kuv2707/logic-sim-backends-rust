// #![allow(unused)]

use circuit::BCircuit;
use quine_mccluskey::qm_simplify_many;
use table::Table;
use types::CLOCK_PIN;

mod circuit;
mod clock_manager;
mod components;
mod quine_mccluskey;
mod table;
mod types;
mod utils;

fn aoc24_24() {
    let inp = r#""#;

    let mut c = BCircuit::new();
    let mut toks = inp.split("\n");
    let mut conns = false;
    while let Some(tok) = toks.next() {
        if tok.len() == 0 {
            conns = true;
            continue;
        }
        let toks = tok.split(" ").collect::<Vec<&str>>();
        if conns {
            let (i1, _) = c.add_component_if_label_absent("BFR", toks[0]);
            let (i2, _) = c.add_component_if_label_absent("BFR", toks[2]);
            let gt = c.add_component(toks[1], "");
            c.connect(gt, 1, i1).unwrap();
            c.connect(gt, 2, i2).unwrap();

            let (o1, _) = c.add_component_if_label_absent("BFR", toks[4]);
            c.connect(o1, 1, gt).unwrap();
            if toks[4].starts_with("z") {
                c.track_output(o1);
            }
        } else {
            let lab = &toks[0][0..toks[0].len() - 1];
            let initval = toks[1].parse::<i32>().unwrap();
            // println!("{} {}", lab, initval);
            c.add_input(lab, if initval == 1 { true } else { false });
        }
    }
    c.compile();
    c.power_on();
    let mut num = 0;
    for id in &c.outputs {
        let comp = c.get_component(id).unwrap().borrow();
        let val = if comp.state { 1_u64 } else { 0 };
        let bitposn = comp.label[1..].parse::<i32>().unwrap();
        num = num | (val << bitposn);
    }
    println!("{}", num);
}

fn main() {
    // return aoc24_24();
}
