pub struct Buffer {
    vals: Vec<f64>,
    pub len: usize,
    pushes: usize,
    pub index: usize,
    pub sum: f64,
}

impl Buffer {
    pub fn new(len: usize) -> Self {
        Buffer {
            vals: vec![0.0; len],
            len,
            pushes: 0,
            index: 0,
            sum: 0.0,
        }
    }

    pub fn push(&mut self, val: f64) {
        if self.pushes >= self.len {
            self.sum -= self.vals[self.index];
        }

        self.sum += val;
        self.vals[self.index] = val;
        self.pushes += 1;
        self.index += 1;
        if self.index >= self.len {
            self.index = 0;
        }
    }

    pub fn qpush(&mut self, val: f64) {
        self.vals[self.index] = val;
        self.index += 1;
        if self.index >= self.len {
            self.index = 0;
        }
    }

    pub fn get(&self, ind: usize) -> f64 {
        self.vals[(self.index + self.len - 1 + ind) % self.len]
    }

    pub fn avg(&self) -> f64 {
        self.sum / self.len as f64
    }
}
