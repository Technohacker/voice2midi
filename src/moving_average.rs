pub struct MovingAverage {
    buffer: Vec<i32>,
    position: usize
}

impl MovingAverage {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![0; size],
            position: 0
        }
    }

    pub fn feed(&mut self, value: i32) {
        self.buffer[self.position] = value;

        self.position += 1;
        self.position %= self.buffer.len();
    }

    pub fn average(&self) -> i32 {
        self.buffer.iter().fold(0, |s, &x| s + x) / self.buffer.len() as i32
    }
}