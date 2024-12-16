use circuit::BCircuit;

mod circuit;
mod clock_manager;
mod components;
mod table;
mod types;
mod utils;

fn sync_counter() {
    let mut circuit = BCircuit::new();

    // 4 bit sync. counter
    let n1 = circuit.add_component("JK", "QA");
    let n2 = circuit.add_component("JK", "QB");
    let n3 = circuit.add_component("JK", "QC");
    let n4 = circuit.add_component("JK", "QD");

    let a1 = circuit.add_component("AND", "a1");
    let a2 = circuit.add_component("AND", "a2");
    let a3 = circuit.add_component("AND", "a3");
    let a4 = circuit.add_component("AND", "a4");
    let a5 = circuit.add_component("AND", "a5");
    let a6 = circuit.add_component("AND", "a6");

    let o1 = circuit.add_component("OR", "o1");
    let o2 = circuit.add_component("OR", "o1");
    let o3 = circuit.add_component("OR", "o1");

    let not0 = circuit.add_component("NOT", "not1");
    let not1 = circuit.add_component("NOT", "not1");
    let not2 = circuit.add_component("NOT", "not1");
    let not3 = circuit.add_component("NOT", "not1");
    let not4 = circuit.add_component("NOT", "not1");

    let m = circuit.register_input("M", !true);
    let clk = circuit.register_input("clk", false);
    let one = circuit.register_input("1", true);

    circuit.connect(n1, 4, clk).unwrap();
    circuit.connect(n2, 4, clk).unwrap();
    circuit.connect(n3, 4, clk).unwrap();
    circuit.connect(n4, 4, clk).unwrap();

    circuit.connect(not0, 1, m).unwrap();

    circuit.connect(n1, 1, one).unwrap();
    circuit.connect(n1, 2, one).unwrap();
    circuit.connect(not1, 1, n1).unwrap();

    circuit.connect(a1, 1, not0).unwrap();
    circuit.connect(a1, 2, n1).unwrap();
    
    circuit.connect(a4, 1, m).unwrap();
    circuit.connect(a4, 2, not1).unwrap();
    
    circuit.connect(o1, 1, a1).unwrap();
    circuit.connect(o1, 2, a4).unwrap();
    circuit.connect(n2, 1, o1).unwrap();
    circuit.connect(n2, 2, o1).unwrap();
    circuit.connect(not2, 1, n2).unwrap();
    
    circuit.connect(a2, 1, a1).unwrap();
    circuit.connect(a2, 2, n2).unwrap();
    
    circuit.connect(a5, 1, a4).unwrap();
    circuit.connect(a5, 2, not2).unwrap();

    circuit.connect(o2, 1, a2).unwrap();
    circuit.connect(o2, 2, a5).unwrap();
    circuit.connect(n3, 1, o2).unwrap();
    circuit.connect(n3, 2, o2).unwrap();
    circuit.connect(not3, 1, n3).unwrap();
    
    circuit.connect(a3, 1, a2).unwrap();
    circuit.connect(a3, 2, n3).unwrap();
    
    circuit.connect(a6, 1, a5).unwrap();
    circuit.connect(a6, 2, not3).unwrap();
    
    
    
    circuit.connect(o3, 1, a3).unwrap();
    circuit.connect(o3, 2, a6).unwrap();
    circuit.connect(n4, 1, o3).unwrap();
    circuit.connect(n4, 2, o3).unwrap();
    circuit.connect(not4, 1, n4).unwrap();

    // circuit.set_component_val(n4, false);
    // circuit.set_component_val(n3, false);
    // circuit.set_component_val(n2, true);
    // circuit.set_component_val(n1, true);

    circuit.compile();
    circuit.power_on();
    let mut val = true;
    for _ in 0..32 {
        circuit.set_component_val(clk, val);
        val = !val;
        if val {
            continue;
        }
        println!(
            "{} {} {} {} ",
            circuit.state(n4).unwrap(),
            circuit.state(n3).unwrap(),
            circuit.state(n2).unwrap(),
            circuit.state(n1).unwrap(),
            // circuit.state(clk).unwrap(),
        );
    }
}


fn main() {
    return sync_counter();
    let mut c = BCircuit::new();
    let not = c.add_component("NOT", "");
    let i1 = c.register_input("A", false);
    let or = c.add_component("OR", "");

    c.connect(not, 1, i1).unwrap();
    c.connect(or, 1, not).unwrap();
    c.power_on();
    println!("{}", c.state(or).unwrap());
}