pub mod circuit;
pub mod clock_manager;
pub mod components;
pub mod quine_mccluskey;
pub mod table;
pub mod types;
pub mod utils;
mod bootstrap;

// todo:
// - save/load from file
// - higher order components - enc/dec/mux etc

// #[cfg(test)]
// mod tests {
//     use circuit::BCircuit;
//     use quine_mccluskey::qm_simplify_many;
//     use table::Table;
//     use types::CLOCK_PIN;

//     use super::*;

    


//     #[test]
//     fn test_truth_table() {
//         let mut c = sample_comb_ckt();
//         println!("{}", c.gen_truth_table());
//     }

//     fn sample_comb_ckt() -> BCircuit {
//         let mut c = BCircuit::new();
//         let i1 = c.add_input("AAAAA", !false);
//         let i2 = c.add_input("B", false);
//         let i3 = c.add_input("C", true);

//         let n1 = c.add_component("NOT", "not1");
//         let n2 = c.add_component("NOT", "not2");
//         let or = c.add_component("OR", "F1");

//         let a1 = c.add_component("AND", "AB");
//         let a2 = c.add_component("AND", "");
//         let a3 = c.add_component("AND", "");

//         c.connect(n1, 1, i1).unwrap();
//         c.connect(n2, 1, i2).unwrap();

//         c.connect(a1, 1, i1).unwrap();
//         c.connect(a1, 2, i2).unwrap();

//         c.connect(a2, 1, n1).unwrap();
//         c.connect(a2, 2, n2).unwrap();

//         c.connect(a3, 1, a2).unwrap();
//         c.connect(a3, 2, i3).unwrap();

//         c.connect(or, 1, a3).unwrap();
//         c.connect(or, 2, a1).unwrap();

//         c.track_output(or);
//         c.compile();
//         c.power_on();
//         c
//     }

//     #[test]
//     fn sync_sttable() {
//         let mut c = BCircuit::new();
//         let ff = c.add_component("JK", "Q");
//         let i1 = c.add_input("JJ", true);
//         let i2 = c.add_input("KK", true);
//         let clk = c.add_input("clk", false);
//         c.clock(clk);
//         c.connect(ff, 1, i1).unwrap();
//         c.connect(ff, 2, i2).unwrap();
//         c.connect(ff, CLOCK_PIN, clk).unwrap();
//         c.track_output(ff);
//         c.compile();
//         c.power_on();
//         let mut val = false;
//         for _ in 0..9 {
//             c.set_component_state(clk, val);
//             val = !val;
//             println!("{}", c.state(ff).unwrap());
//         }
//         println!("-------");
//         println!("{}", c.gen_truth_table());
//     }

//     #[test]

//     fn sync_counter() {
//         let mut circuit = BCircuit::new();
//         // 4 bit sync. counter
//         let n1 = circuit.add_component("JK", "QD");
//         let n2 = circuit.add_component("JK", "QC");
//         let n3 = circuit.add_component("JK", "QB");
//         let n4 = circuit.add_component("JK", "QA");

//         let a1 = circuit.add_component("AND", "a1");
//         let a2 = circuit.add_component("AND", "a2");
//         let a3 = circuit.add_component("AND", "a3");
//         let a4 = circuit.add_component("AND", "a4");
//         let a5 = circuit.add_component("AND", "a5");
//         let a6 = circuit.add_component("AND", "a6");

//         let o1 = circuit.add_component("OR", "o1");
//         let o2 = circuit.add_component("OR", "o1");
//         let o3 = circuit.add_component("OR", "o1");

//         let not0 = circuit.add_component("NOT", "not1");
//         let not1 = circuit.add_component("NOT", "not2");
//         let not2 = circuit.add_component("NOT", "not3");
//         let not3 = circuit.add_component("NOT", "not4");
//         let not4 = circuit.add_component("NOT", "not5");

//         let m = circuit.add_input("M", !true);
//         let clk = circuit.add_input("clk", false);
//         let one = circuit.add_input("1", true);

//         circuit.connect(n1, CLOCK_PIN, clk).unwrap();
//         circuit.connect(n2, CLOCK_PIN, clk).unwrap();
//         circuit.connect(n3, CLOCK_PIN, clk).unwrap();
//         circuit.connect(n4, CLOCK_PIN, clk).unwrap();

//         circuit.connect(not0, 1, m).unwrap();

//         circuit.connect(n1, 1, one).unwrap();
//         circuit.connect(n1, 2, one).unwrap();
//         circuit.connect(not1, 1, n1).unwrap();

//         circuit.connect(a1, 1, not0).unwrap();
//         circuit.connect(a1, 2, n1).unwrap();

//         circuit.connect(a4, 1, m).unwrap();
//         circuit.connect(a4, 2, not1).unwrap();

//         circuit.connect(o1, 1, a1).unwrap();
//         circuit.connect(o1, 2, a4).unwrap();
//         circuit.connect(n2, 1, o1).unwrap();
//         circuit.connect(n2, 2, o1).unwrap();
//         circuit.connect(not2, 1, n2).unwrap();

