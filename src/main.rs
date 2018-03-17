pub mod gb_cpu;
pub mod gb_mem;
mod gb_opcodes;

use gb_cpu::GbCpu;
use gb_mem::RamAddress;

fn main() {
    let cpu = GbCpu::new();
    let mc = cpu.get_memory_controller();
    let addr = RamAddress::new(0x0100);

    println!("Hello, world! {}", mc.borrow().read(addr) as char);
}
