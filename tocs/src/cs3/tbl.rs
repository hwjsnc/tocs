extern crate blake2;
extern crate hex_literal;
extern crate ioutil;
extern crate lazy_static;

use blake2::Digest as _;

pub fn fix(data: &[u8]) -> std::vec::Vec<u8> {
    let mut copy = data.to_vec();
    let _ = fix_in_place_if_necessary(&mut copy);
    copy
}

pub fn fix_in_place_if_necessary(data: &mut std::vec::Vec<u8>) -> Option<()> {
    type FixFun = fn(&mut std::vec::Vec<u8>) -> ();
    lazy_static::lazy_static! {
        static ref FIXFUN: std::collections::HashMap<[u8; 64], FixFun> = [
                (hex_literal::hex!("711db0f9f27f55cfbd99b0b2b556b94d5fe428efe2e4fb34a0f217fd5a08b40d68b6c959870a95a16336f5e05312742fb39dbb0e4ca921e0c3652aacbff81f41"), fix_CS3_PC_1_05_data_text_dat_en_t_name_tbl as FixFun),
                (hex_literal::hex!("6467d5835a0197ef06bf585cbc66d735aa808f6d1029c999c719cfc53ce3832dec92ce25845fb4f27c2ec5c0030020571e2e1a4b3977dba8fee4e413dd64f4fa"), fix_CS3_PC_1_05_data_text_dat_en_t_quest_tbl as FixFun),
                (hex_literal::hex!("7db08a20e3e887e7ad94f5ee1023e7c0130ebaca23f4e9715f5517192b6ab57dfbb60826e12e9e22e4293f9d28c6150189d14ea946b19a640e969958c88b87d3"), fix_CS3_PC_1_05_data_text_dat_fr_t_quest_tbl as FixFun),
        ].into_iter().collect();
    }
    let hash = blake2::Blake2b512::digest(&data);
    let fixfun = FIXFUN.get(&hash[..])?;
    fixfun(data);
    Some(())
}

fn replace_byte(data: &mut [u8], location: usize, expected: u8, replacement: u8) {
    assert_eq!(data[location], expected);
    data[location] = replacement;
}

#[allow(non_snake_case)]
fn fix_CS3_PC_1_05_data_text_dat_en_t_name_tbl(data: &mut std::vec::Vec<u8>) {
    log::debug!("detected CS3 PC version 1.05, 'data/text/dat_en/t_name.tbl'. Applying fixes.");
    assert_eq!(data.len(), 150070);
    let replace = [(22392, 0x57, 0x55), (22493, 0x57, 0x55), (23898, 0x55, 0x53), (23997, 0x54, 0x52)];
    for (location, expected, replacement) in replace {
        replace_byte(data, location, expected, replacement);
    }
}

#[allow(non_snake_case)]
fn fix_CS3_PC_1_05_data_text_dat_en_t_quest_tbl(data: &mut std::vec::Vec<u8>) {
    log::debug!("detected CS3 PC version 1.05, 'data/text/dat_en/t_quest.tbl'. Applying fixes.");
    assert_eq!(data.len(), 58017);
    let replace = [
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
        (3517, 0x2f, 0x33),
    ];
    for (location, expected, replacement) in replace {
        replace_byte(data, location, expected, replacement);
    }
}

#[allow(non_snake_case)]
fn fix_CS3_PC_1_05_data_text_dat_fr_t_quest_tbl(data: &mut std::vec::Vec<u8>) {
    log::debug!("detected CS3 PC version 1.05, 'data/text/dat_fr/t_quest.tbl'. Applying fixes.");
    assert_eq!(data.len(), 52904);
    let replace = [
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
        (3633, 0x33, 0x37),
    ];
    for (location, expected, replacement) in replace {
        replace_byte(data, location, expected, replacement);
    }
}

#[cfg(test)]
mod test {}
