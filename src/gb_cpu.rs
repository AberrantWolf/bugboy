use std::rc::Rc;
use std::cell::RefCell;

use num::FromPrimitive;

use gb_hw_bus::HardwareBus;
use gb_mem::{MemoryController, RamAddress, decrement_16, increment_16, IE_ADDR};
use gb_opcodes::{OpCodes, SecondOpAction, SecondOpRegister, SecondOpType};

use tracelog::{MemChange, TraceLog};

const ZERO_FLAG: u8 = 1 << 7;
const SUBT_FLAG: u8 = 1 << 6;
const HALF_CARRY_FLAG: u8 = 1 << 5;
const CARRY_FLAG: u8 = 1 << 4;

const VBLANK_IF: u8 = 1;
const LCDC_IF: u8 = 1 << 1;
const TIMER_OVERFLOW_IF: u8 = 1 << 2;
const SERIAL_IO_COMPLETE_IF: u8 = 1 << 3;
const P10_P13_TERM_NEG_EDGE_IF: u8 = 1 << 4;

#[derive(Debug)]
pub struct DmgCpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: RamAddress,
    pc: RamAddress,

    ime: bool, // interrupt master enabled
    halt: bool,
    stop: bool,

    clock: u64,

    mc: Rc<RefCell<MemoryController>>,
    bus: Rc<RefCell<HardwareBus>>,
}

impl DmgCpu {
    pub fn new(bus: Rc<RefCell<HardwareBus>>, mc: Rc<RefCell<MemoryController>>) -> Self {
        DmgCpu {
            a: 0u8,
            b: 0u8,
            c: 0u8,
            d: 0u8,
            e: 0u8,
            f: 0u8,
            h: 0u8,
            l: 0u8,
            sp: RamAddress::new(0xFFFEu16),
            pc: RamAddress::new(0x0100u16),

            ime: true,
            halt: false,
            stop: false,

            clock: 0u64,

            mc: mc,
            bus: bus,
        }
    }

    fn sync_hardware_bus(&mut self) {
        self.bus.borrow_mut().sync(self.clock);
    }

    pub fn get_memory_controller(&self) -> Rc<RefCell<MemoryController>> {
        self.mc.clone()
    }

    fn read_pc_mem_and_increment(&mut self) -> u8 {
        let result = self.mc.borrow().read(self.pc.post_inc(1));
        self.clock += 4;
        self.sync_hardware_bus();
        result
    }

    fn get_carry_value(&self) -> u8 {
        (self.f & CARRY_FLAG) >> 5
    }

    // Creating addresses by combining registers (&c)
    fn make_bc_address(&self) -> RamAddress {
        RamAddress::new((self.b as u16) << 8 | self.c as u16)
    }

    fn make_de_address(&self) -> RamAddress {
        RamAddress::new((self.d as u16) << 8 | self.e as u16)
    }

    fn make_hl_address(&self) -> RamAddress {
        RamAddress::new((self.h as u16) << 8 | self.l as u16)
    }

    fn make_ffc_address(&self) -> RamAddress {
        RamAddress::new(0xFF00 | self.c as u16)
    }

    fn make_ffn_address(&mut self) -> RamAddress {
        let n = self.read_pc_mem_and_increment() as u16;
        RamAddress::new(0xFF00u16 | n)
    }

    fn read_address_pair(&mut self) -> (u8, u8) {
        let low = self.read_pc_mem_and_increment();
        let high = self.read_pc_mem_and_increment();
        (low, high)
    }

    fn write_address_pair(&mut self, high: &mut u8, low: &mut u8) {
        let pair = self.read_address_pair();
        *low = pair.0;
        *high = pair.1;
    }

    fn make_nn_address(&mut self) -> RamAddress {
        let pair = self.read_address_pair();
        let low = pair.0 as u16;
        let high = pair.1 as u16;
        RamAddress::new((high << 8) | low)
    }

    // Setting the flag helpers
    fn set_flag(&mut self, mask: u8) {
        self.f |= mask;
    }

    fn reset_flag(&mut self, mask: u8) {
        self.f &= !mask;
    }

    fn set_flag_conditional(&mut self, mask: u8, test: bool) {
        if test {
            self.set_flag(mask);
        } else {
            self.reset_flag(mask);
        }
    }

    fn add(&mut self, a: u8, b: u8) -> u8 {
        let r = self.add_no_zcheck(a, b);
        self.set_flag_conditional(ZERO_FLAG, r as u8 == 0);
        r
    }

    fn add_no_zcheck(&mut self, a: u8, b: u8) -> u8 {
        let r = a.overflowing_add(b);
        let hr = (a & 0x0F).wrapping_add(b & 0x0F);

        self.set_flag_conditional(HALF_CARRY_FLAG, hr > 0x0F);
        self.set_flag_conditional(CARRY_FLAG, r.1); // it wrapped around
        self.reset_flag(SUBT_FLAG);
        r.0
    }

    fn add_with_carry(&mut self, a: u8, b: u8) -> u8 {
        let carry = self.get_carry_value();
        // this increments the dest first, might set flags but they
        // would be overwritten? is this correct behaviour?
        let t = self.add(a, carry);
        self.add(t, b)
    }

    fn subtract(&mut self, a: u8, b: u8) -> u8 {
        let r = self.subtract_no_zcheck(a, b);
        self.set_flag_conditional(ZERO_FLAG, r == 0);
        r
    }

    fn subtract_no_zcheck(&mut self, a: u8, b: u8) -> u8 {
        let r = a.overflowing_sub(b);
        let hr = (a & 0x0F).wrapping_sub(b & 0x0F);

        self.set_flag_conditional(HALF_CARRY_FLAG, hr > 0x0F);
        self.set_flag_conditional(CARRY_FLAG, r.1); // it wrapped around
        self.set_flag(SUBT_FLAG);
        r.0
    }

    fn subtract_with_carry(&mut self, a: u8, b: u8) -> u8 {
        let carry = self.get_carry_value();
        let n = b + carry;
        self.subtract(a, n)
    }