//         circuit.connect(a2, 1, a1).unwrap();
//         circuit.connect(a2, 2, n2).unwrap();

//         circuit.connect(a5, 1, a4).unwrap();
//         circuit.connect(a5, 2, not2).unwrap();

//         circuit.connect(o2, 1, a2).unwrap();
//         circuit.connect(o2, 2, a5).unwrap();
//         circuit.connect(n3, 1, o2).unwrap();
//         circuit.connect(n3, 2, o2).unwrap();
//         circuit.connect(not3, 1, n3).unwrap();

//         circuit.connect(a3, 1, a2).unwrap();
//         circuit.connect(a3, 2, n3).unwrap();

//         circuit.connect(a6, 1, a5).unwrap();
//         circuit.connect(a6, 2, not3).unwrap();

//         circuit.connect(o3, 1, a3).unwrap();
//         circuit.connect(o3, 2, a6).unwrap();
//         circuit.connect(n4, 1, o3).unwrap();
//         circuit.connect(n4, 2, o3).unwrap();
//         circuit.connect(not4, 1, n4).unwrap();

//         circuit.set_component_state(n4, false);
//         circuit.set_component_state(n3, true);
//         circuit.set_component_state(n2, false);
//         circuit.set_component_state(n1, true);

//         circuit.track_output(n1);
//         circuit.track_output(n2);
//         circuit.track_output(n3);
//         circuit.track_output(n4);

//         circuit.clock(clk);

//         circuit.compile();
//         circuit.power_on();
//         let mut val = false;
//         println!("------------------");
//         for _ in 0..32 {
//             circuit.set_component_state(clk, val);
//             val = !val;
//             if !val {
//                 continue;
//             }
//             println!(
//                 "-> {} {} {} {}",
//                 circuit.state(n4).unwrap(),
//                 circuit.state(n3).unwrap(),
//                 circuit.state(n2).unwrap(),
//                 circuit.state(n1).unwrap(),
//                 // circuit.state(clk).unwrap(),
//             );
//         }
//         // println!("{}", circuit.gen_truth_table());
//         // plot(&circuit);
//     }

//     #[test]

//     fn qm() {
//         let mut tt = Table::<char>::new();
//         tt.set_columns(
//             vec!["A", "B", "C", "D", "F1"]
//                 .iter()
//                 .map(|v| v.to_string())
//                 .collect(),
//         )
//         .unwrap();
//         let rows = vec![
//             vec!['0', '0', '0', '0', '0'],
//             vec!['0', '0', '0', '1', '0'],
//             vec!['0', '0', '1', '0', '1'],
//             vec!['0', '0', '1', '1', '1'],
//             vec!['0', '1', '0', '0', '0'],
//             vec!['0', '1', '0', '1', '0'],
//             vec!['0', '1', '1', '0', '0'],
//             vec!['0', '1', '1', '1', '0'],
//             vec!['1', '0', '0', '0', '1'],
//             vec!['1', '0', '0', '1', '0'],
//             vec!['1', '0', '1', '0', '0'],
//             vec!['1', '0', '1', '1', '0'],
//             vec!['1', '1', '0', '0', '1'],
//             vec!['1', '1', '0', '1', '1'],
//             vec!['1', '1', '1', '0', '1'],
//             vec!['1', '1', '1', '1', '1'],
//         ];
//         tt.set_rows(rows).unwrap();
//         let outs = vec!["F1"];
//         let res = qm_simplify_many(&tt, &vec!["A", "B", "C", "D"], &outs);
//         for i in 0..outs.len() {
//             println!("{} = {}", outs[i], res[i]);
//         }
//     }
//     #[test]
//     fn asynchronous_ckt() {
//         let mut ckt = BCircuit::new();
//         let jk1 = ckt.add_component("JK", "A");
//         let jk2 = ckt.add_component("JK", "B");

//         let one = ckt.add_input("one", true);
//         let clk = ckt.add_input("CLK", false);

//         ckt.connect(jk1, 1, one).unwrap();
//         ckt.connect(jk1, 2, one).unwrap();

//         ckt.connect(jk2, 1, one).unwrap();
//         ckt.connect(jk2, 2, one).unwrap();

//         ckt.connect(jk1, CLOCK_PIN, clk).unwrap();
//         ckt.connect(jk2, CLOCK_PIN, jk1).unwrap();

//         ckt.compile();
//         ckt.power_on();

//         ckt.set_component_state(clk, true);
//         ckt.set_component_state(clk, !true);
//         println!("{} {}", ckt.state(jk1).unwrap(), ckt.state(jk2).unwrap());

//         ckt.set_component_state(clk, true);
//         ckt.set_component_state(clk, !true);
//         println!("{} {}", ckt.state(jk1).unwrap(), ckt.state(jk2).unwrap());

//         ckt.set_component_state(clk, true);
//         ckt.set_component_state(clk, !true);
//         println!("{} {}", ckt.state(jk1).unwrap(), ckt.state(jk2).unwrap());
//     }
// }
