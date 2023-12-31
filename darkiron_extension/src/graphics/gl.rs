#![allow(dead_code)]

use std::ffi::{c_char, c_void, CStr, CString};

use once_cell::sync::Lazy;
use windows::core::PCSTR;
use windows::Win32::Graphics::OpenGL::{
    glBindTexture, glDisableClientState, glDrawElements, glEnableClientState, glGenTextures,
    glGetError, glRotatef, glTexImage2D, glTexParameteri, glVertexPointer, gluErrorString,
    wglGetProcAddress, glTexCoordPointer,
};

use crate::console::console_write;

pub const NONE: u32 = 0x0000;

pub const UNSIGNED_SHORT_5_6_5: u32 = 0x8363;
pub const UNSIGNED_SHORT_4_4_4_4_REV: u32 = 0x8365;
pub const UNSIGNED_SHORT_1_5_5_5_REV: u32 = 0x8366;
pub const UNSIGNED_INT_8_8_8_8_REV: u32 = 0x8367;
pub const DSDT_NV: u32 = 0x86F5;
pub const BGRA: u32 = 0x80E1;
pub const COMPRESSED_RGBA_S3TC_DXT1_EXT: u32 = 0x83F1;
pub const COMPRESSED_RGBA_S3TC_DXT3_EXT: u32 = 0x83F2;
pub const COMPRESSED_RGBA_S3TC_DXT5_EXT: u32 = 0x83F3;
pub const DSDT8_NV: u32 = 0x8709;
pub const TEXTURE_CUBE_MAP: u32 = 0x8513;
pub const TEXTURE_RECTANGLE: u32 = 0x84F5;
pub const ARRAY_BUFFER: u32 = 0x8892;
pub const ELEMENT_ARRAY_BUFFER: u32 = 0x8893;
pub const STATIC_DRAW: u32 = 0x88E4;
pub const TEXTURE_2D: u32 = 0xDE1;

pub const VERTEX_ARRAY: u32 = 0x8074;
pub const NORMAL_ARRAY: u32 = 0x8075;
pub const COLOR_ARRAY: u32 = 0x8076;
pub const TEXTURE_COORD_ARRAY: u32 = 0x8078;

pub const BYTE: u32 = 0x1400;
pub const UNSIGNED_BYTE: u32 = 0x1401;
pub const SHORT: u32 = 0x1402;
pub const UNSIGNED_SHORT: u32 = 0x1403;
pub const INT: u32 = 0x1404;
pub const UNSIGNED_INT: u32 = 0x1405;
pub const FLOAT: u32 = 0x1406;

pub const POINTS: u32 = 0x0000;
pub const LINES: u32 = 0x0001;
pub const LINE_LOOP: u32 = 0x0002;
pub const LINE_STRIP: u32 = 0x0003;
pub const TRIANGLES: u32 = 0x0004;
pub const TRIANGLE_STRIP: u32 = 0x0005;
pub const TRIANGLE_FAN: u32 = 0x0006;

///

fn get_proc_address(name: &str) -> unsafe extern "system" fn() -> isize {
    let name2 = CString::new(name).unwrap();
    let name3 = PCSTR(name2.as_ptr() as *const u8);
    let addr = unsafe { wglGetProcAddress(name3).unwrap() };

    return addr;
}

pub fn check_error() {
    let err_code = unsafe { glGetError() };

    if err_code == 0 {
        return;
    }

    let err_str = unsafe {
        let err_str_ptr = gluErrorString(err_code) as *const c_char;

        CStr::from_ptr(err_str_ptr)
    };

    let text = format!("[ui] glTexImage2D: {}", err_str.to_str().unwrap());
    console_write(&text, crate::console::ConsoleColor::Error);
}

///

type def_glGenBuffers = unsafe extern "system" fn(n: u32, buffers: *mut u32);
static mut glGenBuffers: Lazy<def_glGenBuffers> = Lazy::new(|| {
    let addr = get_proc_address("glGenBuffers");
    return unsafe { std::mem::transmute(addr) };
});

type def_glBindBuffer = unsafe extern "system" fn(target: u32, buffer: u32);
static mut glBindBuffer: Lazy<def_glBindBuffer> = Lazy::new(|| {
    let addr = get_proc_address("glBindBuffer");
    return unsafe { std::mem::transmute(addr) };
});

