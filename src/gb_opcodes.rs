use num::FromPrimitive;

struct OpCodeInfo {
    code: u8,
    cycles: usize,
}

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum OpCodes {
    LD_A_A = 0x7F,
    LD_A_B = 0x78,
    LD_A_C = 0x79,
    LD_A_D = 0x7A,
    LD_A_E = 0x7B,
    LD_A_H = 0x7C,
    LD_A_L = 0x7D,
    LD_B_A = 0x47,
    LD_B_B = 0x40,
    LD_B_C = 0x41,
    LD_B_D = 0x42,
    LD_B_E = 0x43,
    LD_B_H = 0x44,
    LD_B_L = 0x45,
    LD_C_A = 0x4F,
    LD_C_B = 0x48,
    LD_C_C = 0x49,
    LD_C_D = 0x4A,
    LD_C_E = 0x4B,
    LD_C_H = 0x4C,
    LD_C_L = 0x4D,
    LD_D_A = 0x57,
    LD_D_B = 0x50,
    LD_D_C = 0x51,
    LD_D_D = 0x52,
    LD_D_E = 0x53,
    LD_D_H = 0x54,
    LD_D_L = 0x55,
    LD_E_A = 0x5F,
    LD_E_B = 0x58,
    LD_E_C = 0x59,
    LD_E_D = 0x5A,
    LD_E_E = 0x5B,
    LD_E_H = 0x5C,
    LD_E_L = 0x5D,
    LD_H_A = 0x67,
    LD_H_B = 0x60,
    LD_H_C = 0x61,
    LD_H_D = 0x62,
    LD_H_E = 0x63,
    LD_H_H = 0x64,
    LD_H_L = 0x65,
    LD_L_A = 0x6F,
    LD_L_B = 0x68,
    LD_L_C = 0x69,
    LD_L_D = 0x6A,
    LD_L_E = 0x6B,
    LD_L_H = 0x6C,
    LD_L_L = 0x6D,
    LD_A_N = 0x3E,
    LD_B_N = 0x06,
    LD_C_N = 0x0E,
    LD_D_N = 0x16,
    LD_E_N = 0x1E,
    LD_H_N = 0x26,
    LD_L_N = 0x2E,
    LD_A_mHL = 0x7E,
    LD_B_mHL = 0x46,
    LD_C_mHL = 0x4E,
    LD_D_mHL = 0x56,
    LD_E_mHL = 0x5E,
    LD_H_mHL = 0x66,
    LD_L_mHL = 0x6E,
    LD_mHL_A = 0x77,
    LD_mHL_B = 0x70,
    LD_mHL_C = 0x71,
    LD_mHL_D = 0x72,
    LD_mHL_E = 0x73,
    LD_mHL_H = 0x74,
    LD_mHL_L = 0x75,
    LD_mHL_N = 0x36,
    LD_A_mBC = 0x0A,
    LD_A_mDE = 0x1A,
    LD_A_mC = 0xF2,
    LD_mC_A = 0xE2,
    LD_A_mN = 0xF0,
    LD_mN_A = 0xE0,
    LD_A_mNN = 0xFA,
    LD_mNN_A = 0xEA,
    LD_A_HLI = 0x2A,
    LD_A_HLD = 0x3A,
    LD_mBC_A = 0x02,
    LD_mDE_A = 0x12,
    LD_HLI_A = 0x22,
    LD_HLD_A = 0x32,
    LD_BC_NN = 0x01,
    LD_DE_NN = 0x11,
    LD_HL_NN = 0x21,
    LD_SP_NN = 0x31,
    LD_SP_HL = 0xF9,
    PUSH_BC = 0xC5,
    PUSH_DE = 0xD5,
    PUSH_HL = 0xE5,
    PUSH_AF = 0xF5,
    POP_BC = 0xC1,
    POP_DE = 0xD1,
    POP_HL = 0xE1,
    POP_AF = 0xF1,
    LDHL_SP_e = 0xF8,
    LD_mNN_SP = 0x08,
    ADD_A_A = 0x87,
    ADD_A_B = 0x80,
    ADD_A_C = 0x81,
    ADD_A_D = 0x82,
    ADD_A_E = 0x83,
    ADD_A_H = 0x84,
    ADD_A_L = 0x85,
    ADD_A_N = 0xC6,
    ADD_A_mHL = 0x86,
    ADC_A_A = 0x8F,
    ADC_A_B = 0x88,
    ADC_A_C = 0x89,
    ADC_A_D = 0x8A,
    ADC_A_E = 0x8B,
    ADC_A_H = 0x8C,
    ADC_A_L = 0x8D,
    ADC_A_N = 0xCE,
    ADC_A_mHL = 0x8E,
    SUB_A = 0x97,
    SUB_B = 0x90,
    SUB_C = 0x91,
    SUB_D = 0x92,
    SUB_E = 0x93,
    SUB_H = 0x94,
    SUB_L = 0x95,
    SUB_N = 0xD6,
    SUB_mHL = 0x96,
    SBC_A_A = 0x9F,
    SBC_A_B = 0x98,
    SBC_A_C = 0x99,
    SBC_A_D = 0x9A,
    SBC_A_E = 0x9B,
    SBC_A_H = 0x9C,
    SBC_A_L = 0x9D,
    SBC_A_N = 0xDE,
    SBC_A_mHL = 0x9E,
    AND_A = 0xA7,
    AND_B = 0xA0,
    AND_C = 0xA1,
    AND_D = 0xA2,
    AND_E = 0xA3,
    AND_H = 0xA4,
    AND_L = 0xA5,
    AND_N = 0xE6,
    AND_mHL = 0xA6,
    OR_A = 0xB7,
    OR_B = 0xB0,
    OR_C = 0xB1,
    OR_D = 0xB2,
    OR_E = 0xB3,
    OR_H = 0xB4,
    OR_L = 0xB5,
    OR_N = 0xF6,
    OR_mHL = 0xB6,
    XOR_A = 0xAF,
    XOR_B = 0xA8,
    XOR_C = 0xA9,
    XOR_D = 0xAA,
    XOR_E = 0xAB,
    XOR_H = 0xAC,
    XOR_L = 0xAD,
    XOR_N = 0xEE,
    XOR_mHL = 0xAE,
    CP_A = 0xBF,
    CP_B = 0xB8,
    CP_C = 0xB9,
    CP_D = 0xBA,
    CP_E = 0xBB,
    CP_H = 0xBC,
    CP_L = 0xBD,
    CP_N = 0xFE,
    CP_mHL = 0xBE,
    INC_A = 0x3C,
    INC_B = 0x04,
    INC_C = 0x0C,
    INC_D = 0x14,
    INC_E = 0x1C,
    INC_H = 0x24,
    INC_L = 0x2C,
    INC_mHL = 0x34,
    DEC_A = 0x3D,
    DEC_B = 0x05,
    DEC_C = 0x0D,
    DEC_D = 0x15,
    DEC_E = 0x1D,
    DEC_H = 0x25,
    DEC_L = 0x2D,
    DEC_mHL = 0x35,
    ADD_HL_BC = 0x09,
    ADD_HL_DE = 0x19,
    ADD_HL_HL = 0x29,
    ADD_HL_SP = 0x39,
    ADD_SP_e = 0xE8,
    INC_BC = 0x03,
    INC_DE = 0x13,
    INC_HL = 0x23,
    INC_SP = 0x33,
    DEC_BC = 0x0B,
    DEC_DE = 0x1B,
    DEC_HL = 0x2B,
    DEC_SP = 0x3B,
    RLCA = 0x07,
    RLA = 0x17,
    RRCA = 0x0F,
    RRA = 0x1F,
    MULTI_BYTE_OP = 0xCB,
    JP_NN = 0xC3,
    JP_NZ_NN = 0xC2,
    JP_Z_NN = 0xCA,
    JP_NC_NN = 0xD2,
    JP_C_NN = 0xDA,
    JR_e = 0x18,
    JR_NZ_e = 0x20,
    JR_Z_e = 0x28,
    JR_NC_e = 0x30,
    JR_C_e = 0x38,
    JP_mHL = 0xE9,
    CALL_NN = 0xCD,
    CALL_NZ_NN = 0xC4,
    CALL_Z_NN = 0xCC,
    CALL_NC_NN = 0xD4,
    CALL_C_NN = 0xDC,
    RET = 0xC9,
    RETI = 0xD9,
    RET_NZ = 0xC0,
    RET_Z = 0xC8,
    RET_NC = 0xD0,
    RET_C = 0xD8,
    RST_0 = 0xC7,
    RST_1 = 0xCF,
    RST_2 = 0xD7,
    RST_3 = 0xDF,
    RST_4 = 0xE7,
    RST_5 = 0xEF,
    RST_6 = 0xF7,
    RST_7 = 0xFF,
    DAA = 0x27,
    CPL = 0x2F,
    NOP = 0x00,
    HALT = 0x76,
    STOP = 0x10,
    EI = 0xF3,
    DI = 0xFB,
}
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SecondOpType {
    // occupying 11_000_000
    ROTATE_SHIFT = 0b00,
    BIT_CHECK = 0b01,
    RESET = 0b10,
    SET = 0b11,
}

