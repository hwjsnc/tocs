// DirectX BC7 (aka OpenGL BPTC) graphics format implementation
//
// Specification: https://registry.khronos.org/DataFormat/specs/1.3/dataformat.1.3.html#bptc_bc7
// Original (?) spec: https://registry.khronos.org/OpenGL/extensions/ARB/ARB_texture_compression_bptc.txt
// Overview: https://learn.microsoft.com/en-us/windows/win32/direct3d11/bc7-format

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

    fn mode(&self) -> u8 {
        std::cmp::min(self.data.leading_zeros(), 8) as u8 // TODO: make sure to only consider the first byte (even if it's all zero)
    }
}

pub type DecodedBlock = [[Color; 4]; 4];

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

fn subset_index(partition: u8, i: u8, subsets: u8) -> u8 {
    const P2: [[u8; 16]; 64] = [
        [0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1],
        [0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1],
        [0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1],
        [0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1],
        [0, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1],
        [0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1],
        [0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1],
        [0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1],
        [0, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0],
        [0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0],
        [0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0],
        [0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 1],
        [0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0],
        [0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0],
        [0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0],
        [0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0],
        [0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0],
        [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
        [0, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 0],
        [0, 0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0],
        [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1],
        [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1],
        [0, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0],
        [0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0],
        [0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0],
        [0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0],
        [0, 1, 1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 0, 0, 1],
        [0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1],
        [0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0],
        [0, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0],
        [0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0],
        [0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 0, 0],
        [0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0],
        [0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1],
        [0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1],
        [0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0],
        [0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0],
        [0, 0, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0],
        [0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1],
        [0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1],
        [0, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0],
        [0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 0],
        [0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1],
        [0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1],
        [0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1],
        [0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1],
        [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1],
        [0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0],
        [0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0],
        [0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1],
    ];
    const P3: [[u8; 16]; 64] = [
        [0, 0, 1, 1, 0, 0, 1, 1, 0, 2, 2, 1, 2, 2, 2, 2],
        [0, 0, 0, 1, 0, 0, 1, 1, 2, 2, 1, 1, 2, 2, 2, 1],
        [0, 0, 0, 0, 2, 0, 0, 1, 2, 2, 1, 1, 2, 2, 1, 1],
        [0, 2, 2, 2, 0, 0, 2, 2, 0, 0, 1, 1, 0, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 1, 1, 2, 2],
        [0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 2, 2],
        [0, 0, 2, 2, 0, 0, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1],
        [0, 0, 1, 1, 0, 0, 1, 1, 2, 2, 1, 1, 2, 2, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2],
        [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2],
        [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2],
        [0, 0, 1, 2, 0, 0, 1, 2, 0, 0, 1, 2, 0, 0, 1, 2],
        [0, 1, 1, 2, 0, 1, 1, 2, 0, 1, 1, 2, 0, 1, 1, 2],
        [0, 1, 2, 2, 0, 1, 2, 2, 0, 1, 2, 2, 0, 1, 2, 2],
        [0, 0, 1, 1, 0, 1, 1, 2, 1, 1, 2, 2, 1, 2, 2, 2],
        [0, 0, 1, 1, 2, 0, 0, 1, 2, 2, 0, 0, 2, 2, 2, 0],
        [0, 0, 0, 1, 0, 0, 1, 1, 0, 1, 1, 2, 1, 1, 2, 2],
        [0, 1, 1, 1, 0, 0, 1, 1, 2, 0, 0, 1, 2, 2, 0, 0],
        [0, 0, 0, 0, 1, 1, 2, 2, 1, 1, 2, 2, 1, 1, 2, 2],
        [0, 0, 2, 2, 0, 0, 2, 2, 0, 0, 2, 2, 1, 1, 1, 1],
        [0, 1, 1, 1, 0, 1, 1, 1, 0, 2, 2, 2, 0, 2, 2, 2],
        [0, 0, 0, 1, 0, 0, 0, 1, 2, 2, 2, 1, 2, 2, 2, 1],
        [0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 2, 2, 0, 1, 2, 2],
        [0, 0, 0, 0, 1, 1, 0, 0, 2, 2, 1, 0, 2, 2, 1, 0],
        [0, 1, 2, 2, 0, 1, 2, 2, 0, 0, 1, 1, 0, 0, 0, 0],
        [0, 0, 1, 2, 0, 0, 1, 2, 1, 1, 2, 2, 2, 2, 2, 2],
        [0, 1, 1, 0, 1, 2, 2, 1, 1, 2, 2, 1, 0, 1, 1, 0],
        [0, 0, 0, 0, 0, 1, 1, 0, 1, 2, 2, 1, 1, 2, 2, 1],
        [0, 0, 2, 2, 1, 1, 0, 2, 1, 1, 0, 2, 0, 0, 2, 2],
        [0, 1, 1, 0, 0, 1, 1, 0, 2, 0, 0, 2, 2, 2, 2, 2],
        [0, 0, 1, 1, 0, 1, 2, 2, 0, 1, 2, 2, 0, 0, 1, 1],
        [0, 0, 0, 0, 2, 0, 0, 0, 2, 2, 1, 1, 2, 2, 2, 1],
        [0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 2, 2, 1, 2, 2, 2],
        [0, 2, 2, 2, 0, 0, 2, 2, 0, 0, 1, 2, 0, 0, 1, 1],
        [0, 0, 1, 1, 0, 0, 1, 2, 0, 0, 2, 2, 0, 2, 2, 2],
        [0, 1, 2, 0, 0, 1, 2, 0, 0, 1, 2, 0, 0, 1, 2, 0],
        [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0],
        [0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2, 0],
        [0, 1, 2, 0, 2, 0, 1, 2, 1, 2, 0, 1, 0, 1, 2, 0],
        [0, 0, 1, 1, 2, 2, 0, 0, 1, 1, 2, 2, 0, 0, 1, 1],
        [0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0, 1, 1],
        [0, 1, 0, 1, 0, 1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2],
        [0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 2, 1, 2, 1, 2, 1],
        [0, 0, 2, 2, 1, 1, 2, 2, 0, 0, 2, 2, 1, 1, 2, 2],
        [0, 0, 2, 2, 0, 0, 1, 1, 0, 0, 2, 2, 0, 0, 1, 1],
        [0, 2, 2, 0, 1, 2, 2, 1, 0, 2, 2, 0, 1, 2, 2, 1],
        [0, 1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 0, 1, 0, 1],
        [0, 0, 0, 0, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1],
        [0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 2, 2, 2, 2],
        [0, 2, 2, 2, 0, 1, 1, 1, 0, 2, 2, 2, 0, 1, 1, 1],
        [0, 0, 0, 2, 1, 1, 1, 2, 0, 0, 0, 2, 1, 1, 1, 2],
        [0, 0, 0, 0, 2, 1, 1, 2, 2, 1, 1, 2, 2, 1, 1, 2],
        [0, 2, 2, 2, 0, 1, 1, 1, 0, 1, 1, 1, 0, 2, 2, 2],
        [0, 0, 0, 2, 1, 1, 1, 2, 1, 1, 1, 2, 0, 0, 0, 2],
        [0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 2, 2, 2, 2],
        [0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 2, 2, 1, 1, 2],
        [0, 1, 1, 0, 0, 1, 1, 0, 2, 2, 2, 2, 2, 2, 2, 2],
        [0, 0, 2, 2, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 2, 2],
        [0, 0, 2, 2, 1, 1, 2, 2, 1, 1, 2, 2, 0, 0, 2, 2],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 2],
        [0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1],
        [0, 2, 2, 2, 1, 2, 2, 2, 0, 2, 2, 2, 1, 2, 2, 2],
        [0, 1, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
        [0, 1, 1, 1, 2, 0, 1, 1, 2, 2, 0, 1, 2, 2, 2, 0],
    ];
    //let i = (x % 4) * 4 + (y % 4); // x-major order
    match subsets {
        2 => P2[partition as usize][i as usize],
        3 => P3[partition as usize][i as usize],
        _ => unreachable!("invalid number of subsets ({subsets})"),
    }
}

fn is_anchor_index(i: u8, partition: u8, subsets: u8) -> bool {
    const A2: [u8; 64] = [
        15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 2, 8, 2, 2, 8, 8, 15, 2, 8, 2, 2, 8, 8, 2, 2, 15, 15, 6, 8, 2, 8, 15, 15, 2, 8, 2, 2, 2, 15, 15, 6, 6, 2, 6, 8, 15, 15, 2,
        2, 15, 15, 15, 15, 15, 2, 2, 15,
    ];

    const A3A: [u8; 64] = [
        3, 3, 15, 15, 8, 3, 15, 15, 8, 8, 6, 6, 6, 5, 3, 3, 3, 3, 8, 15, 3, 3, 6, 10, 5, 8, 8, 6, 8, 5, 15, 15, 8, 15, 3, 5, 6, 10, 8, 15, 15, 3, 15, 5, 15, 15, 15, 15, 3, 15, 5, 5, 5, 8, 5, 10, 5,
        10, 8, 13, 15, 12, 3, 3,
    ];

    const A3B: [u8; 64] = [
        15, 8, 8, 3, 15, 15, 3, 8, 15, 15, 15, 15, 15, 15, 15, 8, 15, 8, 15, 3, 15, 8, 15, 8, 3, 15, 6, 10, 15, 15, 10, 8, 15, 3, 15, 10, 10, 8, 9, 10, 6, 15, 8, 15, 3, 6, 6, 8, 15, 3, 15, 15, 15,
        15, 15, 15, 15, 15, 15, 15, 3, 15, 15, 8,
    ];

    match subsets {
        1 => i == 0,
        2 => i == 0 || i == A2[partition as usize],
        3 => i == 0 || i == A3A[partition as usize] || i == A3B[partition as usize],
        _ => unreachable!("invalid number of subsets ({subsets})"),
    }
}

fn slice_to_u8<T: bitvec::store::BitStore, O: bitvec::order::BitOrder>(slice: &bitvec::slice::BitSlice<T, O>) -> u8 {
    assert!(slice.len() <= 8);
    slice.iter().enumerate().fold(0u8, |acc, (i, b)| acc | ((*b as u8) << i))
}

struct Bc7ModeParams {
    subsets: u8,
    partition_bits: u8,
    has_rotation: bool,
    has_index_selection: bool,
    color_bits: u8,
    alpha_bits: u8,
    p_bits: u8,
    shared_p_bits: u8,
    primary_index_bits: u8,
    secondary_index_bits: u8,
}

pub fn decode_block(block: &Bc7BlockSlice) -> DecodedBlock {
    // mode parameters for modes 0-7.
    const PARAMS: [Bc7ModeParams; 8] = [
        Bc7ModeParams {
            // mode 0
            subsets: 3,
            partition_bits: 4,
            has_rotation: false,
            has_index_selection: false,
            color_bits: 4,
            alpha_bits: 0,
            p_bits: 1,
            shared_p_bits: 0,
            primary_index_bits: 3,
            secondary_index_bits: 0,
        },
        Bc7ModeParams {
            // mode 1
            subsets: 2,
            partition_bits: 6,
            has_rotation: false,
            has_index_selection: false,
            color_bits: 6,
            alpha_bits: 0,
            p_bits: 0,
            shared_p_bits: 1,
            primary_index_bits: 3,
            secondary_index_bits: 0,
        },
        Bc7ModeParams {
            // mode 2
            subsets: 3,
            partition_bits: 6,
            has_rotation: false,
            has_index_selection: false,
            color_bits: 5,
            alpha_bits: 0,
            p_bits: 0,
            shared_p_bits: 0,
            primary_index_bits: 2,
            secondary_index_bits: 0,
        },
        Bc7ModeParams {
            // mode 3
            subsets: 2,
            partition_bits: 6,
            has_rotation: false,
            has_index_selection: false,
            color_bits: 7,
            alpha_bits: 0,
            p_bits: 1,
            shared_p_bits: 0,
            primary_index_bits: 2,
            secondary_index_bits: 0,
        },
        Bc7ModeParams {
            // mode 4
            subsets: 1,
            partition_bits: 0,
            has_rotation: true,
            has_index_selection: true,
            color_bits: 5,
            alpha_bits: 6,
            p_bits: 0,
            shared_p_bits: 0,
            primary_index_bits: 2,
            secondary_index_bits: 3,
        },
        Bc7ModeParams {
            // mode 5
            subsets: 1,
            partition_bits: 0,
            has_rotation: true,
            has_index_selection: false,
            color_bits: 7,
            alpha_bits: 8,
            p_bits: 0,
            shared_p_bits: 0,
            primary_index_bits: 2,
            secondary_index_bits: 2,
        },
        Bc7ModeParams {
            // mode 6
            subsets: 1,
            partition_bits: 0,
            has_rotation: false,
            has_index_selection: false,
            color_bits: 7,
            alpha_bits: 7,
            p_bits: 1,
            shared_p_bits: 0,
            primary_index_bits: 4,
            secondary_index_bits: 0,
        },
        Bc7ModeParams {
            // mode 7
            subsets: 2,
            partition_bits: 6,
            has_rotation: false,
            has_index_selection: false,
            color_bits: 5,
            alpha_bits: 5,
            p_bits: 1,
            shared_p_bits: 0,
            primary_index_bits: 2,
            secondary_index_bits: 0,
        },
    ];

    // we read from the block bit by bit; we don't need random access
    let bits = block.bits();
    let mut off: usize = 0;
    let mut read_bits = move |nbits: u8| -> u8 {
        let nbits: usize = nbits.into();
        let res: u8 = slice_to_u8(&bits[off..off + nbits]);
        off += nbits;
        res
    };

    // determine mode by the number of leading zeros
    let mode_number = block.mode();
    if mode_number > 7 {
        // Hardware should return all zeros for mode 8, so let's do that.
        let mut colors = DecodedBlock::default();
        for x in 0..4 {
            for y in 0..4 {
                colors[x][y].r = 0;
                colors[x][y].g = 0;
                colors[x][y].b = 0;
                colors[x][y].a = 0;
            }
        }
        return colors;
    }
    let p = &PARAMS[mode_number as usize];

    // skip mode identification bits
    _ = read_bits(mode_number + 1);

    // read miscellaneous data
    let partition = read_bits(p.partition_bits);

    let rotation: u8;
    if p.has_rotation {
        rotation = read_bits(2);
    } else {
        rotation = 0;
    }

    let index_number: u8;
    if p.has_index_selection {
        index_number = read_bits(1);
    } else {
        index_number = 0;
    }

    // read endpoint color/alpha data

    // endpoint colors:
    // - innermost index is channel (rgba)
    // - middle index is endpoint (low/high)
    // - outermost index is subset (0, 1, 2; only some modes use all subsets)
    let mut endpoints: [[[u8; 4]; 2]; 3] = <[[[u8; 4]; 2]; 3]>::default();

    for c in 0..3 {
        for i in 0..(p.subsets as usize) {
            for j in 0..2 {
                endpoints[i][j][c] = read_bits(p.color_bits);
            }
        }
    }
    if p.alpha_bits != 0 {
        let c = 3;
        for i in 0..(p.subsets as usize) {
            for j in 0..2 {
                endpoints[i][j][c] = read_bits(p.alpha_bits);
            }
        }
    }

    // per-endpoint p-bits
    if p.p_bits != 0 {
        for i in 0..(p.subsets as usize) {
            for j in 0..2 {
                let nbits = p.p_bits;
                let p = read_bits(nbits);
                for c in 0..4 {
                    endpoints[i][j][c] = (endpoints[i][j][c] << nbits) | p;
                }
            }
        }
    }

    // shared p-bits
    if p.shared_p_bits != 0 {
        for i in 0..(p.subsets as usize) {
            let nbits = p.shared_p_bits;
            let p = read_bits(p.shared_p_bits);
            for j in 0..2 {
                for c in 0..4 {
                    endpoints[i][j][c] = (endpoints[i][j][c] << nbits) | p;
                }
            }
        }
    }

    // fully decode endpoints by shifting the raw bits up and replicating them to the lower bits
    for i in 0..p.subsets {
        for j in 0..2 {
            for c in 0..4 {
                let raw_bits;
                if c < 3 {
                    raw_bits = p.color_bits + p.p_bits + p.shared_p_bits;
                } else {
                    raw_bits = p.alpha_bits + p.p_bits + p.shared_p_bits;
                }
                // note: raw_bits > 4 except when alpha_bits = 0, in which case the values don't matter
                let x = endpoints[i as usize][j as usize][c as usize];
                endpoints[i as usize][j as usize][c as usize] = (x << (8 - raw_bits)) | (x >> (2 * raw_bits - 8));
            }
        }
    }
    let endpoints = endpoints;

    // skip primary index bits if necessary
    let index_bits: u8;
    if index_number == 1 {
        _ = read_bits(16 * p.primary_index_bits - p.subsets);
        index_bits = p.secondary_index_bits;
    } else {
        index_bits = p.primary_index_bits;
    }

    // read the index bits and interpolate the color/alpha values
    let mut colors = DecodedBlock::default();

    let mut i = 0;
    for x in 0..4 {
        for y in 0..4 {
            let subset = subset_index(i, partition, p.subsets) as usize;
            let index: u8;
            if is_anchor_index(i, partition, p.subsets) {
                index = read_bits(index_bits - 1);
            } else {
                index = read_bits(index_bits);
            }
            colors[x][y].r = interpolate(endpoints[subset][0][0], endpoints[subset][1][0], index, index_bits);
            colors[x][y].g = interpolate(endpoints[subset][0][1], endpoints[subset][1][1], index, index_bits);
            colors[x][y].b = interpolate(endpoints[subset][0][2], endpoints[subset][1][2], index, index_bits);
            if p.alpha_bits != 0 {
                colors[x][y].a = interpolate(endpoints[subset][0][3], endpoints[subset][1][3], index, index_bits);
            } else {
                colors[x][y].a = 255;
            }
            match rotation {
                0 => {} // nothing
                1 => std::mem::swap(&mut colors[x][y].a, &mut colors[x][y].r),
                2 => std::mem::swap(&mut colors[x][y].a, &mut colors[x][y].g),
                3 => std::mem::swap(&mut colors[x][y].a, &mut colors[x][y].b),
                _ => unreachable!("invalid rotation number {rotation} should never be constructed"),
            }
            i += 1;
        }
    }

    colors
}

#[derive(Clone, Debug)]
pub enum BC7DecodeError {
    UnsupportedSize,
    DataSize,
}

impl std::fmt::Display for BC7DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            BC7DecodeError::UnsupportedSize => write!(f, "unsupported size: width and height should be multiples of 4"),
            BC7DecodeError::DataSize => write!(f, "incorrect data size"),
        }
    }
}

impl std::error::Error for BC7DecodeError {}

/// Decode a BC7 encoded image with blocks in x-major order
///
/// Returns a colors with pixels in x-major order.
pub fn bc7_decode(data: &[u8], width: usize, height: usize) -> Result<std::vec::Vec<Color>, BC7DecodeError> {
    if (width % 4 != 0) || (height % 4 != 0) {
        return Err(BC7DecodeError::UnsupportedSize); // other sizes are valid, but require more effort
    }

    let number_of_blocks = (width * height) / 16;
    if data.len() != number_of_blocks * 8 {
        return Err(BC7DecodeError::DataSize);
    }

    let mut result = std::vec::Vec::new();
    result.resize(width * height, Color::default());

    let mut off = 0;
    for block_x in 0..(width / 4) {
        for block_y in 0..(height / 4) {
            let block_colors = decode_block(&Bc7BlockSlice::from_slice(&data[off..off + 8]));
            for x in 0..4 {
                for y in 0..4 {
                    let i = (4 * block_x + x) * height + (4 * block_y + y);
                    result[i] = block_colors[x][y];
                }
            }
            off += 8;
        }
    }

    Ok(result)
}
