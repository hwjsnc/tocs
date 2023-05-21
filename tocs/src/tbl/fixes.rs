use hexhex::hex_literal;

use blake2::Digest as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameVersion {
    Unknown,
    CS1,
    CS2,
    CS3,
    CS4,
    CS5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaybeFixed {
    NoFixApplied,
    Fixed,
}

impl MaybeFixed {
    pub fn was_fixed(&self) -> bool {
        match self {
            MaybeFixed::NoFixApplied => false,
            MaybeFixed::Fixed => true,
        }
    }
}

pub enum MaybeFixedCow<'a> {
    NoFixApplied(&'a [u8]),
    Fixed(std::vec::Vec<u8>),
}

impl<'a> MaybeFixedCow<'a> {
    pub fn was_fixed(&self) -> bool {
        match self {
            MaybeFixedCow::NoFixApplied(_) => false,
            MaybeFixedCow::Fixed(_) => true,
        }
    }
}

pub fn fix_in_place(version: GameVersion, data: &mut std::vec::Vec<u8>) -> MaybeFixed {
    match get_fix_for(version, data) {
        Some(fix) => {
            fix(data);
            MaybeFixed::Fixed
        }
        None => MaybeFixed::NoFixApplied,
    }
}

pub fn fix(version: GameVersion, data: &[u8]) -> MaybeFixedCow<'_> {
    match get_fix_for(version, data) {
        Some(fix) => {
            let mut copy = data.to_owned();
            fix(&mut copy);
            MaybeFixedCow::Fixed(copy)
        }
        None => MaybeFixedCow::NoFixApplied(data),
    }
}

fn replace_byte(data: &mut [u8], location: usize, expected: u8, replacement: u8) {
    assert_eq!(data[location], expected);
    data[location] = replacement;
}

fn replace_bytes(data: &mut [u8], replacements: &[(usize, u8, u8)]) {
    for (location, expected, replacement) in replacements.iter().copied() {
        replace_byte(data, location, expected, replacement);
    }
}

type FixFun = dyn Fn(&mut std::vec::Vec<u8>) + Sync;

macro_rules! fix_fun {
    ($ver:expr, $hash:expr, $version_string:literal, $filename:literal, $fixfun:expr) => {
        (
            $ver,
            $hash,
            &(|data: &mut std::vec::Vec<u8>| {
                log::debug!("detected {}, '{}'. Applying fixes", $version_string, $filename);
                $fixfun(data)
            }) as &'static FixFun,
        )
    };
}

