use circuit::Circuit;

mod circuit;
mod components;
mod types;

fn main() {
    let mut circuit = Circuit::new();

    let n1 = circuit.add_component("AND");
    let i1 = circuit.register_input("A", true);
    let i2 = circuit.register_input("B", false);

    circuit.connect(n1, 1, i1).unwrap();
    circuit.connect(n1, 2, i2).unwrap();
    circuit.track_output(n1, "Y");
    circuit.run();
    println!("{}", circuit.get_component(&n1).unwrap().borrow());
    println!("{}", circuit.state(n1).unwrap());
}
