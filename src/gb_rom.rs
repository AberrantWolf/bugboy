use std::cell::RefCell;
use std::fmt::*;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use std::str;
use std::vec::Vec;
use std::result::Result;

use num::FromPrimitive;

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

#[allow(non_camel_case_types)]
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

enum_from_primitive! {
#[derive(Debug)]
enum CartType {
    ROM_ONLY = 0x00,
    MBC1 = 0x01,
    MBC1_RAM = 0x02,
    MBC1_RAM_BATTERY = 0x03,
    MBC2 = 0x05,
    MBC2_BATTERY = 0x06,
    ROM_RAM = 0x08,
    ROM_RAM_BATTERY = 0x09,
    MMM01 = 0x0B,
    MMM01_RAM = 0x0C,
    MMM01_RAM_BATTERY = 0x0D,
    MBC3_TIMER_BATTERY = 0x0F,
    MBC3_TIMER_RAM_BATTERY = 0x10,
    MBC3 = 0x11,
    MBC3_RAM = 0x12,
    MBC3_RAM_BATTERY = 0x13,
    MBC5 = 0x19,
    MBC5_RAM = 0x1A,
    MBC5_RAM_BATTERY = 0x1B,
    MBC5_RUMBLE = 0x1C,
    MBC5_RUMBLE_RAM = 0x1D,
    MBC5_RUMBLE_RAM_BATTERY = 0x1E,
    MBC6 = 0x20,
    MBC7_SENSOR_RUMBLE_RAM_BATTERY = 0x22,
    POCKET_CAMERA = 0xFC,
    BANDAI_TAMA5 = 0xFD,
    HuC3 = 0xFE,
    HuC1_RAM_BATTERY = 0xFF,
}
}

enum_from_primitive! {
#[derive(Debug)]
enum RomSize {
    RS_32KByte = 0x00,  // (no ROM banking)
    RS_64KByte = 0x01,  // (4 banks)
    RS_128KByte = 0x02, // (8 banks)
    RS_256KByte = 0x03, // (16 banks)
    RS_512KByte = 0x04, // (32 banks)
    RS_1MByte = 0x05,   // (64 banks)  - only 63 banks used by MBC1
    RS_2MByte = 0x06,   // (128 banks) - only 125 banks used by MBC1
    RS_4MByte = 0x07,   // (256 banks)
    RS_8MByte = 0x08,   // (512 banks)
    RS_1_1MByte = 0x52, // (72 banks)
    RS_1_2MByte = 0x53, // (80 banks)
    RS_1_5MByte = 0x54, // (96 banks)
}}

enum_from_primitive! {
#[derive(Debug)]
enum CartRamSize {
    CR_None = 0x00,
    CR_2KB = 0x01,
    CR_8KB = 0x02,
    CR_32KB = 0x03,  // (4 banks of 8KBytes each)
    CR_128KB = 0x04, // (16 banks of 8KBytes each)
    CR_64KB = 0x05,  // (8 banks of 8KBytes each)
}}

#[derive(Debug)]
enum DestinationCode {
    Japan = 0x00,
    NonJapan = 0x01, //racist
    Unknown,
}

