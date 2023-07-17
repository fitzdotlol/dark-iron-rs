use windows::Win32::Graphics::OpenGL::{GL_RGBA8, GL_RGBA4, GL_RGB5, GL_RGB, GL_RGB5_A1, GL_BYTE};

use super::gl;

static TextureBindTargets: [u32; 3] = [gl::TEXTURE_2D, gl::TEXTURE_CUBE_MAP, gl::TEXTURE_RECTANGLE];

struct TextureFormat {
    internal_format: u32,
    format: u32,
    ty: u32,
    unk: u32,
}

static TextureFormats: [TextureFormat; 9] = [
    TextureFormat {
        internal_format: gl::NONE,
        format: gl::NONE,
        ty: gl::NONE,
        unk: 0,
    },
    TextureFormat {
        internal_format: GL_RGBA8,
        format: gl::BGRA,
        ty: gl::UNSIGNED_INT_8_8_8_8_REV,
        unk: 4,
    },
    TextureFormat {
        internal_format: GL_RGBA4,
        format: gl::BGRA,
        ty: gl::UNSIGNED_SHORT_4_4_4_4_REV,
        unk: 2,
    },
    TextureFormat {
        internal_format: GL_RGB5_A1,
        format: gl::BGRA,
        ty: gl::UNSIGNED_SHORT_1_5_5_5_REV,
        unk: 2,
    },
    TextureFormat {
        internal_format: GL_RGB5,
        format: GL_RGB,
        ty: gl::UNSIGNED_SHORT_5_6_5,
        unk: 2,
    },
    TextureFormat {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT1_EXT,
        format: gl::NONE,
        ty: gl::NONE,
        unk: 2,
    },
    TextureFormat {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT3_EXT,
        format: gl::NONE,
        ty: gl::NONE,
        unk: 0,
    },
    TextureFormat {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT5_EXT,
        format: gl::NONE,
        ty: gl::NONE,
        unk: 0,
    },
    TextureFormat {
        internal_format: gl::DSDT8_NV,
        format: gl::DSDT_NV,
        ty: GL_BYTE,
        unk: 2,
    },
];

struct OpenGlTexture {
    unk_00: u32,
    unk_01: u32,
    unk_02: u32,
    unk_03: u32,
    unk_04: u32,
    unk_05: u32,
    unk_06: u32,
    unk_07: u32,
    unk_08: u32,
    unk_09: u32,
    unk_10: u32,
    unk_11: u32,
    bind_target: u32,     // 12, index into TextureBindTargets
    internal_format: u32, // 13
    format: u32,          // 14, index into TextureFormats
    unk_15: u32,
    unk_16: u32,
    unk_17: u32,
    gl_id: u32, // 18
}

#[derive(Debug)]
#[repr(C)]
pub struct OpenGlWindow {
    pub unk_00: u32,
    pub unk_01: u32,
    pub unk_02: u32,
    pub unk_03: u32,
    pub width: i32,  // 4
    pub height: i32, // 5
    pub unk_06: u32,
    pub unk_07: u32,
    pub unk_08: u32,
    pub unk_09: u32,
    pub unk_10: u32,
    pub unk_11: u32,
    pub x: i32, // 12
    pub y: i32, // 13
}

struct DeviceOpenGl {
    // HWND: + 0x39E4
    // HDC: + 0x39F0
}
