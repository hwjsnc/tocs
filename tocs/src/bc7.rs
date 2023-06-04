// DirectX BC7 (aka OpenGL BPTC) graphics format implementation
//
// Specification: https://registry.khronos.org/DataFormat/specs/1.3/dataformat.1.3.html#bptc_bc7
// Original (?) spec: https://registry.khronos.org/OpenGL/extensions/ARB/ARB_texture_compression_bptc.txt
// Overview: https://learn.microsoft.com/en-us/windows/win32/direct3d11/bc7-format

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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
        std::cmp::min(self.data[..8].leading_zeros(), 8) as u8
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
        1 => 0,
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

struct BitReader<'a, T: bitvec::store::BitStore, O: bitvec::order::BitOrder> {
    data: &'a bitvec::slice::BitSlice<T, O>,
    offset: usize,
}

impl<'a, T: bitvec::store::BitStore, O: bitvec::order::BitOrder> BitReader<'a, T, O> {
    pub fn new(data: &'a bitvec::slice::BitSlice<T, O>) -> Self {
        Self { data, offset: 0 }
    }

    pub fn read_bits(&mut self, n: usize) -> &'a bitvec::slice::BitSlice<T, O> {
        let result = &self.data[self.offset..self.offset + n];
        self.offset += n;
        result
    }

    pub fn read_bits_as_u8(&mut self, n: usize) -> u8 {
        assert!(n <= 8);
        self.read_bits(n).iter().enumerate().fold(0u8, |acc, (i, b)| acc | ((*b as u8) << i))
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn skip(&mut self, n: usize) {
        self.offset += n;
    }

    pub fn seek(&mut self, off: usize) {
        self.offset = off;
    }
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

#[allow(clippy::needless_range_loop, clippy::needless_late_init)]
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

    let mut reader = BitReader::new(block.bits());

    // determine mode by the number of leading zeros
    let mode_number = block.mode();
    if mode_number > 7 {
        // Hardware should return all zeros for mode 8, so let's do that.
        // Note that the derived Default falls back on u8::default() which is 0.
        return DecodedBlock::default();
    }
    let p = &PARAMS[mode_number as usize];

    /*
    // for identifying problems: color each block based just on its mode
    let mode_colors = [
        Color { r: 255, g: 0, b: 0, a: 255 }, // 0: red
        Color { r: 0, g: 255, b: 0, a: 255 }, // 1: green
        Color { r: 0, g: 0, b: 255, a: 255 }, // 2: blue
        Color { r: 255, g: 255, b: 0, a: 255 }, // 3: yellow
        Color { r: 255, g: 0, b: 255, a: 255 }, // 4: purple
        Color { r: 0, g: 255, b: 255, a: 255 }, // 5: cyan
        Color { r: 128, g: 128, b: 128, a: 255 }, // 6: grey
        Color { r: 255, g: 255, b: 255, a: 255 }, // 7: white
    ];

    let mut colors = DecodedBlock::default();
    for x in 0..4 {
        for y in 0..4 {
            colors[x][y] = mode_colors[mode_number as usize];
        }
    }
    return colors;
    */

    // skip mode identification bits
    reader.skip((mode_number + 1).into());

    // read miscellaneous data
    let partition = reader.read_bits_as_u8(p.partition_bits.into());

    let rotation: u8;
    if p.has_rotation {
        rotation = reader.read_bits_as_u8(2);
    } else {
        rotation = 0;
    }

    let index_number: u8;
    if p.has_index_selection {
        index_number = reader.read_bits_as_u8(1);
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
                endpoints[i][j][c] = reader.read_bits_as_u8(p.color_bits.into());
            }
        }
    }
    if p.alpha_bits != 0 {
        let c = 3;
        for i in 0..(p.subsets as usize) {
            for j in 0..2 {
                endpoints[i][j][c] = reader.read_bits_as_u8(p.alpha_bits.into());
            }
        }
    }

    // per-endpoint p-bits
    if p.p_bits != 0 {
        for i in 0..(p.subsets as usize) {
            for j in 0..2 {
                let nbits = p.p_bits;
                let p = reader.read_bits_as_u8(nbits.into());
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
            let p = reader.read_bits_as_u8(p.shared_p_bits.into());
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

    // determine which index set is used for color and alpha, in case there are two
    let color_index_bits;
    let alpha_index_bits;
    let color_indexes_offset;
    let alpha_indexes_offset;
    if p.secondary_index_bits == 0 {
        color_index_bits = p.primary_index_bits;
        alpha_index_bits = p.primary_index_bits;
        color_indexes_offset = reader.offset();
        alpha_indexes_offset = reader.offset();
    } else {
        let total_primary_index_bits: usize = (16 * p.primary_index_bits - p.subsets).into();
        if index_number == 0 {
            // color uses primary indexes, alpha uses secondary indexes
            color_index_bits = p.primary_index_bits;
            alpha_index_bits = p.secondary_index_bits;
            color_indexes_offset = reader.offset();
            alpha_indexes_offset = reader.offset() + total_primary_index_bits;
        } else {
            // color uses secondary indexes, alpha uses primary indexes
            color_index_bits = p.secondary_index_bits;
            alpha_index_bits = p.primary_index_bits;
            color_indexes_offset = reader.offset() + total_primary_index_bits;
            alpha_indexes_offset = reader.offset();
        }
    }

    // read the color index bits and interpolate the color values
    let mut colors = DecodedBlock::default();

    reader.seek(color_indexes_offset);
    let mut i = 0;
    for y in 0..4 {
        for x in 0..4 {
            let subset = subset_index(partition, i, p.subsets) as usize;
            let index: u8;
            if is_anchor_index(i, partition, p.subsets) {
                index = reader.read_bits_as_u8((color_index_bits - 1).into());
            } else {
                index = reader.read_bits_as_u8(color_index_bits.into());
            }
            colors[x][y].r = interpolate(endpoints[subset][0][0], endpoints[subset][1][0], index, color_index_bits);
            colors[x][y].g = interpolate(endpoints[subset][0][1], endpoints[subset][1][1], index, color_index_bits);
            colors[x][y].b = interpolate(endpoints[subset][0][2], endpoints[subset][1][2], index, color_index_bits);
            i += 1;
        }
    }

    // read the alpha index bits (if there are any) and interpolate the alpha values; otherwise fill alpha channel with max value
    if p.alpha_bits > 0 {
        reader.seek(alpha_indexes_offset);
        i = 0;
        for y in 0..4 {
            for x in 0..4 {
                let subset = subset_index(partition, i, p.subsets) as usize;
                let index: u8;
                if is_anchor_index(i, partition, p.subsets) {
                    index = reader.read_bits_as_u8((alpha_index_bits - 1).into());
                } else {
                    index = reader.read_bits_as_u8(alpha_index_bits.into());
                }
                colors[x][y].a = interpolate(endpoints[subset][0][3], endpoints[subset][1][3], index, alpha_index_bits);
                i += 1;
            }
        }
    } else {
        for y in 0..4 {
            for x in 0..4 {
                colors[x][y].a = 255;
            }
        }
    }

    // rotate color and alpha if necessary
    if rotation != 0 {
        for y in 0..4 {
            for x in 0..4 {
                match rotation {
                    0 => unreachable!("we only run the loop if rotation != 0"),
                    1 => std::mem::swap(&mut colors[x][y].a, &mut colors[x][y].r),
                    2 => std::mem::swap(&mut colors[x][y].a, &mut colors[x][y].g),
                    3 => std::mem::swap(&mut colors[x][y].a, &mut colors[x][y].b),
                    _ => unreachable!("invalid rotation number {rotation} should never be constructed"),
                }
            }
        }
    }

    colors
}

#[derive(Clone, Debug)]
pub enum BC7DecodeError {
    DataSize,
}

impl std::fmt::Display for BC7DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            BC7DecodeError::DataSize => write!(f, "incorrect data size"),
        }
    }
}

impl std::error::Error for BC7DecodeError {}

/// Decode a BC7 encoded image with blocks in x-minor order.
///
/// Returns colors with pixels in x-minor order.
#[allow(clippy::needless_range_loop)]
pub fn decode(data: &[u8], width: usize, height: usize) -> Result<std::vec::Vec<Color>, BC7DecodeError> {
    let nblocks_x = width / 4 + ((width % 4) != 0) as usize;
    let nblocks_y = height / 4 + ((height % 4) != 0) as usize;

    if data.len() != nblocks_x * nblocks_y * 16 {
        return Err(BC7DecodeError::DataSize);
    }

    let mut result = std::vec::Vec::new();
    result.resize(width * height, Color::default());

    let mut off = 0;
    for y0 in (0..height).step_by(4) {
        for x0 in (0..width).step_by(4) {
            let block_colors = decode_block(&Bc7BlockSlice::from_slice(&data[off..off + 16]));
            for dx in 0..4 {
                for dy in 0..4 {
                    let x = x0 + dx;
                    let y = y0 + dy;
                    if x >= width || y >= height {
                        continue;
                    }
                    let i = x + y * width;
                    result[i] = block_colors[dx][dy];
                }
            }
            off += 16;
        }
    }

    Ok(result)
}