enum_from_primitive! {
#[allow(non_camel_case_types)]
#[derive(Debug)]
enum OldLicenseCode {
    none =  0x0,
    nintendo =  0x1,
    capcom =  0x8,
    hot_b =  0x9,
    electronic_arts = 0x13,
    hudsonsoft = 0x18,
    itc_entertainment = 0x19,
    pcm_complete = 0x24,
    san_x = 0x25,
    kotobuki_systems = 0x28,
    seta = 0x29,
    infogrames = 0x30,
    nintendo2 = 0x31,
    bandai = 0x32,
    GBC_use_new = 0x33,
    konami = 0x34,
    hector = 0x35,
    capcom2 = 0x38,
    banpresto = 0x39,
    ubi_soft = 0x41,
    atlus = 0x42,
    malibu = 0x44,
    angel = 0x46,
    spectrum_holoby = 0x47,
    irem = 0x49,
    absolute = 0x50,
    acclaim = 0x51,
    activision = 0x52,
    american_sammy = 0x53,
    gametek = 0x54,
    park_place = 0x55,
    ljn = 0x56,
    matchbox = 0x57,
    milton_bradley = 0x59,
    titus = 0x60,
    virgin = 0x61,
    ocean = 0x67,
    electronic_arts2 = 0x69,
    infogrames2 = 0x70,
    interplay = 0x71,
    broderbund = 0x72,
    sculptered_soft = 0x73,
    the_sales_curve = 0x75,
    t_hq = 0x78,
    accolade = 0x79,
    misawa_entertainment = 0x80,
    lozc = 0x83,
    tokuma_shoten_intermedia = 0x86,
    chun_soft = 0x91,
    video_system = 0x92,
    tsuburava = 0x93,
    varie = 0x95,
    yonezawa_s_pal = 0x96,
    kaneko = 0x97,
    arc = 0x99,
    jaleco = 0x0A,
    coconuts = 0x0B,
    elite_systems = 0x0C,
    yanoman = 0x1A,
    clary = 0x1D,
    virgin2 = 0x1F,
    entertainment_i = 0x3C,
    gremlin = 0x3E,
    virgin3 = 0x4A,
    malibu2 = 0x4D,
    u_s_gold = 0x4F,
    mindscape = 0x5A,
    romstar = 0x5B,
    naxat_soft = 0x5C,
    tradewest = 0x5D,
    elite_systems2 = 0x6E,
    electro_brain = 0x6F,
    triffix_entertainment = 0x7A,
    microprose = 0x7C,
    kemco = 0x7F,
    bullet_proof_software = 0x8B,
    vic_tokai = 0x8C,
    ape = 0x8E,
    i_max = 0x8F,
    nihon_bussan = 0x9A,
    tecmo = 0x9B,
    imagineer = 0x9C,
    banpresto2 = 0x9D,
    nova = 0x9F,
    hori_electric = 0xA1,
    bandai2 = 0xA2,
    konami2 = 0xA4,
    kawada = 0xA6,
    takara = 0xA7,
    technos_japan = 0xA9,
    broderbund2 = 0xAA,
    toei_animation = 0xAC,
    toho = 0xAD,
    namco = 0xAF,
    acclaim2 = 0xB0,
    ascii_or_nexoft = 0xB1,
    bandai3 = 0xB2,
    enix = 0xB4,
    hal = 0xB6,
    snk = 0xB7,
    pony_canyon = 0xB9,
    culture_brain_o = 0xBA,
    sunsoft = 0xBB,
    sony_imagesoft = 0xBD,
    sammy = 0xBF,
    taito = 0xC0,
    kemco2 = 0xC2,
    squaresoft = 0xC3,
    tokuma_shoten_intermedia2 = 0xC4,
    data_east = 0xC5,
    tonkin_house = 0xC6,
    koei = 0xC8,
    ufl = 0xC9,
    ultra = 0xCA,
    vap = 0xCB,
    use_ = 0xCC,
    meldac = 0xCD,
    pony_canyon_or = 0xCE,
    angel2 = 0xCF,
    taito2 = 0xD0,
    sofel = 0xD1,
    quest = 0xD2,
    sigma_enterprises = 0xD3,
    ask_kodansha = 0xD4,
    naxat_soft2 = 0xD6,
    copya_systems = 0xD7,
    banpresto3 = 0xD9,
    tomy = 0xDA,
    ljn2 = 0xDB,
    ncs = 0xDD,
    human = 0xDE,
    altron = 0xDF,
    jaleco2 = 0xE0,
    towachiki = 0xE1,
    uutaka = 0xE2,
    varie2 = 0xE3,
    epoch = 0xE5,
    athena = 0xE7,
    asmik = 0xE8,
    natsume = 0xE9,
    king_records = 0xEA,
    atlus2 = 0xEB,
    epic_sony_records = 0xEC,
    igs = 0xEE,
    a_wave = 0xF0,
    extreme_entertainment = 0xF3,
    ljn3 = 0xFF,
}}

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
    ram_size: CartRamSize,      // ram on the cart
    dest_code: DestinationCode, // turn to enum
    old_license_code: OldLicenseCode,
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

        let rom = GbRom {
            title: match str::from_utf8(&buf[0x0134..0x0143]) {
                Ok(s) => String::from(s),
                Err(err) => return Err(format!("ERROR: reading rom title: {}", err)),
            },
            mfg_code: match str::from_utf8(&buf[0x013F..0x0143]) {
                Ok(s) => String::from(s),
                Err(err) => return Err(format!("ERROR: reading manufacturer code: {}", err)),
            },
            color_support: match CgbFlag::from_u8(buf[0x0143]) {
                Ok(val) => val,
                Err(e) => return Err(e),
            },
            new_license_code: match NewLicenseCode::decode(&buf[0x0144..0x0146]) {
                Ok(val) => val,
                Err(e) => {
                    println!("{}", e);
                    NewLicenseCode::None
                }
            },
            sgb_compatible: match buf[0x0146] {
                0x03 => true,
                0x00 => false,
                f @ _ => {
                    println!("Unexpected SGB support flag valye: {}", f);
                    false
                }
            },
            cart_type: {
                let code = buf[0x0147];
                match CartType::from_u8(code) {
                    Some(val) => val,
                    None => return Err(format!("ERROR: Unrecognised cart type: {}", code)),
                }
            },
            rom_size: {
                let code = buf[0x0148];
                match RomSize::from_u8(code) {
                    Some(val) => val,
                    None => return Err(format!("ERROR: Unregocnised rom size: {}", code)),
                }
            },
            ram_size: {
                let code = buf[0x0149];
                match CartRamSize::from_u8(code) {
                    Some(val) => val,
                    None => return Err(format!("ERROR: Unregocnised cart ram size: {}", code)),
                }
            },
            dest_code: match buf[0x014A] {
                0x00 => DestinationCode::Japan,
                0x01 => DestinationCode::NonJapan,
                f @ _ => {
                    println!("ERROR: Unrecognised destination code: {}", f);
                    DestinationCode::Unknown
                }
            },
            old_license_code: {
                let code = buf[0x014B];
                match OldLicenseCode::from_u8(code) {
                    Some(val) => val,
                    None => {
                        println!("WARNING: Unregocnised old license code: {}", code);
                        OldLicenseCode::none
                    }
                }
            },
            mask_rom_version: buf[0x014C],
            complement_checksum: buf[0x014D],
            checksum: [buf[0x014E], buf[0x014F]],
            data: RefCell::new(buf), // put last to avoid getting data after moving
        };

        // TODO: Do the checksum and offer to reject the ROM if it seems too bad

        rom.print_info();
        Ok(rom)
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn print_info(&self) {
        let w = 26;
        println!("=== ROM Info ===============");
        println!("+ {:2$}: {}", "Loaded rom", self.title, w);
        println!("+ {:2$}: {}", "Version", self.mask_rom_version, w);
        println!("+ {:2$}: {}", "Size", self.data.borrow().len(), w);
        println!("+ {:2$}: {}", "Mfg code", self.mfg_code, w);
        println!(
            "+ {:2$}: {:?}",
            "Old license code", self.old_license_code, w
        );
        println!(
            "+ {:2$}: {:?}",
            "New license code", self.new_license_code, w
        );
        println!("+ {:2$}: {:?}", "Region", self.dest_code, w);
        println!("+ {:2$}: {:?}", "Cart type", self.cart_type, w);
        println!("+ {:2$}: {:?}", "ROM size", self.rom_size, w);
        println!("+ {:2$}: {:?}", "RAM size", self.ram_size, w);
        println!(
            "+ {:2$}: {}",
            "Super GameBoy compat",
            if self.sgb_compatible { "yes" } else { "no" },
            w
        );
        println!("+ {:2$}: {:?}", "Color support", self.color_support, w);
        println!(
            "+ {:2$}: {}",
            "Complement checksum", self.complement_checksum, w
        );
        println!(
            "+ {:2$}: {:?}",
            "Encoded (whole) checksum", self.checksum, w
        );
    }
}
