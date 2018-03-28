use std::vec::Vec;

use gb_opcodes::OpCodes;

pub enum MemChangeDest {
    RegA,
    RegB,
    RegC,
    RegD,
    RegE,
    RegF,
    RegH,
    RegL,
    Mem(u16),
}

pub struct MemChange {
    dest: MemChangeDest,
    value: u8,
}

pub struct TraceLog {
    opcode: OpCodes,
    changes: Vec<MemChange>,
}
