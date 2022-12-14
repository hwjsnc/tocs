/*
loading dds textures

resources:
- texconv, a DX texture conversion tool: https://github.com/microsoft/DirectXTex/wiki/Texconv
- C++ code for bare-bones DDS texture loader https://learn.microsoft.com/en-us/windows/uwp/gaming/complete-code-for-ddstextureloader
- texture block compression: https://learn.microsoft.com/en-us/windows/uwp/graphics-concepts/texture-block-compression
- BC6/BC7 (aka BPTC_FLOAT/BPTC) texture compression format: https://registry.khronos.org/OpenGL/extensions/ARB/ARB_texture_compression_bptc.txt
- BC7 texture format (but incomplete): https://learn.microsoft.com/en-us/windows/uwp/graphics-concepts/bc7-format
*/

#[derive(Clone, Copy, Debug)]
pub struct PixelFormat {
    pub size: u32,
    pub flags: u32,
    pub id: [u8; 4],
    pub rgb_bit_count: u32,
    pub r_bit_mask: u32,
    pub g_bit_mask: u32,
    pub b_bit_mask: u32,
    pub a_bit_mask: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct BaseHeader {
    pub size: u32,
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mip_map_count: u32,
    pub reserved_1: [u32; 11],
    pub dds_pixel_format: PixelFormat,
    pub caps: [u32; 4],
    pub reserved_2: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Dxt10ExtensionHeader {
    pub dxgi_format: u32,
    pub resource_dimension: u32,
    pub misc_flag: u32,
    pub array_size: u32,
    pub reserved: u32,
}

#[derive(Clone, Copy, Debug)]
pub enum Header {
    Base(BaseHeader),
    WithDxt10(BaseHeader, Dxt10ExtensionHeader),
}

#[derive(Clone, Copy, Debug)]
pub struct TextureRef<'a> {
    pub header: Header,
    pub raw_data: &'a [u8],
}

impl PixelFormat {
    fn from_slice(slice: &[u8]) -> Self {
        assert_eq!(slice.len(), 32);
        Self {
            size: u32::from_le_bytes(slice[0..4].try_into().expect("4 bytes")),
            flags: u32::from_le_bytes(slice[4..8].try_into().expect("4 bytes")),
            id: slice[8..12].try_into().expect("4 bytes"),
            rgb_bit_count: u32::from_le_bytes(slice[12..16].try_into().expect("4 bytes")),
            r_bit_mask: u32::from_le_bytes(slice[16..20].try_into().expect("4 bytes")),
            g_bit_mask: u32::from_le_bytes(slice[20..24].try_into().expect("4 bytes")),
            b_bit_mask: u32::from_le_bytes(slice[24..28].try_into().expect("4 bytes")),
            a_bit_mask: u32::from_le_bytes(slice[28..32].try_into().expect("4 bytes")),
        }
    }
}

impl BaseHeader {
    fn from_slice(slice: &[u8]) -> Self {
        assert_eq!(slice.len(), 23 * 4 + 32);
        Self {
            size: u32::from_le_bytes(slice[0..4].try_into().expect("4 bytes")),
            flags: u32::from_le_bytes(slice[4..8].try_into().expect("4 bytes")),
            height: u32::from_le_bytes(slice[8..12].try_into().expect("4 bytes")),
            width: u32::from_le_bytes(slice[12..16].try_into().expect("4 bytes")),
            pitch_or_linear_size: u32::from_le_bytes(slice[16..20].try_into().expect("4 bytes")),
            depth: u32::from_le_bytes(slice[20..24].try_into().expect("4 bytes")),
            mip_map_count: u32::from_le_bytes(slice[24..28].try_into().expect("4 bytes")),
            reserved_1: [
                u32::from_le_bytes(slice[28..32].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[32..36].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[36..40].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[40..44].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[44..48].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[48..52].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[52..56].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[56..60].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[60..64].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[64..68].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[68..72].try_into().expect("4 bytes")),
            ],
            dds_pixel_format: PixelFormat::from_slice(&slice[72..104]),
            caps: [
                u32::from_le_bytes(slice[104..108].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[108..112].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[112..116].try_into().expect("4 bytes")),
                u32::from_le_bytes(slice[116..120].try_into().expect("4 bytes")),
            ],
            reserved_2: u32::from_le_bytes(slice[120..124].try_into().expect("4 bytes")),
        }
    }
}

impl Dxt10ExtensionHeader {
    fn from_slice(slice: &[u8]) -> Self {
        assert_eq!(slice.len(), 20);
        Self {
            dxgi_format: u32::from_le_bytes(slice[0..4].try_into().expect("4 bytes")),
            resource_dimension: u32::from_le_bytes(slice[4..8].try_into().expect("4 bytes")),
            misc_flag: u32::from_le_bytes(slice[8..12].try_into().expect("4 bytes")),
            array_size: u32::from_le_bytes(slice[12..16].try_into().expect("4 bytes")),
            reserved: u32::from_le_bytes(slice[16..20].try_into().expect("4 bytes")),
        }
    }
}

pub fn load_texture(data: &[u8]) -> Result<TextureRef<'_>, String> {
    if data.len() < 124 {
        return Err(format!("data too short ({} bytes long)", data.len()));
    }

    let magic_header: [u8; 4] = data[0..4].try_into().expect("we checked the length earlier");
    if magic_header != ['D' as u8, 'D' as u8, 'S' as u8, ' ' as u8] {
        return Err(format!("unexpected magic header '{:?}'", magic_header));
    }

    let base_header = BaseHeader::from_slice(&data[4..128]);

    if (base_header.flags & 4) != 0 && base_header.dds_pixel_format.id == ['D' as u8, 'X' as u8, '1' as u8, '0' as u8] {
        if data.len() < 148 {
            return Err(format!("data too short ({} bytes long)", data.len()));
        }
        let dx10_ext_header = Dxt10ExtensionHeader::from_slice(&data[128..148]);

        Ok(TextureRef {
            header: Header::WithDxt10(base_header, dx10_ext_header),
            raw_data: &data[148..],
        })
    } else {
        Ok(TextureRef {
            header: Header::Base(base_header),
            raw_data: &data[128..],
        })
    }
}
