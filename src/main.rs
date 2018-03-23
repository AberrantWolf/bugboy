#![feature(custom_attribute)]
#![feature(slice_patterns)]
#[macro_use]
extern crate enum_primitive;
extern crate num;

mod gb_cpu;
mod gb_mem;
mod gb_opcodes;
mod gb_hw_bus;
mod gb_rom;

use std::cell::RefCell;
use std::env;
use std::path::Path;
use std::rc::Rc;

use gb_cpu::DmgCpu;
use gb_hw_bus::HardwareBus;
use gb_mem::{MemoryController, RamAddress};
use gb_rom::GbRom;

struct DmgBoy {
    cpu: Rc<RefCell<DmgCpu>>,
    mc: Rc<RefCell<MemoryController>>,
    bus: Rc<RefCell<HardwareBus>>,
}

impl DmgBoy {
    fn new(rom: GbRom) -> Self {
        let bus = Rc::new(RefCell::new(HardwareBus::new()));
        let mc = Rc::new(RefCell::new(MemoryController::new(rom)));
        let cpu = Rc::new(RefCell::new(DmgCpu::new(bus.clone(), mc.clone())));
        DmgBoy {
            bus: bus,
            mc: mc,
            cpu: cpu,
        }
    }

    fn run(&mut self) {
        let mut ticks = 100000;
        loop {
            self.cpu.borrow_mut().tick();
            if self.cpu.borrow().is_stopped() {
                println!("Game was stopped");
                break;
            }

            ticks -= 1;
            if ticks == 0 {
                println!("Reached the end of timer.");
                break;
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = Path::new(&args[1]);

    let mut absolute_path = match env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            println!("Unable to get current environment path: {}", e);
            panic!();
        }
    };
    absolute_path.push(path);

    let rom = match GbRom::new(absolute_path) {
        Ok(r) => r,
        Err(e) => {
            println!("ERROR loading ROM: {}", e);
            return;
        }
    };

    let mut bugboy = DmgBoy::new(rom);
    {
        let mc = bugboy.mc.borrow();
        let addr = RamAddress::new(0x0100);
        println!("Hello, world! {}", mc.read(addr) as char);
    }

    bugboy.run();

    println!("Done.");
}
