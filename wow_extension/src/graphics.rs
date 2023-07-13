#![allow(dead_code, unused_imports)]
use std::ffi::c_char;
use std::mem::size_of;
use winapi::ctypes::c_void;
use winapi::um::winnt::RtlCopyMemory;

use windows::Win32::Graphics::OpenGL::*;
use wow_mem::detour_fn;

use crate::console::console_write;
use crate::math::{RectI, Matrix4};

#[detour_fn(0x00482D70)]
extern "thiscall" fn CGWorldFrame__RenderWorld(this: *const c_void) {
    unsafe {
        hook_CGWorldFrame__RenderWorld.disable().unwrap();
        hook_CGWorldFrame__RenderWorld.call(this);
        hook_CGWorldFrame__RenderWorld.enable().unwrap();
    }
}

// menu scene render callback
#[detour_fn(0x76D240)]
extern "thiscall" fn sub_76D240(this: *const c_void) {
    unsafe {
        hook_sub_76D240.disable().unwrap();
        hook_sub_76D240.call(this);
        hook_sub_76D240.enable().unwrap();
    }
}

// breaking news render callback??? that makes no sense, but that's what it looks like.
#[detour_fn(0x787220)]
extern "thiscall" fn sub_787220(this: *const c_void) {
    unsafe {
        hook_sub_787220.disable().unwrap();
        hook_sub_787220.call(this);
        hook_sub_787220.enable().unwrap();
    }
}

static mut TRIS_INITIALIZED: bool = false;

fn create_orthographic_projection(near: f32, far: f32) -> Matrix4 {
    let mut projection = Matrix4 {
        m: [[0.0; 4]; 4],
    };

    let r = unsafe { &*std::mem::transmute::<u32, *const RectI>(0x00884E20) };

    let left = r.x1 as f32;
    let right = r.x2 as f32;
    let top = r.y1 as f32;
    let bottom = r.y2 as f32;

    projection.m[0][0] = 2.0 / (right - left);
    projection.m[1][1] = 2.0 / (top - bottom);
    projection.m[2][2] = 1.0 / (far - near);
    projection.m[3][0] = -(right + left) / (right - left);
    projection.m[3][1] = -(top + bottom) / (top - bottom);
    projection.m[3][2] = -near / (far - near);
    projection.m[3][3] = 1.0;

    projection
}

static mut UI_ENABLED: bool = true;

// binds texture: sub_59F0F0

static TextureBindTargets: [u32; 3] = [
    GL_TEXTURE_2D,
    0x8513, // GL_TEXTURE_CUBE_MAP
    0x84F5, //GL_TEXTURE_RECTANGLE
];

static TextureInternalFormats: [u32; 9] = [
    GL_NONE,
    GL_RGBA8,
    GL_RGBA4,
    GL_RGB5_A1,
    GL_RGB5,
    0x83F1, // GL_COMPRESSED_RGBA_S3TC_DXT1_EXT
    0x83F2, // GL_COMPRESSED_RGBA_S3TC_DXT3_EXT
    0x83F3, // GL_COMPRESSED_RGBA_S3TC_DXT5_EXT
    0x8709, // GL_DSDT8_NV
];

static TextureFormats: [u32; 9] = [
    GL_NONE,
    0x80E1, // GL_BGRA
    0x80E1, // GL_BGRA
    0x80E1, // GL_BGRA
    GL_RGB,
    GL_NONE,
    GL_NONE,
    GL_NONE,
    0x86F5, // GL_DSDT_NV
];

static TextureTypes: [u32; 9] = [
    GL_NONE,
    0x8367, // GL_UNSIGNED_INT_8_8_8_8_REV
    0x8365, // GL_UNSIGNED_SHORT_4_4_4_4_REV
    0x8366, // GL_UNSIGNED_SHORT_1_5_5_5_REV
    0x8363, // GL_UNSIGNED_SHORT_5_6_5
    GL_NONE,
    GL_NONE,
    GL_NONE,
    GL_BYTE,
];

static TextureUnks: [u32; 9] = [
    0,
    4,
    2,
    2,
    2,
    2,
    0,
    0,
    2,
];

struct CGxOpenGlTexture {
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
    bind_target: u32, // 12
    internal_format: u32, // 13
    format: u32, // 14
    unk_15: u32,
    unk_16: u32,
    unk_17: u32,
    gl_id: u32, // 18
}

unsafe fn draw_ui() {
    let projection = create_orthographic_projection(0.01, 10000.0);
    let proj_mtx = projection.m.flatten();
    
    glMatrixMode(GL_PROJECTION);
    glPushMatrix();
    glLoadMatrixf(proj_mtx.as_ptr());
    glMatrixMode(GL_MODELVIEW);
    glPushMatrix();
    glLoadIdentity();

    let width = 500.0;
    let height = 500.0;

    let left = 150.0;
    let right = left + width;
    let top = 50.0;
    let bottom = top + height;

    glBindTexture(GL_TEXTURE_2D, 0);
    glColor3ub(0x8E, 0x67, 0xD6);

    glBegin(GL_QUADS);
    {
        glVertex3f(left, top, 0.0);
        glVertex3f(right, top, 0.0);
        glVertex3f(right, bottom, 0.0);
        glVertex3f(left, bottom, 0.0);
    }
    glEnd();

    glPopMatrix();
    glMatrixMode(GL_PROJECTION);
    glPopMatrix();
}

#[detour_fn(0x0059BA10)]
unsafe extern "thiscall" fn sub_59BA10(this: u32, a2: u32) -> u32 {
    if UI_ENABLED {
        draw_ui();
    }

    hook_sub_59BA10.disable().unwrap();
    let ret_val = hook_sub_59BA10.call(this, a2);
    hook_sub_59BA10.enable().unwrap();

    ret_val
}

pub fn toggle() {
    unsafe { UI_ENABLED = !UI_ENABLED };
}

pub fn init() {
    unsafe {
        hook_CGWorldFrame__RenderWorld.enable().unwrap();
        hook_sub_76D240.enable().unwrap();
        hook_sub_787220.enable().unwrap();

        hook_sub_59BA10.enable().unwrap();
    }
}