impl SecondOpType {
    pub fn from_u8(val: u8) -> Self {
        let bits = (val & 0b11) >> 6;
        match bits {
            0b00 => SecondOpType::ROTATE_SHIFT,
            0b01 => SecondOpType::BIT_CHECK,
            0b10 => SecondOpType::RESET,
            0b11 => SecondOpType::SET,
            _ => panic!("SecondOpType::from_u8 should never get here"),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SecondOpAction {
    // occupying 00_111_000
    RLC = 0b000,
    RL = 0b010,
    RRC = 0b001,
    RR = 0b011,
    SLA = 0b100,
    SRA = 0b101,
    SRL = 0b111,
    SWAP = 0b110,
}

impl SecondOpAction {
    pub fn from_u8(val: u8) -> SecondOpAction {
        let bits = (val & 0b00_111_000) >> 3;
        match bits {
            0b000 => SecondOpAction::RLC,
            0b010 => SecondOpAction::RL,
            0b001 => SecondOpAction::RRC,
            0b011 => SecondOpAction::RR,
            0b100 => SecondOpAction::SLA,
            0b101 => SecondOpAction::SRA,
            0b111 => SecondOpAction::SRL,
            0b110 => SecondOpAction::SWAP,
            _ => panic!("SecondOpAction::from_u8 should never get here"),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SecondOpRegister {
    // occupying 00_000_111
    A = 0b111,
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    mHL = 0b110,
}

impl SecondOpRegister {
    pub fn from_u8(val: u8) -> Self {
        let bits = val & 0b111;
        match bits {
            0b111 => SecondOpRegister::A,
            0b000 => SecondOpRegister::B,
            0b001 => SecondOpRegister::C,
            0b010 => SecondOpRegister::D,
            0b011 => SecondOpRegister::E,
            0b100 => SecondOpRegister::H,
            0b101 => SecondOpRegister::L,
            0b110 => SecondOpRegister::mHL,
            _ => panic!("SecondOpRegister::from_u8 should never get here"),
        }
    }
}
