#[derive(Debug)]
pub struct HardwareBus {
    cycles: u64,
}

impl HardwareBus {
    pub fn new() -> Self {
        HardwareBus { cycles: 0u64 }
    }

    pub fn sync(&mut self, count: u64) {
        // TODO: Update all the other things
        self.cycles == count;
    }
}
