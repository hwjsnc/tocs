extern crate blake2;

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
                (hex_literal::hex!("e3a75ea6c0902da1f8da0d052305a4db98839a8a3c4403cf129144177a3910504a4c8b11ce8aa177db3a7554c58575e86a41655fb8414a90ce2c3083ecef88d5"), fix_CS1_PC_1_6_dat_us_t_main as FixFun),
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
fn fix_CS1_PC_1_6_dat_us_t_main(data: &mut std::vec::Vec<u8>) {
    log::info!("detected Cold Steel PC version 1.6, 'data/text/dat_us/t_main.tbl'. Applying fix.");
    assert_eq!(data.len(), 155086);
    let replace = [(127310, 0x54, 0x47), (128670, 0xa4, 0x53)];
    for (location, expected, replacement) in replace {
        replace_byte(data, location, expected, replacement);
    }
}

#[cfg(test)]
mod test {}