    fn set_logic_flags(&mut self, result: u8, set_half_carry: bool) {
        self.f &= !(SUBT_FLAG | CARRY_FLAG);
        if set_half_carry {
            self.f |= HALF_CARRY_FLAG;
        } else {
            self.f &= !HALF_CARRY_FLAG;
        }

        self.set_flag_conditional(ZERO_FLAG, result == 0);
    }

    fn add_to_u16(&mut self, a: u8, b: u16) -> u16 {
        self.add(a, b as u8);
        a as u16 + b
    }

    // increment/decrement
    fn increment(&mut self, byte: &mut u8) {
        self.set_flag_conditional(HALF_CARRY_FLAG, (*byte & 0x0F) == 0x0F);
        self.reset_flag(SUBT_FLAG);
        *byte = (*byte).wrapping_add(1);
        self.set_flag_conditional(ZERO_FLAG, *byte == 0);
    }

    fn decrement(&mut self, byte: &mut u8) {
        self.set_flag(SUBT_FLAG);
        self.set_flag_conditional(HALF_CARRY_FLAG, (*byte & 0x0F) == 0x00);
        *byte = (*byte).wrapping_sub(1);
        self.set_flag_conditional(ZERO_FLAG, *byte == 0);
    }

    // rotation

    // rotate left through self, but still copies leftmost bit to carry
    fn do_rlc(&mut self, value: u8) -> u8 {
        let result = value.rotate_left(1);

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1000_0000) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    // rotate left through carry (and carry into rightmost bit)
    fn do_rl(&mut self, value: u8) -> u8 {
        let carry = self.get_carry_value();
        let result = (value << 1) | carry;

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1000_0000) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    // rotate right through self, but copy rightmost bit to carry
    fn do_rrc(&mut self, value: u8) -> u8 {
        let result = value.rotate_right(1);

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    // rotate right through the carry
    fn do_rr(&mut self, value: u8) -> u8 {
        let carry = self.get_carry_value();
        let result = (value >> 1) | (carry << 7);

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1) == 0b1);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    // shift operations
    fn do_sla(&mut self, value: u8) -> u8 {
        let result = value << 1;

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1000_0000) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    fn do_sra(&mut self, value: u8) -> u8 {
        let msb = value & 0b1000_0000;
        let result = value >> 1 | msb;

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    fn do_srl(&mut self, value: u8) -> u8 {
        let result = value >> 1;

        self.set_flag_conditional(CARRY_FLAG, (value & 0b1) > 0);
        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG);

        result
    }

    fn do_swap(&mut self, value: u8) -> u8 {
        let result = (value & 0x0F) << 4 | (value & 0xF0) >> 4;

        self.set_flag_conditional(ZERO_FLAG, result == 0);
        self.reset_flag(SUBT_FLAG | HALF_CARRY_FLAG | CARRY_FLAG);

        result
    }

    // program flow
    fn read_pc_as_address(&mut self) -> u16 {
        let mc = self.mc.borrow();
        let low = mc.read(self.pc.post_inc(1)) as u16;
        let high = mc.read(self.pc.post_inc(1)) as u16;
        high << 8 | low
    }

    fn do_jump_conditional(&mut self, test: bool) {
        let dest = self.read_pc_as_address();
        if test {
            self.pc.set(dest);
        }
    }

    fn do_jump_relative_conditional(&mut self, test: bool) {
        let offset = self.mc.borrow().read(self.pc.post_inc(1));
        self.sync_hardware_bus();

        if test {
            self.pc.inc(offset as i8 as u16);
        }
    }

