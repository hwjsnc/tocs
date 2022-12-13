// DirectX BC7 graphics format implementation

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Position {
    x: u32,
    y: u32,
}

pub struct Bc7BlockSlice<'a> {
    data: &'a bitvec::slice::BitSlice<u8, bitvec::order::Lsb0>, // length = exactly 128 bits
}

impl<'a> Bc7BlockSlice<'a> {
    fn from_slice(slice: &'a [u8]) -> Self {
        assert_eq!(slice.len(), 16);
        Self {
            data: bitvec::slice::BitSlice::from_slice(slice),
        }
    }

    fn bits(&self) -> &bitvec::slice::BitSlice<u8, bitvec::order::Lsb0> {
        self.data
    }
}

pub type DecodedBlock = [[Color; 4]; 4];

/*
fn get_color_bit_count(block: &Bc7Block, mode: &BlockMode) -> u8 {
    match mode {
        BlockMode::Mode0 | BlockMode::Mode1 => 3,
        BlockMode::Mode2 | BlockMode::Mode3 | BlockMode::Mode5 | BlockMode::Mode7 => 2,
        BlockMode::Mode6 => 4,
        BlockMode::Mode4 => if (block[0] & 0b1000_000) == 0 {
            2
        } else {
            3
        },
        BlockMode::Mode8 => unreachable!("color bit count not defined for BC7 mode 8"),
    }
}
*/

fn interpolate(a: u8, b: u8, index: u8, precision: u8) -> u8 {
    const W2: [u16; 4] = [0, 21, 43, 64];
    const W3: [u16; 8] = [0, 9, 18, 27, 37, 46, 55, 64];
    const W4: [u16; 16] = [0, 4, 9, 13, 17, 21, 26, 30, 34, 38, 43, 47, 51, 55, 60, 64];
    let weight: u16 = match precision {
        2 => W2[index as usize],
        3 => W3[index as usize],
        4 => W4[index as usize],
        _ => unreachable!("index precision {precision} is invalid"),
    };
    let a: u16 = a.into();
    let b: u16 = b.into();
    (((64 - weight) * a + weight * b + 32) >> 6) as u8
}

fn decode_block(pixel_pos: Position, block: &Bc7BlockSlice) -> DecodedBlock {
    // For block modes, see: https://learn.microsoft.com/en-us/windows/win32/direct3d11/bc7-format-mode-reference
    match block.bits().leading_zeros() {
        0 => decode_block_mode_0(pixel_pos, block),
        1 => todo!(),
        2 => todo!(),
        3 => todo!(),
        4 => todo!(),
        5 => todo!(),
        6 => todo!(),
        7 => todo!(),
        _ => decode_block_mode_8(block),
    }
}

fn slice_to_u8<T: bitvec::store::BitStore, O: bitvec::order::BitOrder>(slice: &bitvec::slice::BitSlice<T, O>) -> u8 {
    assert!(slice.len() <= 8);
    slice.iter().enumerate().fold(0u8, |acc, (i, b)| acc | ((*b as u8) << i))
}

fn decode_block_mode_0(pixel_pos: Position, block: &Bc7BlockSlice) -> DecodedBlock {
    // Mode 0:
    // - Only color components, no alpha
    // - 3 subsets (i.e. pair of color endpoints) per block
    // - RGBP 4.4.4.1 endpoints, unique P-bit per endpoint
    // - 3 bit indices
    // - 16 partitions
    //
    // from lsb to msb:
    // - 1 bit mode identifier: 0b1
    // - 4 bits: partition
    // - 6 times 4 bits: R0 to R5
    // - 6 times 4 bits: G0 to G5
    // - 6 times 4 bits: B0 to B5
    // - 6 times 1 bit: P0 to P5
    // 45 bits: index
    let bits = block.bits();

    let partition = slice_to_u8(&bits[1..5]);

    // extract endpoint rgb values
    let mut off = 5;
    let mut r_values: [u8; 6] = <[u8; 6]>::default();
    let mut g_values: [u8; 6] = <[u8; 6]>::default();
    let mut b_values: [u8; 6] = <[u8; 6]>::default();
    for x in r_values.iter_mut() {
        *x = slice_to_u8(&bits[off..off + 4]);
        off += 4;
    }
    for x in g_values.iter_mut() {
        *x = slice_to_u8(&bits[off..off + 4]);
        off += 4;
    }
    for x in b_values.iter_mut() {
        *x = slice_to_u8(&bits[off..off + 4]);
        off += 4;
    }
    assert_eq!(off, 77);

    // extract p-bits and apply them to the endpoint rgb values
    for i in 0..6 {
        r_values[i] = r_values[i] << 1 | bits[off] as u8;
        g_values[i] = g_values[i] << 1 | bits[off] as u8;
        b_values[i] = b_values[i] << 1 | bits[off] as u8;
        off += 1;
    }
    assert_eq!(off, 83);

    // shift endpoint bits up and replicate them on the lower bits
    for i in 0..6 {
        r_values[i] = r_values[i] << 3 | r_values[i] >> 2;
        g_values[i] = g_values[i] << 3 | g_values[i] >> 2;
        b_values[i] = b_values[i] << 3 | b_values[i] >> 2;
    }
    assert_eq!(off, 83);
    // endpoints are decoded now

    // decode pixels one by one
    let mut result = DecodedBlock::default();
    for i in 0..4 {
        for j in 0..4 {
            let subset_index: usize = todo!("{partition} {pixel_pos:?} (and maybe i, j?)"); // 0, 1, or 2
            let color_idx_1 = 2 * subset_index;
            let color_idx_2 = 2 * subset_index + 1;
            let index = slice_to_u8(&bits[off..off + 3]);
            off += 3;

            result[i][j] = Color {
                r: interpolate(r_values[color_idx_1], r_values[color_idx_2], index, 3),
                g: interpolate(g_values[color_idx_1], g_values[color_idx_2], index, 3),
                b: interpolate(b_values[color_idx_1], b_values[color_idx_2], index, 3),
                a: 255,
            }
        }
    }
    assert_eq!(off, 128);

    result
}

fn decode_block_mode_8(block: &Bc7BlockSlice) -> DecodedBlock {
    // Mode 8 is reserved and should not be present
    // From lsb to msb:
    // - 8 bits mode identifier: 0x00
    // - 120 bits: unspecified

    // Hardware is specified to return all 0, so let's do that.
    let mut result: DecodedBlock = DecodedBlock::default();
    for row in result.iter_mut() {
        for px in row.iter_mut() {
            *px = Color { r: 0, g: 0, b: 0, a: 0 };
        }
    }
    result
}
