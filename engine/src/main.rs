use circuit::BCircuit;

mod circuit;
mod components;
mod table;
mod types;
mod utils;

fn main() {
    let mut circuit = BCircuit::new();

    let n1 = circuit.add_component("AND");

    let i1 = circuit.register_input("A", true);
    let i2 = circuit.register_input("B", true);
    let i3 = circuit.register_input("C", false);

    circuit.connect(n1, 1, i1).unwrap();
    circuit.connect(n1, 2, i2).unwrap();

    let not1 = circuit.add_component("NOT");
    circuit.connect(not1, 1, i3).unwrap();

    let n2 = circuit.add_component("AND");
    circuit.connect(n2, 1, n1).unwrap();
    circuit.connect(n2, 2, not1).unwrap();

    circuit.track_output(n2, "Y");
    circuit.compile();
    println!("");
    circuit.run();

    println!(
        "{}",
        circuit.get_component(&n2).unwrap().borrow().state_expr
    );
    println!("{}", circuit.state(n2).unwrap());
}