fn get_fix_for(version: GameVersion, data: &[u8]) -> Option<&'static FixFun> {
    let hash: [u8; 64] = blake2::Blake2b512::digest(data).try_into().unwrap();

    static FIXES: &[(GameVersion, [u8; 64], &FixFun)] = &[
        fix_fun!(
            GameVersion::CS1,
            *hex_literal!("e3a75ea6c0902da1f8da0d052305a4db98839a8a3c4403cf129144177a3910504a4c8b11ce8aa177db3a7554c58575e86a41655fb8414a90ce2c3083ecef88d5"),
            "Trails of Cold Steel PC version 1.6",
            "data/text/dat_us/t_main.tbl",
            |data| replace_bytes(data, &[(127310, 0x54, 0x47), (128670, 0xa4, 0x53)])
        ),
        fix_fun!(
            GameVersion::CS3,
            *hex_literal!("711db0f9f27f55cfbd99b0b2b556b94d5fe428efe2e4fb34a0f217fd5a08b40d68b6c959870a95a16336f5e05312742fb39dbb0e4ca921e0c3652aacbff81f41"),
            "Trails of Cold Steel III PC version 1.05",
            "data/text/dat_en/t_name.tbl",
            |data| replace_bytes(data, &[(22392, 0x57, 0x55), (22493, 0x57, 0x55), (23898, 0x55, 0x53), (23997, 0x54, 0x52)])
        ),
        fix_fun!(
            GameVersion::CS3,
            *hex_literal!("6467d5835a0197ef06bf585cbc66d735aa808f6d1029c999c719cfc53ce3832dec92ce25845fb4f27c2ec5c0030020571e2e1a4b3977dba8fee4e413dd64f4fa"),
            "Trails of Cold Steel III PC version 1.05",
            "data/text/dat_en/t_quest.tbl",
            |data| replace_bytes(
                data,
                &[
                    (914, 0x2c, 0x30),
                    (972, 0x35, 0x39),
                    (1039, 0x34, 0x38),
                    (1105, 0x2f, 0x33),
                    (1166, 0x2b, 0x2f),
                    (1223, 0x33, 0x37),
                    (1288, 0x2c, 0x30),
                    (1346, 0x2c, 0x30),
                    (1404, 0x32, 0x36),
                    (1468, 0x2b, 0x2f),
                    (1525, 0x2e, 0x32),
                    (1585, 0x2d, 0x31),
                    (1644, 0x34, 0x38),
                    (1710, 0x36, 0x3a),
                    (1778, 0x2e, 0x32),
                    (1838, 0x34, 0x38),
                    (1904, 0x34, 0x38),
                    (1970, 0x35, 0x39),
                    (2037, 0x33, 0x37),
                    (2102, 0x26, 0x2a),
                    (2154, 0x2d, 0x31),
                    (2213, 0x2e, 0x32),
                    (2273, 0x35, 0x39),
                    (2340, 0x34, 0x38),
                    (2406, 0x2d, 0x31),
                    (2465, 0x36, 0x3a),
                    (2533, 0x36, 0x3a),
                    (2601, 0x30, 0x34),
                    (2663, 0x36, 0x3a),
                    (2731, 0x35, 0x39),
                    (2798, 0x34, 0x38),
                    (2864, 0x34, 0x38),
                    (2930, 0x35, 0x39),
                    (2997, 0x36, 0x3a),
                    (3065, 0x28, 0x2c),
                    (3119, 0x31, 0x35),
                    (3182, 0x35, 0x39),
                    (3249, 0x35, 0x39),
                    (3316, 0x35, 0x39),
                    (3383, 0x35, 0x39),
                    (3450, 0x35, 0x39),
                    (3517, 0x2f, 0x33)
                ]
            )
        ),
        fix_fun!(
            GameVersion::CS3,
            *hex_literal!("7db08a20e3e887e7ad94f5ee1023e7c0130ebaca23f4e9715f5517192b6ab57dfbb60826e12e9e22e4293f9d28c6150189d14ea946b19a640e969958c88b87d3"),
            "Trails of Cold Steel III PC version 1.05",
            "data/text/dat_fr/t_quest.tbl",
            |data| replace_bytes(
                data,
                &[
                    (921, 0x27, 0x2b),
                    (974, 0x36, 0x3a),
                    (1042, 0x39, 0x3d),
                    (1113, 0x33, 0x37),
                    (1178, 0x34, 0x38),
                    (1244, 0x34, 0x38),
                    (1310, 0x32, 0x36),
                    (1374, 0x37, 0x3b),
                    (1443, 0x31, 0x35),
                    (1506, 0x34, 0x38),
                    (1572, 0x37, 0x3b),
                    (1641, 0x38, 0x3c),
                    (1711, 0x35, 0x39),
                    (1778, 0x33, 0x37),
                    (1843, 0x36, 0x3a),
                    (1911, 0x31, 0x35),
                    (1974, 0x36, 0x3a),
                    (2042, 0x32, 0x36),
                    (2106, 0x37, 0x3b),
                    (2175, 0x33, 0x37),
                    (2240, 0x39, 0x3d),
                    (2311, 0x34, 0x38),
                    (2377, 0x38, 0x3c),
                    (2447, 0x31, 0x35),
                    (2510, 0x35, 0x39),
                    (2577, 0x34, 0x38),
                    (2643, 0x34, 0x38),
                    (2709, 0x2d, 0x31),
                    (2768, 0x35, 0x39),
                    (2835, 0x34, 0x38),
                    (2901, 0x38, 0x3c),
                    (2971, 0x3a, 0x3e),
                    (3043, 0x28, 0x2c),
                    (3097, 0x2d, 0x31),
                    (3156, 0x2c, 0x30),
                    (3214, 0x38, 0x3c),
                    (3284, 0x38, 0x3c),
                    (3354, 0x38, 0x3c),
                    (3424, 0x38, 0x3c),
                    (3494, 0x38, 0x3c),
                    (3564, 0x37, 0x3b),
                    (3633, 0x33, 0x37)
                ]
            )
        ),
    ];

    FIXES
        .iter()
        .find_map(|(v, h, fun)| if (v == &version || version == GameVersion::Unknown) && h == &hash { Some(*fun) } else { None })
}
