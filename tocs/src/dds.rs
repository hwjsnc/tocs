/*
loading dds textures

resources:
- texconv, a DX texture conversion tool: https://github.com/microsoft/DirectXTex/wiki/Texconv
- C++ code for bare-bones DDS texture loader https://learn.microsoft.com/en-us/windows/uwp/gaming/complete-code-for-ddstextureloader
- texture block compression: https://learn.microsoft.com/en-us/windows/uwp/graphics-concepts/texture-block-compression
- BC7 (aka BPTC) texture compression format: https://registry.khronos.org/DataFormat/specs/1.3/dataformat.1.3.html#bptc_bc7
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

pub fn write_header<F: std::io::Write>(f: &mut F, header: &Header) -> std::io::Result<()> {
    let BaseHeader {
        size,
        flags,
        height,
        width,
        pitch_or_linear_size,
        depth,
        mip_map_count,
        reserved_1,
        dds_pixel_format,
        caps,
        reserved_2,
    } = match header {
        Header::Base(h) => h,
        Header::WithDxt10(base, ext) => base,
    };

    f.write(&size.to_le_bytes())?;
    f.write(&flags.to_le_bytes())?;
    f.write(&height.to_le_bytes())?;
    f.write(&width.to_le_bytes())?;
    f.write(&pitch_or_linear_size.to_le_bytes())?;
    f.write(&depth.to_le_bytes())?;
    f.write(&mip_map_count.to_le_bytes())?;
    for x in reserved_1 {
        f.write(&x.to_le_bytes())?;
    }
    f.write(&dds_pixel_format.size.to_le_bytes())?;
    f.write(&dds_pixel_format.flags.to_le_bytes())?;
    f.write(&dds_pixel_format.id)?;
    f.write(&dds_pixel_format.rgb_bit_count.to_le_bytes())?;
    f.write(&dds_pixel_format.r_bit_mask.to_le_bytes())?;
    f.write(&dds_pixel_format.g_bit_mask.to_le_bytes())?;
    f.write(&dds_pixel_format.b_bit_mask.to_le_bytes())?;
    f.write(&dds_pixel_format.a_bit_mask.to_le_bytes())?;
    for x in caps {
        f.write(&x.to_le_bytes())?;
    }
    f.write(&reserved_2.to_le_bytes())?;

    if let Header::WithDxt10(_, ext) = header {
        let Dxt10ExtensionHeader {
            dxgi_format,
            resource_dimension,
            misc_flag,
            array_size,
            reserved,
        } = ext;
        f.write(&dxgi_format.to_le_bytes())?;
        f.write(&resource_dimension.to_le_bytes())?;
        f.write(&misc_flag.to_le_bytes())?;
        f.write(&array_size.to_le_bytes())?;
        f.write(&reserved.to_le_bytes())?;
    }

    Ok(())
}

pub mod dxgi_format {
    // the following values are taken from https://learn.microsoft.com/en-us/windows/win32/api/dxgiformat/ne-dxgiformat-dxgi_format
    pub const UNKNOWN: u32 = 0;
    pub const R32G32B32A32_TYPELESS: u32 = 1;
    pub const R32G32B32A32_FLOAT: u32 = 2;
    pub const R32G32B32A32_UINT: u32 = 3;
    pub const R32G32B32A32_SINT: u32 = 4;
    pub const R32G32B32_TYPELESS: u32 = 5;
    pub const R32G32B32_FLOAT: u32 = 6;
    pub const R32G32B32_UINT: u32 = 7;
    pub const R32G32B32_SINT: u32 = 8;
    pub const R16G16B16A16_TYPELESS: u32 = 9;
    pub const R16G16B16A16_FLOAT: u32 = 10;
    pub const R16G16B16A16_UNORM: u32 = 11;
    pub const R16G16B16A16_UINT: u32 = 12;
    pub const R16G16B16A16_SNORM: u32 = 13;
    pub const R16G16B16A16_SINT: u32 = 14;
    pub const R32G32_TYPELESS: u32 = 15;
    pub const R32G32_FLOAT: u32 = 16;
    pub const R32G32_UINT: u32 = 17;
    pub const R32G32_SINT: u32 = 18;
    pub const R32G8X24_TYPELESS: u32 = 19;
    pub const D32_FLOAT_S8X24_UINT: u32 = 20;
    pub const R32_FLOAT_X8X24_TYPELESS: u32 = 21;
    pub const X32_TYPELESS_G8X24_UINT: u32 = 22;
    pub const R10G10B10A2_TYPELESS: u32 = 23;
    pub const R10G10B10A2_UNORM: u32 = 24;
    pub const R10G10B10A2_UINT: u32 = 25;
    pub const R11G11B10_FLOAT: u32 = 26;
    pub const R8G8B8A8_TYPELESS: u32 = 27;
    pub const R8G8B8A8_UNORM: u32 = 28;
    pub const R8G8B8A8_UNORM_SRGB: u32 = 29;
    pub const R8G8B8A8_UINT: u32 = 30;
    pub const R8G8B8A8_SNORM: u32 = 31;
    pub const R8G8B8A8_SINT: u32 = 32;
    pub const R16G16_TYPELESS: u32 = 33;
    pub const R16G16_FLOAT: u32 = 34;
    pub const R16G16_UNORM: u32 = 35;
    pub const R16G16_UINT: u32 = 36;
    pub const R16G16_SNORM: u32 = 37;
    pub const R16G16_SINT: u32 = 38;
    pub const R32_TYPELESS: u32 = 39;
    pub const D32_FLOAT: u32 = 40;
    pub const R32_FLOAT: u32 = 41;
    pub const R32_UINT: u32 = 42;
    pub const R32_SINT: u32 = 43;
    pub const R24G8_TYPELESS: u32 = 44;
    pub const D24_UNORM_S8_UINT: u32 = 45;
    pub const R24_UNORM_X8_TYPELESS: u32 = 46;
    pub const X24_TYPELESS_G8_UINT: u32 = 47;
    pub const R8G8_TYPELESS: u32 = 48;
    pub const R8G8_UNORM: u32 = 49;
    pub const R8G8_UINT: u32 = 50;
    pub const R8G8_SNORM: u32 = 51;
    pub const R8G8_SINT: u32 = 52;
    pub const R16_TYPELESS: u32 = 53;
    pub const R16_FLOAT: u32 = 54;
    pub const D16_UNORM: u32 = 55;
    pub const R16_UNORM: u32 = 56;
    pub const R16_UINT: u32 = 57;
    pub const R16_SNORM: u32 = 58;
    pub const R16_SINT: u32 = 59;
    pub const R8_TYPELESS: u32 = 60;
    pub const R8_UNORM: u32 = 61;
    pub const R8_UINT: u32 = 62;
    pub const R8_SNORM: u32 = 63;
    pub const R8_SINT: u32 = 64;
    pub const A8_UNORM: u32 = 65;
    pub const R1_UNORM: u32 = 66;
    pub const R9G9B9E5_SHAREDEXP: u32 = 67;
    pub const R8G8_B8G8_UNORM: u32 = 68;
    pub const G8R8_G8B8_UNORM: u32 = 69;
    pub const BC1_TYPELESS: u32 = 70;
    pub const BC1_UNORM: u32 = 71;
    pub const BC1_UNORM_SRGB: u32 = 72;
    pub const BC2_TYPELESS: u32 = 73;
    pub const BC2_UNORM: u32 = 74;
    pub const BC2_UNORM_SRGB: u32 = 75;
    pub const BC3_TYPELESS: u32 = 76;
    pub const BC3_UNORM: u32 = 77;
    pub const BC3_UNORM_SRGB: u32 = 78;
    pub const BC4_TYPELESS: u32 = 79;
    pub const BC4_UNORM: u32 = 80;
    pub const BC4_SNORM: u32 = 81;
    pub const BC5_TYPELESS: u32 = 82;
    pub const BC5_UNORM: u32 = 83;
    pub const BC5_SNORM: u32 = 84;
    pub const B5G6R5_UNORM: u32 = 85;
    pub const B5G5R5A1_UNORM: u32 = 86;
    pub const B8G8R8A8_UNORM: u32 = 87;
    pub const B8G8R8X8_UNORM: u32 = 88;
    pub const R10G10B10_XR_BIAS_A2_UNORM: u32 = 89;
    pub const B8G8R8A8_TYPELESS: u32 = 90;
    pub const B8G8R8A8_UNORM_SRGB: u32 = 91;
    pub const B8G8R8X8_TYPELESS: u32 = 92;
    pub const B8G8R8X8_UNORM_SRGB: u32 = 93;
    pub const BC6H_TYPELESS: u32 = 94;
    pub const BC6H_UF16: u32 = 95;
    pub const BC6H_SF16: u32 = 96;
    pub const BC7_TYPELESS: u32 = 97;
    pub const BC7_UNORM: u32 = 98;
    pub const BC7_UNORM_SRGB: u32 = 99;
    pub const AYUV: u32 = 100;
    pub const Y410: u32 = 101;
    pub const Y416: u32 = 102;
    pub const NV12: u32 = 103;
    pub const P010: u32 = 104;
    pub const P016: u32 = 105;
    pub const _420_OPAQUE: u32 = 106;
    pub const YUY2: u32 = 107;
    pub const Y210: u32 = 108;
    pub const Y216: u32 = 109;
    pub const NV11: u32 = 110;
    pub const AI44: u32 = 111;
    pub const IA44: u32 = 112;
    pub const P8: u32 = 113;
    pub const A8P8: u32 = 114;
    pub const B4G4R4A4_UNORM: u32 = 115;
    pub const P208: u32 = 130;
    pub const V208: u32 = 131;
    pub const V408: u32 = 132;
    pub const SAMPLER_FEEDBACK_MIN_MIP_OPAQUE: u32 = 0xfffffff;
    pub const SAMPLER_FEEDBACK_MIP_REGION_USED_OPAQUE: u32 = 0xfffffff;
    pub const FORCE_UINT: u32 = 0xfffffff;
}
