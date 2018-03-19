mod gb_cpu;
mod gb_mem;
mod gb_opcodes;
mod gb_hw_bus;

use std::rc::Rc;
use std::cell::RefCell;

use gb_cpu::DmgCpu;
use gb_mem::{MemoryController, RamAddress};
use gb_hw_bus::HardwareBus;

struct DmgBoy {
    cpu: Rc<RefCell<DmgCpu>>,
    mc: Rc<RefCell<MemoryController>>,
    bus: Rc<RefCell<HardwareBus>>,
}

impl DmgBoy {
    fn new() -> Self {
        let bus = Rc::new(RefCell::new(HardwareBus::new()));
        let mc = Rc::new(RefCell::new(MemoryController::new()));
        let cpu = Rc::new(RefCell::new(DmgCpu::new(bus.clone(), mc.clone())));
        DmgBoy {
            bus: bus,
            mc: mc,
            cpu: cpu,
        }
    }

    fn run(&mut self) {
        loop {
            self.cpu.borrow_mut().tick();
        }
    }
}

fn main() {
    let mut bugboy = DmgBoy::new();
    {
        let mc = bugboy.mc.borrow();
        let addr = RamAddress::new(0x0100);
        println!("Hello, world! {}", mc.read(addr) as char);
    }

    bugboy.run();
}