    fn push_address_parts(&mut self, high: u8, low: u8) -> Result<(), String> {
        match self.mc.borrow_mut().write(self.sp.dec(1), low) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }
        self.sync_hardware_bus();
        match self.mc.borrow_mut().write(self.sp.dec(1), high) {
            Ok(_) => (),
            Err(err) => return Err(err),
        }
        self.sync_hardware_bus();
        Ok(())
    }

    fn push_address_u16(&mut self, addr: u16) -> Result<(), String> {
        let high = (addr & 0xFF00 >> 8) as u8;
        let low = (addr & 0x00FF) as u8;
        return self.push_address_parts(high, low);
    }

    fn push_address(&mut self, addr: RamAddress) -> Result<(), String> {
        return self.push_address_u16(addr.get());
    }

    fn pop_address_parts(&mut self) -> (u8, u8) {
        let high = self.mc.borrow().read(self.sp.post_inc(1));
        self.sync_hardware_bus();
        let low = self.mc.borrow().read(self.sp.post_inc(1));
        self.sync_hardware_bus();
        (high, low)
    }

    fn pop_address_u16(&mut self) -> u16 {
        let parts = self.pop_address_parts();
        (parts.0 as u16) << 8 | (parts.1 as u16)
    }

    fn do_call_conditional(&mut self, test: bool) -> Result<(), String> {
        let dest = self.read_pc_as_address();

        if test {
            let addr = self.pc.get();
            self.pc.set(dest);
            return self.push_address_u16(addr);
        }

        Ok(())
    }

    fn do_return_conditional(&mut self, test: bool) {
        if test {
            let addr = self.pop_address_u16();
            self.pc.set(addr);
        }
    }

    // multibyte ops
    fn hand_rotate_shift_op(&mut self, value: u8, op: SecondOpAction) -> u8 {
        match op {
            SecondOpAction::RLC => return self.do_rlc(value),
            SecondOpAction::RL => return self.do_rl(value),
            SecondOpAction::RRC => return self.do_rrc(value),
            SecondOpAction::RR => return self.do_rr(value),
            SecondOpAction::SLA => return self.do_sla(value),
            SecondOpAction::SRA => return self.do_sra(value),
            SecondOpAction::SRL => return self.do_srl(value),
            SecondOpAction::SWAP => return self.do_swap(value),
        }
    }

    fn decode_and_execute_cb_op(&mut self, sop: u8) -> Result<(), String> {
        let op_type: SecondOpType = SecondOpType::from_u8(sop);
        let action = SecondOpAction::from_u8(sop);
        let register = SecondOpRegister::from_u8(sop);

        let bit_mask = 1 << (action as u8);

        match op_type {
            SecondOpType::BIT_CHECK => {
                let reg_value = match register {
                    SecondOpRegister::A => self.a,
                    SecondOpRegister::B => self.b,
                    SecondOpRegister::C => self.c,
                    SecondOpRegister::D => self.d,
                    SecondOpRegister::E => self.e,
                    SecondOpRegister::H => self.h,
                    SecondOpRegister::L => self.l,
                    SecondOpRegister::mHL => {
                        let val = self.mc.borrow().read(self.make_hl_address());
                        self.sync_hardware_bus();
                        val
                    }
                };
                self.set_flag_conditional(ZERO_FLAG, (reg_value & bit_mask) == 0);
            }
            SecondOpType::SET => match register {
                SecondOpRegister::A => self.a |= bit_mask,
                SecondOpRegister::B => self.b |= bit_mask,
                SecondOpRegister::C => self.c |= bit_mask,
                SecondOpRegister::D => self.d |= bit_mask,
                SecondOpRegister::E => self.e |= bit_mask,
                SecondOpRegister::H => self.h |= bit_mask,
                SecondOpRegister::L => self.l |= bit_mask,
                SecondOpRegister::mHL => {
                    let hl = self.make_hl_address();
                    let val = self.mc.borrow_mut().read(hl);
                    self.sync_hardware_bus();
                    match self.mc.borrow_mut().write(hl, val | bit_mask) {
                        Ok(_) => (),
                        Err(err) => return Err(err),
                    }
                    self.sync_hardware_bus();
                }
            },
            SecondOpType::RESET => match register {
                SecondOpRegister::A => self.a &= !bit_mask,
                SecondOpRegister::B => self.b &= !bit_mask,
                SecondOpRegister::C => self.c &= !bit_mask,
                SecondOpRegister::D => self.d &= !bit_mask,
                SecondOpRegister::E => self.e &= !bit_mask,
                SecondOpRegister::H => self.h &= !bit_mask,
                SecondOpRegister::L => self.l &= !bit_mask,
                SecondOpRegister::mHL => {
                    let mut mc = self.mc.borrow_mut();
                    let hl = self.make_hl_address();
                    let val = mc.read(hl);
                    match mc.write(hl, val & !bit_mask) {
                        Ok(_) => (),
                        Err(err) => return Err(err),
                    }
                }
            },
            SecondOpType::ROTATE_SHIFT => match register {
                SecondOpRegister::A => {
                    let a = self.a;
                    self.a = self.hand_rotate_shift_op(a, action);
                }
                SecondOpRegister::B => self.b &= !bit_mask,
                SecondOpRegister::C => self.c &= !bit_mask,
                SecondOpRegister::D => self.d &= !bit_mask,
                SecondOpRegister::E => self.e &= !bit_mask,
                SecondOpRegister::H => self.h &= !bit_mask,
                SecondOpRegister::L => self.l &= !bit_mask,
                SecondOpRegister::mHL => {
                    let hl = self.make_hl_address();
                    let val = self.mc.borrow_mut().read(hl);
                    self.sync_hardware_bus();
                    match self.mc.borrow_mut().write(hl, val & !bit_mask) {
                        Ok(_) => (),
                        Err(err) => return Err(err),
                    }
                    self.sync_hardware_bus();
                }
            },
        }

        Ok(())
    }

    fn is_flag_set(&self, flag: u8) -> bool {
        (self.f & flag) == flag
    }

    fn do_daa(&mut self) {
        let n = self.is_flag_set(SUBT_FLAG);
        let hc = self.is_flag_set(HALF_CARRY_FLAG);
        let c = self.is_flag_set(CARRY_FLAG);

        let mut temp = self.a as u16;

        if n {
            if hc {
                temp = (temp.wrapping_sub(0x06)) & 0xFF;
            }
            if c {
                temp = temp - 0x06;
            }
        } else {
            if hc || (temp & 0x0f) > 9 {
                temp += 0x06;
            }
            if c || temp > 0x9f {
                temp += 0x60;
            }
        }

        let a = temp as u8;
        self.a = a;
        self.set_flag_conditional(ZERO_FLAG, a == 0);
        self.set_flag_conditional(CARRY_FLAG, temp > 0xFF);
        self.reset_flag(HALF_CARRY_FLAG);
    }

    pub fn is_stopped(&self) -> bool {
        self.stop
    }

    pub fn tick(&mut self, log: &mut Vec<TraceLog>) -> Result<(), String> {
        if self.stop {
            return Ok(());
        }

        let op_val = self.read_pc_mem_and_increment();
        return self.do_op(op_val, log);
    }

    pub fn do_op(&mut self, op_val: u8, log: &mut Vec<TraceLog>) -> Result<(), String> {
        let op = match OpCodes::from_u8(op_val) {
            Some(op) => op,
            None => {
                let err = format!("Unrecognized opcode value: {}!!!", op_val);
                return Err(err);
            }
        };

        let mut log_item = TraceLog::new(op);

        // should be safe to subtract 1 because we just incremented?
        println!(
            "    A:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} F:{:02X} H:{:02X} L:{:02X}",
            self.a, self.b, self.c, self.d, self.e, self.f, self.h, self.l
        );
        println!(
            "{:<10} ({:#06X})",
            format!("{:?}", op),
            self.pc.get().wrapping_sub(1)
        );

        let mut result: Result<(), String> = Ok(());
        match op {
            OpCodes::LD_A_A => {
                // do nothing since it's copying to itself
            }
            OpCodes::LD_A_B => {
                self.a = self.b;
            }
            OpCodes::LD_A_C => {
                self.a = self.c;
            }
            OpCodes::LD_A_D => {
                self.a = self.d;
            }
            OpCodes::LD_A_E => {
                self.a = self.e;
            }
            OpCodes::LD_A_H => {
                self.a = self.h;
            }
            OpCodes::LD_A_L => {
                self.a = self.l;
            }
            OpCodes::LD_B_A => {
                self.b = self.a;
            }
            OpCodes::LD_B_B => {
                // pass
            }
            OpCodes::LD_B_C => {
                self.b = self.c;
            }
            OpCodes::LD_B_D => {
                self.b = self.d;
            }
            OpCodes::LD_B_E => {
                self.b = self.e;
            }
            OpCodes::LD_B_H => {
                self.b = self.h;
            }
            OpCodes::LD_B_L => {
                self.b = self.l;
            }
            OpCodes::LD_C_A => {
                self.c = self.a;
            }
            OpCodes::LD_C_B => {
                self.c = self.b;
            }
            OpCodes::LD_C_C => {
                // pass
            }
            OpCodes::LD_C_D => {
                self.c = self.d;
            }
            OpCodes::LD_C_E => {
                self.c = self.e;
            }
            OpCodes::LD_C_H => {
                self.c = self.h;
            }
            OpCodes::LD_C_L => {
                self.c = self.l;
            }
            OpCodes::LD_D_A => {
                self.d = self.a;
            }
            OpCodes::LD_D_B => {
                self.d = self.b;
            }
            OpCodes::LD_D_C => {
                self.d = self.c;
            }
            OpCodes::LD_D_D => {
                // pass
            }
            OpCodes::LD_D_E => {
                self.d = self.e;
            }
            OpCodes::LD_D_H => {
                self.d = self.h;
            }
            OpCodes::LD_D_L => {
                self.d = self.l;
            }
            OpCodes::LD_E_A => {
                self.e = self.a;
            }
            OpCodes::LD_E_B => {
                self.e = self.b;
            }
            OpCodes::LD_E_C => {
                self.e = self.c;
            }
            OpCodes::LD_E_D => {
                self.e = self.d;
            }
            OpCodes::LD_E_E => {
                // pass
            }
            OpCodes::LD_E_H => {
                self.e = self.h;
            }
            OpCodes::LD_E_L => {
                self.e = self.l;
            }
            OpCodes::LD_H_A => {
                self.h = self.a;
            }
            OpCodes::LD_H_B => {
                self.h = self.b;
            }
            OpCodes::LD_H_C => {
                self.h = self.c;
            }
            OpCodes::LD_H_D => {
                self.h = self.d;
            }
            OpCodes::LD_H_E => {
                self.h = self.e;
            }
            OpCodes::LD_H_H => {
                // pass
            }
            OpCodes::LD_H_L => {
                self.h = self.l;
            }
            OpCodes::LD_L_A => {
                self.l = self.a;
            }
            OpCodes::LD_L_B => {
                self.l = self.b;
            }
            OpCodes::LD_L_C => {
                self.l = self.c;
            }
            OpCodes::LD_L_D => {
                self.l = self.d;
            }
            OpCodes::LD_L_E => {
                self.l = self.e;
            }
            OpCodes::LD_L_H => {
                self.l = self.h;
            }
            OpCodes::LD_L_L => {
                // pass
            }
            OpCodes::LD_A_N => {
                self.a = self.read_pc_mem_and_increment();
            }
            OpCodes::LD_B_N => {
                self.b = self.read_pc_mem_and_increment();
            }
            OpCodes::LD_C_N => {
                self.c = self.read_pc_mem_and_increment();
            }
            OpCodes::LD_D_N => {
                self.d = self.read_pc_mem_and_increment();
            }
            OpCodes::LD_E_N => {
                self.e = self.read_pc_mem_and_increment();
            }
            OpCodes::LD_H_N => {
                self.h = self.read_pc_mem_and_increment();
            }
            OpCodes::LD_L_N => {
                self.l = self.read_pc_mem_and_increment();
            }
            OpCodes::LD_A_mHL => {
                let addr = self.make_hl_address();
                self.a = self.mc.borrow().read(addr);
            }
            OpCodes::LD_B_mHL => {
                let addr = self.make_hl_address();
                self.b = self.mc.borrow().read(addr);
            }
            OpCodes::LD_C_mHL => {
                let addr = self.make_hl_address();
                self.c = self.mc.borrow().read(addr);
            }
            OpCodes::LD_D_mHL => {
                let addr = self.make_hl_address();
                self.d = self.mc.borrow().read(addr);
            }
            OpCodes::LD_E_mHL => {
                let addr = self.make_hl_address();
                self.e = self.mc.borrow().read(addr);
            }
            OpCodes::LD_H_mHL => {
                let addr = self.make_hl_address();
                self.h = self.mc.borrow().read(addr);
            }
            OpCodes::LD_L_mHL => {
                let addr = self.make_hl_address();
                self.l = self.mc.borrow().read(addr);
            }
            OpCodes::LD_mHL_A => {
                let addr = self.make_hl_address();
                let val = self.a;
                result = self.mc.borrow_mut().write(addr, val);
            }
            OpCodes::LD_mHL_B => {
                let addr = self.make_hl_address();
                let val = self.b;
                result = self.mc.borrow_mut().write(addr, val);
            }
            OpCodes::LD_mHL_C => {
                let addr = self.make_hl_address();
                let val = self.c;
                result = self.mc.borrow_mut().write(addr, val);
            }
            OpCodes::LD_mHL_D => {
                let addr = self.make_hl_address();
                let val = self.d;
                result = self.mc.borrow_mut().write(addr, val);
            }
            OpCodes::LD_mHL_E => {
                let addr = self.make_hl_address();
                let val = self.e;
                result = self.mc.borrow_mut().write(addr, val);
            }
            OpCodes::LD_mHL_H => {
                let addr = self.make_hl_address();
                let val = self.h;
                result = self.mc.borrow_mut().write(addr, val);
            }
            OpCodes::LD_mHL_L => {
                let addr = self.make_hl_address();
                let val = self.l;
                result = self.mc.borrow_mut().write(addr, val);
            }
            OpCodes::LD_mHL_N => {
                let addr = self.make_hl_address();
                let val = self.read_pc_mem_and_increment();
                result = self.mc.borrow_mut().write(addr, val);
            }
            OpCodes::LD_A_mBC => {
                let addr = self.make_bc_address();
                self.a = self.mc.borrow().read(addr);
            }
            OpCodes::LD_A_mDE => {
                let addr = self.make_de_address();
                self.a = self.mc.borrow().read(addr);
            }
            OpCodes::LD_A_mC => {
                let addr = self.make_ffc_address();
                self.a = self.mc.borrow().read(addr);
            }
            OpCodes::LD_mC_A => {
                let addr = self.make_ffc_address();
                let a = self.a;
                result = self.mc.borrow_mut().write(addr, a);
            }
            OpCodes::LD_A_mN => {
                let addr = self.make_ffn_address();
                self.a = self.mc.borrow().read(addr);
            }
            OpCodes::LD_mN_A => {
                let addr = self.make_ffn_address();
                let a = self.a;
                result = self.mc.borrow_mut().write(addr, a);
            }
            OpCodes::LD_A_mNN => {
                let addr = self.make_nn_address();
                self.a = self.mc.borrow().read(addr);
            }
            OpCodes::LD_mNN_A => {
                let addr = self.make_nn_address();
                let a = self.a;
                result = self.mc.borrow_mut().write(addr, a);
            }
            OpCodes::LD_A_HLI => {
                let addr = self.make_hl_address();
                self.a = self.mc.borrow().read(addr);
                increment_16(&mut self.h, &mut self.l);
            }
            OpCodes::LD_A_HLD => {
                let addr = self.make_hl_address();
                self.a = self.mc.borrow().read(addr);
                decrement_16(&mut self.h, &mut self.l);
            }
            OpCodes::LD_mBC_A => {
                let addr = self.make_bc_address();
                let a = self.a;
                result = self.mc.borrow_mut().write(addr, a);
            }
            OpCodes::LD_mDE_A => {
                let addr = self.make_de_address();
                let a = self.a;
                result = self.mc.borrow_mut().write(addr, a);
            }
            OpCodes::LD_HLI_A => {
                let addr = self.make_hl_address();
                let a = self.a;
                increment_16(&mut self.h, &mut self.l);
                result = self.mc.borrow_mut().write(addr, a);
            }
            OpCodes::LD_HLD_A => {
                let addr = self.make_hl_address();
                let a = self.a;
                decrement_16(&mut self.h, &mut self.l);
                result = self.mc.borrow_mut().write(addr, a);
            }
            OpCodes::LD_BC_NN => {
                let pair = self.read_address_pair();
                self.b = pair.1;
                self.c = pair.0;
            }
            OpCodes::LD_DE_NN => {
                let pair = self.read_address_pair();
                self.d = pair.1;
                self.e = pair.0;
            }
            OpCodes::LD_HL_NN => {
                let pair = self.read_address_pair();
                self.h = pair.1;
                self.l = pair.0;
            }
            OpCodes::LD_SP_NN => {
                self.sp = self.make_nn_address();
            }
            OpCodes::LD_SP_HL => {
                self.sp = self.make_hl_address();
            }
            OpCodes::PUSH_BC => {
                let b = self.b;
                let c = self.c;
                result = self.push_address_parts(b, c);
            }
            OpCodes::PUSH_DE => {
                let d = self.d;
                let e = self.e;
                result = self.push_address_parts(d, e);
            }
            OpCodes::PUSH_HL => {
                let h = self.h;
                let l = self.l;
                result = self.push_address_parts(h, l);
            }
            OpCodes::PUSH_AF => {
                let a = self.a;
                let f = self.f;
                result = self.push_address_parts(a, f);
            }
            OpCodes::POP_BC => {
                let parts = self.pop_address_parts();
                self.b = parts.0;
                self.c = parts.1;
            }
            OpCodes::POP_DE => {
                let parts = self.pop_address_parts();
                self.b = parts.0;
                self.c = parts.1;
            }
            OpCodes::POP_HL => {
                let parts = self.pop_address_parts();
                self.b = parts.0;
                self.c = parts.1;
            }
            OpCodes::POP_AF => {
                let parts = self.pop_address_parts();
                self.b = parts.0;
                self.c = parts.1;
            }
            OpCodes::LDHL_SP_e => {
                let b = self.read_pc_mem_and_increment();
                let sp = self.sp.get();
                let temp = self.add_to_u16(b, sp);
                self.h = ((temp & 0xFF00) >> 8) as u8;
                self.l = (temp & 0x00FF) as u8;
            }
            OpCodes::LD_mNN_SP => {
                let mut addr = self.make_nn_address();
                let sp = self.sp.get();
                result = self.mc
                    .borrow_mut()
                    .write(addr.post_inc(1), (sp & 0x00ff) as u8);
                match result {
                    Ok(_) => (),
                    r @ Err(_) => return r,
                }
                result = self.mc.borrow_mut().write(addr, ((sp & 0xff00) >> 8) as u8);
            }
            OpCodes::ADD_A_A => {
                let val = self.a;
                let a = self.a;
                self.a = self.add(a, val);
            }
            OpCodes::ADD_A_B => {
                let val = self.b;
                let a = self.a;
                self.a = self.add(a, val);
            }
            OpCodes::ADD_A_C => {
                let val = self.c;
                let a = self.a;
                self.a = self.add(a, val);
            }
            OpCodes::ADD_A_D => {
                let val = self.d;
                let a = self.a;
                self.a = self.add(a, val);
            }
            OpCodes::ADD_A_E => {
                let val = self.e;
                let a = self.a;
                self.a = self.add(a, val);
            }
            OpCodes::ADD_A_H => {
                let val = self.h;
                let a = self.a;
                self.a = self.add(a, val);
            }
            OpCodes::ADD_A_L => {
                let val = self.l;
                let a = self.a;
                self.a = self.add(a, val);
            }
            OpCodes::ADD_A_N => {
                let val = self.read_pc_mem_and_increment();
                let a = self.a;
                self.a = self.add(a, val);
            }
            OpCodes::ADD_A_mHL => {
                let addr = self.make_hl_address();
                let val = self.mc.borrow().read(addr);
                let a = self.a;
                self.a = self.add(a, val);
            }
            OpCodes::ADC_A_A => {
                let val = self.a;
                let a = self.a;
                self.a = self.add_with_carry(a, val);
            }
            OpCodes::ADC_A_B => {
                let val = self.b;
                let a = self.a;
                self.a = self.add_with_carry(a, val);
            }
            OpCodes::ADC_A_C => {
                let val = self.c;
                let a = self.a;
                self.a = self.add_with_carry(a, val);
            }
            OpCodes::ADC_A_D => {
                let val = self.d;
                let a = self.a;
                self.a = self.add_with_carry(a, val);
            }
            OpCodes::ADC_A_E => {
                let val = self.e;
                let a = self.a;
                self.a = self.add_with_carry(a, val);
            }
            OpCodes::ADC_A_H => {
                let val = self.h;
                let a = self.a;
                self.a = self.add_with_carry(a, val);
            }
            OpCodes::ADC_A_L => {
                let val = self.l;
                let a = self.a;
                self.a = self.add_with_carry(a, val);
            }
            OpCodes::ADC_A_N => {
                let val = self.read_pc_mem_and_increment();
                let a = self.a;
                self.a = self.add_with_carry(a, val);
            }
            OpCodes::ADC_A_mHL => {
                let addr = self.make_hl_address();
                let val = self.mc.borrow().read(addr);
                let a = self.a;
                self.a = self.add_with_carry(a, val);
            }
            OpCodes::SUB_A => {
                let val = self.a;
                let a = self.a;
                self.a = self.subtract(a, val);
            }
            OpCodes::SUB_B => {
                let val = self.b;
                let a = self.a;
                self.a = self.subtract(a, val);
            }
            OpCodes::SUB_C => {
                let val = self.c;
                let a = self.a;
                self.a = self.subtract(a, val);
            }
            OpCodes::SUB_D => {
                let val = self.d;
                let a = self.a;
                self.a = self.subtract(a, val);
            }
            OpCodes::SUB_E => {
                let val = self.e;
                let a = self.a;
                self.a = self.subtract(a, val);
            }
            OpCodes::SUB_H => {
                let val = self.h;
                let a = self.a;
                self.a = self.subtract(a, val);
            }
            OpCodes::SUB_L => {
                let val = self.l;
                let a = self.a;
                self.a = self.subtract(a, val);
            }
            OpCodes::SUB_N => {
                let val = self.read_pc_mem_and_increment();
                let a = self.a;
                self.a = self.subtract(a, val);
            }
            OpCodes::SUB_mHL => {
                let addr = self.make_hl_address();
                let val = self.mc.borrow().read(addr);
                let a = self.a;
                self.a = self.subtract(a, val);
            }
            OpCodes::SBC_A_A => {
                let a = self.a;
                let val = self.a;
                self.a = self.subtract_with_carry(a, val);
            }
            OpCodes::SBC_A_B => {
                let a = self.a;
                let val = self.b;
                self.a = self.subtract_with_carry(a, val);
            }
            OpCodes::SBC_A_C => {
                let a = self.a;
                let val = self.c;
                self.a = self.subtract_with_carry(a, val);
            }
            OpCodes::SBC_A_D => {
                let a = self.a;
                let val = self.d;
                self.a = self.subtract_with_carry(a, val);
            }
            OpCodes::SBC_A_E => {
                let a = self.a;
                let val = self.e;
                self.a = self.subtract_with_carry(a, val);
            }
            OpCodes::SBC_A_H => {
                let a = self.a;
                let val = self.h;
                self.a = self.subtract_with_carry(a, val);
            }
            OpCodes::SBC_A_L => {
                let a = self.a;
                let val = self.l;
                self.a = self.subtract_with_carry(a, val);
            }
            OpCodes::SBC_A_N => {
                let val = self.read_pc_mem_and_increment();
                let a = self.a;
                self.a = self.subtract_with_carry(a, val);
            }
            OpCodes::SBC_A_mHL => {
                let addr = self.make_hl_address();
                let val = self.mc.borrow().read(addr);
                let a = self.a;
                self.a = self.subtract_with_carry(a, val);
            }
            OpCodes::AND_A => {
                self.a = self.a & self.a;
                let a = self.a;
                self.set_logic_flags(a, true);
            }
            OpCodes::AND_B => {
                self.a = self.a & self.b;
                let a = self.a;
                self.set_logic_flags(a, true);
            }
            OpCodes::AND_C => {
                self.a = self.a & self.c;
                let a = self.a;
                self.set_logic_flags(a, true);
            }
            OpCodes::AND_D => {
                self.a = self.a & self.d;
                let a = self.a;
                self.set_logic_flags(a, true);
            }
            OpCodes::AND_E => {
                self.a = self.a & self.e;
                let a = self.a;
                self.set_logic_flags(a, true);
            }
            OpCodes::AND_H => {
                self.a = self.a & self.h;
                let a = self.a;
                self.set_logic_flags(a, true);
            }
            OpCodes::AND_L => {
                self.a = self.a & self.l;
                let a = self.a;
                self.set_logic_flags(a, true);
            }
            OpCodes::AND_N => {
                self.a = self.a & self.read_pc_mem_and_increment();
                let a = self.a;
                self.set_logic_flags(a, true);
            }
            OpCodes::AND_mHL => {
                let addr = self.make_hl_address();
                let val = self.mc.borrow().read(addr);
                self.a = self.a & val;
                let a = self.a;
                self.set_logic_flags(a, true);
            }
            OpCodes::OR_A => {
                self.a = self.a | self.a;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::OR_B => {
                self.a = self.a | self.b;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::OR_C => {
                self.a = self.a | self.c;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::OR_D => {
                self.a = self.a | self.d;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::OR_E => {
                self.a = self.a | self.e;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::OR_H => {
                self.a = self.a | self.h;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::OR_L => {
                self.a = self.a | self.l;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::OR_N => {
                let val = self.read_pc_mem_and_increment();
                self.a = self.a | val;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::OR_mHL => {
                let addr = self.make_hl_address();
                let val = self.mc.borrow().read(addr);
                self.a = self.a | val;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::XOR_A => {
                self.a ^= self.a;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::XOR_B => {
                self.a ^= self.b;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::XOR_C => {
                self.a ^= self.c;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::XOR_D => {
                self.a ^= self.d;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::XOR_E => {
                self.a ^= self.e;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::XOR_H => {
                self.a ^= self.h;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::XOR_L => {
                self.a ^= self.l;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::XOR_N => {
                let val = self.read_pc_mem_and_increment();
                self.a ^= val;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::XOR_mHL => {
                let addr = self.make_hl_address();
                let val = self.mc.borrow().read(addr);
                self.a ^= val;
                let a = self.a;
                self.set_logic_flags(a, false);
            }
            OpCodes::CP_A => {
                let val = self.a;
                let a = self.a;
                self.subtract_with_carry(a, val);
            }
            OpCodes::CP_B => {
                let val = self.b;
                let a = self.a;
                self.subtract_with_carry(a, val);
            }
            OpCodes::CP_C => {
                let val = self.c;
                let a = self.a;
                self.subtract_with_carry(a, val);
            }
            OpCodes::CP_D => {
                let val = self.d;
                let a = self.a;
                self.subtract_with_carry(a, val);
            }
            OpCodes::CP_E => {
                let val = self.e;
                let a = self.a;
                self.subtract_with_carry(a, val);
            }
            OpCodes::CP_H => {
                let val = self.h;
                let a = self.a;
                self.subtract_with_carry(a, val);
            }
            OpCodes::CP_L => {
                let val = self.l;
                let a = self.a;
                self.subtract_with_carry(a, val);
            }
            OpCodes::CP_N => {
                let val = self.read_pc_mem_and_increment();
                let a = self.a;
                self.subtract_with_carry(a, val);
            }
            OpCodes::CP_mHL => {
                let addr = self.make_hl_address();
                let val = self.mc.borrow().read(addr);
                let a = self.a;
                self.subtract_with_carry(a, val);
            }
            OpCodes::INC_A => {
                let mut val = self.a;
                self.increment(&mut val);
                self.a = val;
            }
            OpCodes::INC_B => {
                let mut val = self.b;
                self.increment(&mut val);
                self.b = val;
            }
            OpCodes::INC_C => {
                let mut val = self.c;
                self.increment(&mut val);
                self.c = val;
            }
            OpCodes::INC_D => {
                let mut val = self.d;
                self.increment(&mut val);
                self.d = val;
            }
            OpCodes::INC_E => {
                let mut val = self.e;
                self.increment(&mut val);
                self.e = val;
            }
            OpCodes::INC_H => {
                let mut val = self.h;
                self.increment(&mut val);
                self.h = val;
            }
            OpCodes::INC_L => {
                let mut val = self.l;
                self.increment(&mut val);
                self.l = val;
            }
            OpCodes::INC_mHL => {
                let addr = self.make_hl_address();
                let mut val = self.mc.borrow().read(addr);
                self.increment(&mut val);
                result = self.mc.borrow_mut().write(addr, val);
            }
            OpCodes::DEC_A => {
                let mut val = self.a;
                self.decrement(&mut val);
                self.a = val;
            }
            OpCodes::DEC_B => {
                let mut val = self.b;
                self.decrement(&mut val);
                self.b = val;
            }
            OpCodes::DEC_C => {
                let mut val = self.c;
                self.decrement(&mut val);
                self.c = val;
            }
            OpCodes::DEC_D => {
                let mut val = self.d;
                self.decrement(&mut val);
                self.d = val;
            }
            OpCodes::DEC_E => {
                let mut val = self.e;
                self.decrement(&mut val);
                self.e = val;
            }
            OpCodes::DEC_H => {
                let mut val = self.h;
                self.decrement(&mut val);
                self.h = val;
            }
            OpCodes::DEC_L => {
                let mut val = self.l;
                self.decrement(&mut val);
                self.l = val;
            }
            OpCodes::DEC_mHL => {
                let addr = self.make_hl_address();
                let mut val = self.mc.borrow().read(addr);
                self.decrement(&mut val);
                result = self.mc.borrow_mut().write(addr, val);
            }
            OpCodes::ADD_HL_BC => {
                let h = self.h;
                let l = self.l;
                let b = self.b;
                let c = self.c;
                self.l = self.add(l, c);
                self.h = self.add_with_carry(h, b);
            }
            OpCodes::ADD_HL_DE => {
                let h = self.h;
                let l = self.l;
                let d = self.d;
                let e = self.e;
                self.l = self.add(l, e);
                self.h = self.add_with_carry(h, d);
            }
            OpCodes::ADD_HL_HL => {
                let h = self.h;
                let l = self.l;
                self.l = self.add(l, l);
                self.h = self.add_with_carry(h, h);
            }
            OpCodes::ADD_HL_SP => {
                let sp_val = self.sp.get();
                let h = self.h;
                let l = self.l;
                self.l = self.add(l, sp_val as u8);
                let carry = self.get_carry_value();
                self.h = self.add(h, ((sp_val & 0xFF00) >> 8) as u8 + carry);
            }
            OpCodes::ADD_SP_e => {
                let val = self.read_pc_mem_and_increment() as u16;
                self.sp.inc(val);
            }
            OpCodes::INC_BC => {
                increment_16(&mut self.b, &mut self.c);
            }
            OpCodes::INC_DE => {
                increment_16(&mut self.d, &mut self.e);
            }
            OpCodes::INC_HL => {
                increment_16(&mut self.h, &mut self.l);
            }
            OpCodes::INC_SP => {
                self.sp.inc(1);
            }
            OpCodes::DEC_BC => {
                decrement_16(&mut self.b, &mut self.c);
            }
            OpCodes::DEC_DE => {
                decrement_16(&mut self.d, &mut self.e);
            }
            OpCodes::DEC_HL => {
                decrement_16(&mut self.h, &mut self.l);
            }
            OpCodes::DEC_SP => {
                self.sp.dec(1);
            }
            OpCodes::RLCA => {
                let a = self.a;
                self.a = self.do_rlc(a);
            }
            OpCodes::RLA => {
                let a = self.a;
                self.a = self.do_rl(a);
            }
            OpCodes::RRCA => {
                let a = self.a;
                self.a = self.do_rrc(a);
            }
            OpCodes::RRA => {
                let a = self.a;
                self.a = self.do_rr(a);
            }
            OpCodes::MULTI_BYTE_OP => {
                // this code accounts for many variants based on the second byte read
                let next_op = self.read_pc_mem_and_increment();
                result = self.decode_and_execute_cb_op(next_op);
            }
            OpCodes::JP_NN => {
                self.do_jump_conditional(true);
            }
            OpCodes::JP_NZ_NN => {
                let f = self.f;
                self.do_jump_conditional((f & ZERO_FLAG) == 0);
            }
            OpCodes::JP_Z_NN => {
                let f = self.f;
                self.do_jump_conditional((f & ZERO_FLAG) == ZERO_FLAG);
            }
            OpCodes::JP_NC_NN => {
                let f = self.f;
                self.do_jump_conditional((f & CARRY_FLAG) == 0);
            }
            OpCodes::JP_C_NN => {
                let f = self.f;
                self.do_jump_conditional((f & CARRY_FLAG) == CARRY_FLAG);
            }
            OpCodes::JR_e => {
                self.do_jump_relative_conditional(true);
            }
            OpCodes::JR_NZ_e => {
                let f = self.f;
                self.do_jump_relative_conditional((f & ZERO_FLAG) == 0);
            }
            OpCodes::JR_Z_e => {
                let f = self.f;
                self.do_jump_relative_conditional((f & ZERO_FLAG) == ZERO_FLAG);
            }
            OpCodes::JR_NC_e => {
                let f = self.f;
                self.do_jump_relative_conditional((f & CARRY_FLAG) == 0);
            }
            OpCodes::JR_C_e => {
                let f = self.f;
                self.do_jump_relative_conditional((f & CARRY_FLAG) == CARRY_FLAG);
            }
            OpCodes::JP_mHL => {
                // self.actually just loads self.hL into self.pc, not memory at self.hL... :(
                self.pc = self.make_hl_address();
            }
            OpCodes::CALL_NN => {
                result = self.do_call_conditional(true);
            }
            OpCodes::CALL_NZ_NN => {
                let f = self.f;
                result = self.do_call_conditional((f & ZERO_FLAG) == 0);
            }
            OpCodes::CALL_Z_NN => {
                let f = self.f;
                result = self.do_call_conditional((f & ZERO_FLAG) == ZERO_FLAG);
            }
            OpCodes::CALL_NC_NN => {
                let f = self.f;
                result = self.do_call_conditional((f & CARRY_FLAG) == 0);
            }
            OpCodes::CALL_C_NN => {
                let f = self.f;
                result = self.do_call_conditional((f & CARRY_FLAG) == CARRY_FLAG);
            }
            OpCodes::RET => {
                self.do_return_conditional(true);
            }
            OpCodes::RETI => {
                self.do_return_conditional(true);
                self.ime = true;
            }
            OpCodes::RET_NZ => {
                let f = self.f;
                self.do_return_conditional((f & ZERO_FLAG) == 0);
            }
            OpCodes::RET_Z => {
                let f = self.f;
                self.do_return_conditional((f & ZERO_FLAG) == ZERO_FLAG);
            }
            OpCodes::RET_NC => {
                let f = self.f;
                self.do_return_conditional((f & CARRY_FLAG) == 0);
            }
            OpCodes::RET_C => {
                let f = self.f;
                self.do_return_conditional((f & CARRY_FLAG) == CARRY_FLAG);
            }
            OpCodes::RST_0 => {
                let pc = self.pc;
                result = self.push_address(pc);
                self.pc = RamAddress::new(0x0000);
            }
            OpCodes::RST_1 => {
                let pc = self.pc;
                result = self.push_address(pc);
                self.pc = RamAddress::new(0x0008);
            }
            OpCodes::RST_2 => {
                let pc = self.pc;
                result = self.push_address(pc);
                self.pc = RamAddress::new(0x0010);
            }
            OpCodes::RST_3 => {
                let pc = self.pc;
                result = self.push_address(pc);
                self.pc = RamAddress::new(0x0018);
            }
            OpCodes::RST_4 => {
                let pc = self.pc;
                result = self.push_address(pc);
                self.pc = RamAddress::new(0x0020);
            }
            OpCodes::RST_5 => {
                let pc = self.pc;
                result = self.push_address(pc);
                self.pc = RamAddress::new(0x0028);
            }
            OpCodes::RST_6 => {
                let pc = self.pc;
                result = self.push_address(pc);
                self.pc = RamAddress::new(0x0030);
            }
            OpCodes::RST_7 => {
                let pc = self.pc;
                result = self.push_address(pc);
                self.pc = RamAddress::new(0x0038);
            }
            OpCodes::DAA => {
                self.do_daa();
            }
            OpCodes::CPL => {
                self.a = !self.a;
            }
            OpCodes::NOP => {
                // literally no operation done here
            }
            OpCodes::HALT => {
                self.halt = true;
            }
            OpCodes::STOP => {
                // TODO: set all inputs to self.lOW
                self.stop = true;
                result = self.mc.borrow_mut().write(IE_ADDR, 0);
            }
            OpCodes::EI => {
                self.ime = true;
            }
            OpCodes::DI => {
                self.ime = false;
            }
        }

        log.push(log_item);

        result
    }
}
