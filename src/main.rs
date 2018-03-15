mod gb_cpu;
mod gb_mem;
mod gb_opcodes;

use gb_cpu::GbCpu;

fn main() {
    let cpu = GbCpu::new();
    println!("Hello, world!");
}
