use std::collections::{BTreeMap, HashSet};

use crate::table::Table;

const DONT_CARE: char = '_';

pub fn qm_simplify_many(t: &Table<char>, inps: &Vec<&str>, outs: &Vec<&str>) -> Vec<String> {
    // returns simplified expressions of vars in outs (from corresponding
    // column in truth table) in terms of input vars
    return outs
        .iter()
        .map(|out| qm_simplify_one(t, inps, out))
        .collect();
}

pub fn qm_simplify_one(t: &Table<char>, inps: &Vec<&str>, out: &str) -> String {
    let mut grp = grp_by_ones(t, inps, out);
    let mut unpaired = HashSet::<(u16, usize)>::new();
    let mut prime_implicants = HashSet::new();

    loop {
        let mut nxt_grp_table: BTreeMap<u16, Vec<(Vec<u16>, Vec<char>)>> = BTreeMap::new();
        // mark all rows as unpaired
        for g_no in grp.keys() {
            for i in 0..grp.get(g_no).unwrap().len() {
                unpaired.insert((*g_no, i));
            }
        }

        for curr_grp_no in grp.keys() {
            let curr_grp_rows = grp.get(curr_grp_no).unwrap();
            match grp.range((curr_grp_no + 1)..).next() {
                Some(nxt_grp_entry) => {
                    let nxt_grp_no = nxt_grp_entry.0;
                    let nxt_grp_rows = grp.get(nxt_grp_no).unwrap();

                    let nxx = form_nxt_table_grp_from_rows(
                        curr_grp_rows,
                        nxt_grp_rows,
                        &mut unpaired,
                        *curr_grp_no,
                        *nxt_grp_no,
                    );
                    if nxx.len() > 0 {
                        nxt_grp_table.insert(*curr_grp_no, nxx);
                    }
                }
                None => {
                    // means curr_grp_no is the last group
                    break;
                }
            }
        }

        for k in &unpaired {
            prime_implicants.insert(grp.get(&k.0).unwrap()[k.1].clone());
        }
        unpaired.clear();

        if nxt_grp_table.len() == 0 {
            for grps in grp.values() {
                for k in grps {
                    prime_implicants.insert(k.clone());
                }
            }
            break;
        }
        grp = nxt_grp_table;
    }
    //todo: find essential prime implicants

    let mut simplified_exp = String::new();
    prime_implicants
        .iter()
        .map(|v| {
            let mut exp = Vec::new();
            let ins = &v.1;
            for i in 0..inps.len() {
                if ins[i] == DONT_CARE {
                    continue;
                }
                exp.push(format!(
                    "{}{}",
                    if ins[i] == '0' { "!" } else { "" },
                    inps[i]
                ));
            }
            exp.join(".")
        })
        .collect::<Vec<String>>()
        .join("+")
}

fn form_nxt_table_grp_from_rows(
    curr_grp_rows: &Vec<(Vec<u16>, Vec<char>)>,
    nxt_grp_rows: &Vec<(Vec<u16>, Vec<char>)>,
    unpaired: &mut HashSet<(u16, usize)>,
    curr_grp_no: u16,
    nxt_grp_no: u16,
) -> Vec<(Vec<u16>, Vec<char>)> {
    let mut nxt_table_rows = Vec::new();
    for i in 0..curr_grp_rows.len() {
        for j in 0..nxt_grp_rows.len() {
            if differ_by_one_entry(&curr_grp_rows[i].1, &nxt_grp_rows[j].1) {
                let mut nxt_entry_minterms = curr_grp_rows[i]
                    .0
                    .iter()
                    .chain(nxt_grp_rows[j].0.iter())
                    .cloned()
                    .collect::<Vec<u16>>();
                nxt_entry_minterms.sort();
                nxt_entry_minterms.dedup();
                let nxt_entry_vals = collate_rows(&curr_grp_rows[i].1, &nxt_grp_rows[j].1);
                nxt_table_rows.push((nxt_entry_minterms, nxt_entry_vals));
                unpaired.remove(&(curr_grp_no, i));
                unpaired.remove(&(nxt_grp_no, j));
            }
        }
    }
    nxt_table_rows
}

