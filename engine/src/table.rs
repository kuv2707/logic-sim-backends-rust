use std::{collections::HashMap, fmt};

// a rudimentary implementation
pub struct Table<T> {
    cols: Vec<String>,
    col_idx: HashMap<String, usize>,
    pub rows: Vec<Vec<T>>,
}

impl<T: Default + Clone> Table<T> {
    pub fn new() -> Table<T> {
        return Table {
            cols: Vec::new(),
            col_idx: HashMap::new(),
            rows: Vec::new(),
        };
    }
    pub fn set_columns(&mut self, cols: Vec<String>) -> Result<(), String> {
        self.cols = cols;
        self.col_idx.clear();
        // duplicate column values will cause undefined behaviour
        for i in 0..self.cols.len() {
            self.col_idx.insert(self.cols[i].clone(), i);
        }
        Ok(())
    }
    pub fn set_rows(&mut self, rows: Vec<Vec<T>>) -> Result<(), String> {
        self.rows = rows;
        Ok(())
    }
    pub fn add_row(&mut self) -> usize {
        self.rows.push(vec![T::default(); self.cols.len()]);
        self.rows.len() - 1
    }
    pub fn set_val_at(&mut self, i: usize, j: &str, val: T) {
        self.rows[i][*(self.col_idx.get(j).unwrap())] = val;
    }
    pub fn get_val_at(& self, i: usize, j: &str) -> &T {
        &self.rows[i][*(self.col_idx.get(j).unwrap())]
    }
}

impl<T: fmt::Display> fmt::Display for Table<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pad = self
            .cols
            .iter()
            .map(|c| {
                let width = c.len() + 2 + 1; // 2 spaces left and right of the label text
                let ret = (" ".repeat(width / 2), " ".repeat(width - width / 2));
                ret
            })
            .collect::<Vec<(String, String)>>();
        let total_width = pad.iter().fold(0, |a, b| a + b.0.len() + b.1.len() + 2);

        writeln!(f, "|{}|", "¯".repeat(total_width - 1))?;
        write!(f, "|")?;
        for i in 0..pad.len() {
            write!(f, "  \x1b[33m{}\x1b[0m  |", self.cols[i])?;
        }
        writeln!(f)?;
        writeln!(f, "|{}|", "-".repeat(total_width - 1))?;
        for row in &self.rows {
            write!(f, "|")?;
            for i in 0..pad.len() {
                write!(f, "{}{}{}|", pad[i].0, row[i], pad[i].1)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "|{}|", "_".repeat(total_width - 1))?;
        Ok(())
    }
}

pub fn bitwise_counter(bits: usize) -> impl Iterator<Item = Vec<bool>> {
    let total_combs = 1 << bits;
    (0..total_combs).map(move |n| {
        (0..bits)
            .map(|i| (1 << i & n) > 0)
            .rev()
            .collect::<Vec<bool>>()
    })
}
