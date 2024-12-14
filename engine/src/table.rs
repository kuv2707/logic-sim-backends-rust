use std::{collections::HashMap, fmt};

// a rudimentary implementation
pub struct Table<T> {
    cols: Vec<String>,
    col_idx: HashMap<String, usize>,
    rows: Vec<Vec<T>>,
}

impl<T: Default + Clone> Table<T> {
    pub fn new() -> Table<T> {
        return Table {
            cols: Vec::new(),
            col_idx: HashMap::new(),
            rows: Vec::new(),
        };
    }
    pub fn reset_columns(&mut self, cols: Vec<String>) {
        self.cols = cols;
        self.rows.clear();
        self.col_idx.clear();
        for i in 0..self.cols.len() {
            self.col_idx.insert(self.cols[i].clone(), i);
        }
    }
    pub fn add_row(&mut self) -> usize {
        self.rows.push(vec![T::default(); self.cols.len()]);
        self.rows.len() - 1
    }
    pub fn set_val_at(&mut self, i: usize, j: &str, val: T) {
        self.rows[i][*(self.col_idx.get(j).unwrap())] = val;
    }
}

impl<T: fmt::Display> fmt::Display for Table<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, " {}", "-".repeat(6*self.cols.len()-1))?;
        write!(f,"|")?;
        for col in &self.cols {
            write!(f, "  {}  |", col)?;
        }
        writeln!(f)?;
        writeln!(f, "|{}", "-----|".repeat(self.cols.len()))?;
        for row in &self.rows {
            write!(f,"|")?;
            for item in row {
                write!(f, "  {}  |", item)?;
            }
            writeln!(f)?;
        }
        writeln!(f, " {}", "-".repeat(6*self.cols.len()-1))?;
        Ok(())
    }
}

pub fn bitwise_counter(bits: usize) -> impl Iterator<Item = Vec<bool>> {
    let total_combs = 1 << bits;
    (0..total_combs).map(move |n| (0..bits).map(|i| (1 << i & n) > 0).rev().collect::<Vec<bool>>())
}