fn differ_by_one_entry(a: &Vec<char>, b: &Vec<char>) -> bool {
    if a.len() != b.len() {
        panic!("Not equal lens when comparing");
    }
    let mut diff = false;
    for i in 0..a.len() {
        if a[i] == b[i] {
            continue;
        }
        if diff {
            return false;
        };
        diff = true;
    }
    diff
}

fn collate_rows(a: &Vec<char>, b: &Vec<char>) -> Vec<char> {
    if a.len() != b.len() {
        panic!("Not equal lens when collating");
    }
    let mut k = vec![' '; a.len()];
    for i in 0..a.len() {
        if a[i] == b[i] {
            k[i] = a[i];
        } else {
            k[i] = DONT_CARE;
        }
    }
    k
}

fn grp_by_ones(
    t: &Table<char>,
    inps: &Vec<&str>,
    out: &str,
) -> BTreeMap<u16, Vec<(Vec<u16>, Vec<char>)>> {
    let mut grps = BTreeMap::<u16, Vec<(Vec<u16>, Vec<char>)>>::new();
    // for each row where output is 1, put it in the group corresp to number of 1's
    // in its inputs
    for i in 0..t.rows.len() {
        let tval = t.get_val_at(i, out);
        if *tval == '1' {
            let num_ones = inps.iter().fold(0, |v, w| {
                let tval = t.get_val_at(i, *w);
                if *tval == '1' {
                    v + 1
                } else {
                    v
                }
            });
            let irow = inps.iter().map(|v| t.get_val_at(i, *v).clone()).collect();
            if !grps.contains_key(&num_ones) {
                grps.insert(num_ones, Vec::new());
            }
            grps.get_mut(&num_ones)
                .unwrap()
                .push((vec![i as u16], irow));
        }
    }
    grps
}

mod tests {
    use crate::table::Table;

    use super::qm_simplify_many;

    #[test]
    fn qm() {
        let mut tt = Table::<char>::new();
        tt.set_columns(
            vec!["A", "B", "C", "D", "a", "b", "c", "d", "e", "f", "g"]
                .iter()
                .map(|v| v.to_string())
                .collect(),
        )
        .unwrap();
        let rows = vec![
            vec!['0', '0', '0', '0', '1', '1', '1', '1', '1', '1', '0'], // 0
            vec!['0', '0', '0', '1', '0', '1', '1', '0', '0', '0', '0'], // 1
            vec!['0', '0', '1', '0', '1', '1', '0', '1', '1', '0', '1'], // 2
            vec!['0', '0', '1', '1', '1', '1', '1', '1', '0', '0', '1'], // 3
            vec!['0', '1', '0', '0', '0', '1', '1', '0', '0', '1', '1'], // 4
            vec!['0', '1', '0', '1', '1', '0', '1', '1', '0', '1', '1'], // 5
            vec!['0', '1', '1', '0', '1', '0', '1', '1', '1', '1', '1'], // 6
            vec!['0', '1', '1', '1', '1', '1', '1', '0', '0', '0', '0'], // 7
            vec!['1', '0', '0', '0', '1', '1', '1', '1', '1', '1', '1'], // 8
            vec!['1', '0', '0', '1', '1', '1', '1', '1', '0', '1', '1'], // 9
        ];

        tt.set_rows(rows).unwrap();
        let outs = vec!["a", "b", "c", "d", "e", "f", "g"];
        let res = qm_simplify_many(&tt, &vec!["A", "B", "C", "D"], &outs);
        for i in 0..outs.len() {
            println!("{} = {};", outs[i], res[i]);
        }
    }
}
