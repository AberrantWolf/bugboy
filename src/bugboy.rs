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
mod tracelog;

use std::cell::RefCell;
use std::env;
use std::io;
use std::path::Path;
use std::rc::Rc;

use gb_cpu::DmgCpu;
use gb_hw_bus::HardwareBus;
use gb_mem::{MemoryController, RamAddress};
use gb_rom::GbRom;

use tracelog::TraceLog;

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
        let mut max_ticks = 100_000;
        //let mut buffer = String::new();
        //let stdin = io::stdin();
        let mut log: Vec<TraceLog> = Vec::new();
        loop {
            match self.cpu.borrow_mut().tick(&mut log) {
                Ok(_) => (),
                Err(e) => {
                    println!("ERROR: {}", e);
                    break;
                }
            }

            if self.cpu.borrow().is_stopped() {
                println!("Game was stopped");
                break;
            }

            max_ticks -= 1;
            if max_ticks == 0 {
                println!("Reached the end of timer.");
                break;
            }

            // match stdin.read_line(&mut buffer) {
            //     Ok(_) => continue,
            //     Err(e) => {
            //         println!("Error reading stdin: {}", e);
            //         return;
            //     }
            // }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Too few arguments specified.");
        return;
    }

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
