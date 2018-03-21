use std::cell::RefCell;
use std::fmt::*;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use std::str;
use std::vec::Vec;
use std::result::Result;

#[derive(Debug)]
enum CgbFlag {
    None,
    Supported,
    Exclusive,
}

impl CgbFlag {
    fn from_u8(val: u8) -> Result<Self, String> {
        Ok(match val {
            0x80 => CgbFlag::Supported,
            0xC0 => CgbFlag::Exclusive,
            0x00 => CgbFlag::None,
            f @ _ => {
                println!("Unexpected CGB support flag value: {}", f);
                CgbFlag::None
            }
        })
    }
}

#[derive(Debug)]
enum NewLicenseCode {
    None,
    NintendoRnD1,
    Capcom,
    Electronic_Arts,
    Hudson_Soft,
    b_ai,
    kss,
    pow,
    PCM_Complete,
    san_x,
    Kemco_Japan,
    seta,
    Viacom,
    Nintendo,
    Bandai,
    Ocean_Acclaim,
    Konami,
    Hector,
    Taito,
    Hudson,
    Banpresto,
    Ubi_Soft,
    Atlus,
    Malibu,
    angel,
    Bullet_Proof,
    irem,
    Absolute,
    Acclaim,
    Activision,
    American_sammy,
    Konami2,
    Hi_tech_entertainment,
    LJN,
    Matchbox,
    Mattel,
    Milton_Bradley,
    Titus,
    Virgin,
    LucasArts,
    Ocean,
    Electronic_Arts2,
    Infogrames,
    Interplay,
    Broderbund,
    sculptured,
    sci,
    THQ,
    Accolade,
    misawa,
    lozc,
    tokuma_shoten_i,
    tsukuda_ori,
    Chunsoft,
    Video_system,
    Ocean_Acclaim2,
    Varie,
    Yonezawas_pal,
    Kaneko,
    Pack_in_soft,
    Konami_Yu_Gi_Oh,
}

impl NewLicenseCode {
    fn decode(val: &[u8]) -> Result<Self, String> {
        Ok(match *val {
            [0x0, 0x0] => NewLicenseCode::None,
            [0x0, 0x1] => NewLicenseCode::NintendoRnD1,
            [0x0, 0x8] => NewLicenseCode::Capcom,
            [0x1, 0x3] => NewLicenseCode::Electronic_Arts,
            [0x1, 0x8] => NewLicenseCode::Hudson_Soft,
            [0x1, 0x9] => NewLicenseCode::b_ai,
            [0x2, 0x0] => NewLicenseCode::kss,
            [0x2, 0x2] => NewLicenseCode::pow,
            [0x2, 0x4] => NewLicenseCode::PCM_Complete,
            [0x2, 0x5] => NewLicenseCode::san_x,
            [0x2, 0x8] => NewLicenseCode::Kemco_Japan,
            [0x2, 0x9] => NewLicenseCode::seta,
            [0x3, 0x0] => NewLicenseCode::Viacom,
            [0x3, 0x1] => NewLicenseCode::Nintendo,
            [0x3, 0x2] => NewLicenseCode::Bandai,
            [0x3, 0x3] => NewLicenseCode::Ocean_Acclaim,
            [0x3, 0x4] => NewLicenseCode::Konami,
            [0x3, 0x5] => NewLicenseCode::Hector,
            [0x3, 0x7] => NewLicenseCode::Taito,
            [0x3, 0x8] => NewLicenseCode::Hudson,
            [0x3, 0x9] => NewLicenseCode::Banpresto,
            [0x4, 0x1] => NewLicenseCode::Ubi_Soft,
            [0x4, 0x2] => NewLicenseCode::Atlus,
            [0x4, 0x4] => NewLicenseCode::Malibu,
            [0x4, 0x6] => NewLicenseCode::angel,
            [0x4, 0x7] => NewLicenseCode::Bullet_Proof,
            [0x4, 0x9] => NewLicenseCode::irem,
            [0x5, 0x0] => NewLicenseCode::Absolute,
            [0x5, 0x1] => NewLicenseCode::Acclaim,
            [0x5, 0x2] => NewLicenseCode::Activision,
            [0x5, 0x3] => NewLicenseCode::American_sammy,
            [0x5, 0x4] => NewLicenseCode::Konami2,
            [0x5, 0x5] => NewLicenseCode::Hi_tech_entertainment,
            [0x5, 0x6] => NewLicenseCode::LJN,
            [0x5, 0x7] => NewLicenseCode::Matchbox,
            [0x5, 0x8] => NewLicenseCode::Mattel,
            [0x5, 0x9] => NewLicenseCode::Milton_Bradley,
            [0x6, 0x0] => NewLicenseCode::Titus,
            [0x6, 0x1] => NewLicenseCode::Virgin,
            [0x6, 0x4] => NewLicenseCode::LucasArts,
            [0x6, 0x7] => NewLicenseCode::Ocean,
            [0x6, 0x9] => NewLicenseCode::Electronic_Arts2,
            [0x7, 0x0] => NewLicenseCode::Infogrames,
            [0x7, 0x1] => NewLicenseCode::Interplay,
            [0x7, 0x2] => NewLicenseCode::Broderbund,
            [0x7, 0x3] => NewLicenseCode::sculptured,
            [0x7, 0x5] => NewLicenseCode::sci,
            [0x7, 0x8] => NewLicenseCode::THQ,
            [0x7, 0x9] => NewLicenseCode::Accolade,
            [0x8, 0x0] => NewLicenseCode::misawa,
            [0x8, 0x3] => NewLicenseCode::lozc,
            [0x8, 0x6] => NewLicenseCode::tokuma_shoten_i,
            [0x8, 0x7] => NewLicenseCode::tsukuda_ori,
            [0x9, 0x1] => NewLicenseCode::Chunsoft,
            [0x9, 0x2] => NewLicenseCode::Video_system,
            [0x9, 0x3] => NewLicenseCode::Ocean_Acclaim2,
            [0x9, 0x5] => NewLicenseCode::Varie,
            [0x9, 0x6] => NewLicenseCode::Yonezawas_pal,
            [0x9, 0x7] => NewLicenseCode::Kaneko,
            [0x9, 0x9] => NewLicenseCode::Pack_in_soft,
            [0xA, 0x4] => NewLicenseCode::Konami_Yu_Gi_Oh,
            _ => return Err(format!("WARNING: Unexpected licensee code: {:?}", val)),
        })
    }
}

