pub struct ClockManager {
    curr: bool,
    last: bool,
}

impl ClockManager {
    pub fn new() -> ClockManager {
        return ClockManager {
            curr: false,
            last: false,
        };
    }
    pub fn reset_clock_hist(&mut self) {
        self.curr = false;
        self.last = false;
    }
    pub fn clock_triggered(&self) -> bool {
        self.last == false && self.curr
    }
    pub fn push(&mut self, val: bool) {
        if val == self.curr {
            return;
        }
        self.last = self.curr;
        self.curr = val
    }
}