type def_glBufferData =
    unsafe extern "system" fn(target: u32, size: u32, data: *const c_void, usage: u32);
static mut glBufferData: Lazy<def_glBufferData> = Lazy::new(|| {
    let addr = get_proc_address("glBufferData");
    return unsafe { std::mem::transmute(addr) };
});

type def_glDrawArrays = unsafe extern "system" fn(mode: u32, first: u32, count: u32);
static mut glDrawArrays: Lazy<def_glDrawArrays> = Lazy::new(|| {
    let addr = get_proc_address("glDrawArrays");
    return unsafe { std::mem::transmute(addr) };
});
///

pub fn gen_buffers(n: u32) -> Vec<u32> {
    let mut buffers = Vec::new();
    buffers.resize(n as usize, 0);

    unsafe {
        glGenBuffers(n, buffers.as_mut_ptr());
    }

    buffers
}

pub fn gen_buffer() -> u32 {
    let mut buffer = 0u32;

    unsafe {
        glGenBuffers(1, &mut buffer);
    }

    buffer
}

pub fn bind_buffer(target: u32, buffer: u32) {
    unsafe {
        glBindBuffer(target, buffer);
    }
}

pub fn buffer_data<T>(target: u32, data: &Vec<T>, usage: u32) {
    unsafe {
        glBufferData(
            target,
            (data.len() * std::mem::size_of::<T>()) as u32,
            data.as_ptr() as *const c_void,
            usage,
        );
    }
}

pub fn draw_arrays(mode: u32, first: u32, count: u32) {
    unsafe {
        glDrawArrays(mode, first, count);
    }
}

/////
/// wrappers
/////

pub fn tex_parameter_i(target: u32, param: u32, value: u32) {
    unsafe {
        glTexParameteri(target, param, value as i32);
    }
}

pub fn gen_texture() -> u32 {
    let mut texture = 0u32;

    unsafe {
        glGenTextures(1, &mut texture);
    }

    texture
}

pub fn bind_texture(target: u32, texture: u32) {
    unsafe {
        glBindTexture(target, texture);
    }
}

pub fn tex_image_2d(
    target: u32,
    level: u32,
    internal_format: u32,
    width: u32,
    height: u32,
    border: u32,
    format: u32,
    ty: u32,
    data: *const u8,
) {
    unsafe {
        glTexImage2D(
            target,
            level as i32,
            internal_format as i32,
            width as i32,
            height as i32,
            border as i32,
            format,
            ty,
            data as *const c_void,
        );
    }
}

pub fn rotatef(angle: f32, x: f32, y: f32, z: f32) {
    unsafe {
        glRotatef(angle, x, y, z);
    }
}

pub fn vertex_pointer<T>(offset: u32) {
    unsafe {
        glVertexPointer(
            3,
            FLOAT,
            std::mem::size_of::<T>() as i32,
            offset as *const c_void,
        );
    }
}

pub fn tex_coord_pointer<T>(offset: u32) {
    unsafe {
        glTexCoordPointer(
            2,
            FLOAT,
            std::mem::size_of::<T>() as i32,
            offset as *const c_void,
        );
    }
}

pub fn color_pointer<T>(size: u32, ty: u32, offset: u32) {
    unsafe {
        glVertexPointer(
            size as i32,
            ty,
            std::mem::size_of::<T>() as i32,
            offset as *const c_void,
        );
    }
}


pub fn draw_elements(target: u32, num_indices: usize, ty: u32, offset: u32) {
    unsafe {
        glDrawElements(target, num_indices as i32, ty, offset as *const c_void);
    }
}

pub fn enable_client_state(target: u32) {
    unsafe {
        glEnableClientState(target);
    }
}

pub fn disable_client_state(target: u32) {
    unsafe {
        glDisableClientState(target);
    }
}

// test

#[derive(Default)]
struct Buffer<T> {
    target: u32,
    data: Vec<T>,
    gl_id: u32,
}

impl<T> Buffer<T> {
    fn new(target: u32, data: Vec<T>) -> Self {
        Self {
            target,
            data,
            gl_id: 0,
        }
    }

    fn bind(&self) {

    }

    fn buffer_data(&self, data: Vec<T>, usage: u32) {
        buffer_data(self.target, &data, usage);
    }
}