#[derive(Debug)]
enum CartType {
    ROM_ONLY,
    MBC1,
    MBC1_RAM,
    MBC1_RAM_BATTERY,
    MBC2,
    MBC2_BATTERY,
    ROM_RAM,
    ROM_RAM_BATTERY,
    MMM01,
    MMM01_RAM,
    MMM01_RAM_BATTERY,
    MBC3_TIMER_BATTERY,
    MBC3_TIMER_RAM_BATTERY,
    MBC3,
    MBC3_RAM,
    MBC3_RAM_BATTERY,
    MBC5,
    MBC5_RAM,
    MBC5_RAM_BATTERY,
    MBC5_RUMBLE,
    MBC5_RUMBLE_RAM,
    MBC5_RUMBLE_RAM_BATTERY,
    MBC6,
    MBC7_SENSOR_RUMBLE_RAM_BATTERY,
    POCKET_CAMERA,
    BANDAI_TAMA5,
    HuC3,
    HuC1_RAM_BATTERY,
}

#[derive(Debug)]
enum RomSize {
    RS_32KByte,  // (no ROM banking)
    RS_64KByte,  // (4 banks)
    RS_128KByte, // (8 banks)
    RS_256KByte, // (16 banks)
    RS_512KByte, // (32 banks)
    RS_1MByte,   // (64 banks)  - only 63 banks used by MBC1
    RS_2MByte,   // (128 banks) - only 125 banks used by MBC1
    RS_4MByte,   // (256 banks)
    RS_8MByte,   // (512 banks)
    RS_1_1MByte, // (72 banks)
    RS_1_2MByte, // (80 banks)
    RS_1_5MByte, // (96 banks)
}

#[derive(Debug)]
enum CartRamSize {
    CR_None,
    CR_2KB,
    CR_8KB,
    CR_32KB,  // (4 banks of 8KBytes each)
    CR_128KB, // (16 banks of 8KBytes each)
    CR_64KB,  // (8 banks of 8KBytes each)
}

#[derive(Debug)]
pub struct GbRom {
    data: RefCell<Vec<u8>>,
    title: String,
    mfg_code: String,
    color_support: CgbFlag,
    new_license_code: NewLicenseCode,
    sgb_compatible: bool,
    cart_type: CartType,
    rom_size: RomSize,
    ram_size: CartRamSize, // ram on the cart
    dest_code: u8,         // turn to enum
    old_license_code: u8,
    mask_rom_version: u8,
    complement_checksum: u8,
    checksum: [u8; 2],
}

impl GbRom {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        let mut romfile = match fs::File::open(&path) {
            Ok(r) => r,
            Err(err) => {
                println!("Error opening file: {}", err);
                panic!();;
            }
        };

        let mut buf = Vec::new();
        let size = match romfile.read_to_end(&mut buf) {
            Ok(s) => s,
            Err(err) => {
                println!("Error reading bytes from rom file: {}", err);
                panic!();;
            }
        };

        println!("Read {} bytes", size);

        Ok(GbRom {
            data: RefCell::new(buf),
            title: match str::from_utf8(&buf[0x0134..0x013E]) {
                Ok(s) => String::from(s),
                Err(err) => return Err(format!("ERROR: reading rom title: {}", err)),
            },
            mfg_code: match str::from_utf8(&buf[0x013F..0x0142]) {
                Ok(s) => String::from(s),
                Err(err) => return Err(format!("ERROR: reading manufacturer code: {}", err)),
            },
            color_support: match CgbFlag::from_u8(buf[0x0143]) {
                Ok(val) => val,
                Err(e) => return Err(e),
            },
            new_license_code: match NewLicenseCode::decode(&buf[0x0144..0x0145]) {
                Ok(val) => val,
                Err(e) => return Err(e),
            },
            sgb_compatible: match buf[0x0146] {
                0x03 => true,
                0x00 => false,
                f @ _ => {
                    println!("Unexpected SGB support flag valye: {}", f);
                    false
                }
            },
            cart_type: match buf[0x0147] {
                0x00 => CartType::ROM_ONLY,
                0x01 => CartType::MBC1,
                0x02 => CartType::MBC1_RAM,
                0x03 => CartType::MBC1_RAM_BATTERY,
                0x05 => CartType::MBC2,
                0x06 => CartType::MBC2_BATTERY,
                0x08 => CartType::ROM_RAM,
                0x09 => CartType::ROM_RAM_BATTERY,
                0x0B => CartType::MMM01,
                0x0C => CartType::MMM01_RAM,
                0x0D => CartType::MMM01_RAM_BATTERY,
                0x0F => CartType::MBC3_TIMER_BATTERY,
                0x10 => CartType::MBC3_TIMER_RAM_BATTERY,
                0x11 => CartType::MBC3,
                0x12 => CartType::MBC3_RAM,
                0x13 => CartType::MBC3_RAM_BATTERY,
                0x19 => CartType::MBC5,
                0x1A => CartType::MBC5_RAM,
                0x1B => CartType::MBC5_RAM_BATTERY,
                0x1C => CartType::MBC5_RUMBLE,
                0x1D => CartType::MBC5_RUMBLE_RAM,
                0x1E => CartType::MBC5_RUMBLE_RAM_BATTERY,
                0x20 => CartType::MBC6,
                0x22 => CartType::MBC7_SENSOR_RUMBLE_RAM_BATTERY,
                0xFC => CartType::POCKET_CAMERA,
                0xFD => CartType::BANDAI_TAMA5,
                0xFE => CartType::HuC3,
                0xFF => CartType::HuC1_RAM_BATTERY,
                f @ _ => return Err(format!("ERROR: Unregocnised cart type: {}", f)),
            },
            rom_size: match buf[0x0148] {
                0x00 => RomSize::RS_32KByte,
                0x01 => RomSize::RS_64KByte,
                0x02 => RomSize::RS_128KByte,
                0x03 => RomSize::RS_256KByte,
                0x04 => RomSize::RS_512KByte,
                0x05 => RomSize::RS_1MByte,
                0x06 => RomSize::RS_2MByte,
                0x07 => RomSize::RS_4MByte,
                0x08 => RomSize::RS_8MByte,
                0x52 => RomSize::RS_1_1MByte,
                0x53 => RomSize::RS_1_2MByte,
                0x54 => RomSize::RS_1_5MByte,
                f @ _ => return Err(format!("ERROR: Unregocnised rom size: {}", f)),
            },
            ram_size: match buf[0x0149] {
                0x00 => CartRamSize::CR_None,
                0x01 => CartRamSize::CR_2KB,
                0x02 => CartRamSize::CR_8KB,
                0x03 => CartRamSize::CR_32KB,
                0x04 => CartRamSize::CR_128KB,
                0x05 => CartRamSize::CR_64KB,
                f @ _ => return Err(format!("ERROR: Unregocnised cart ram size: {}", f)),
            },
            dest_code: u8, // turn to enum
            old_license_code: u8,
            mask_rom_version: u8,
            complement_checksum: u8,
            checksum: [u8; 2],
        })
    }
}
