#![allow(dead_code, unused_imports)]
use std::ffi::c_char;
use std::mem::size_of;
use winapi::ctypes::c_void;
use winapi::shared::d3d9types::D3DCOLOR;
use winapi::shared::{
    d3d9,
    d3d9::{IDirect3DDevice9, IDirect3DVertexBuffer9},
    d3d9types,
};
use winapi::um::winnt::RtlCopyMemory;
use wow_mem::detour_fn;

use crate::console::console_write;

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

#[repr(C)]
struct Vertex {
    x: f32,
    y: f32,
    z: f32,
    rhw: f32,
    color: u32,
}

impl Vertex {
    pub fn new(x: f32, y: f32, z: f32, rhw: f32, color: u32) -> Self {
        Vertex {
            x: x,
            y: y,
            z: z,
            rhw: rhw,
            color: color,
        }
    }
}

static mut HACK_INITIALIZED: bool = false;

static mut buf_ptr: *mut IDirect3DVertexBuffer9 = std::ptr::null_mut();
const FVF: u32 = d3d9types::D3DFVF_XYZRHW | d3d9types::D3DFVF_DIFFUSE;

unsafe fn init_triangle(dev: &IDirect3DDevice9) {
    let verts = vec![
        Vertex::new(120.0, 400.0, 1.0, 1.0, 0xFFFFFFFF),
        Vertex::new(120.0, 50.0, 1.0, 1.0, 0xFFFFFFFF),
        Vertex::new(520.0, 50.0, 1.0, 1.0, 0xFFFFFFFF),
        Vertex::new(520.0, 400.0, 1.0, 1.0, 0xFFFFFFFF),
    ];

    IDirect3DDevice9::CreateVertexBuffer(
        dev,
        (verts.len() * std::mem::size_of::<Vertex>()) as u32,
        0,
        FVF,
        d3d9types::D3DPOOL_MANAGED,
        &mut buf_ptr,
        0 as *mut *mut winapi::ctypes::c_void,
    );

    let mut pVoid: *mut c_void = std::ptr::null_mut();
    let ppVoid: *mut *mut c_void = &mut pVoid;

    IDirect3DVertexBuffer9::Lock(&*buf_ptr, 0, 0, ppVoid, 0);

    RtlCopyMemory(
        pVoid,
        verts.as_ptr() as *const c_void,
        verts.len() * std::mem::size_of::<Vertex>(),
    );

    IDirect3DVertexBuffer9::Unlock(&*buf_ptr);
}

unsafe fn draw_triangle(dev: &IDirect3DDevice9) {
    IDirect3DDevice9::SetFVF(dev, FVF);
    IDirect3DDevice9::SetStreamSource(dev, 0, buf_ptr, 0, std::mem::size_of::<Vertex>() as u32);
    IDirect3DDevice9::DrawPrimitive(dev, d3d9types::D3DPT_TRIANGLEFAN, 0, 2);
}

#[detour_fn(0x005A17A0)]
unsafe extern "thiscall" fn CGxDeviceD3d__ISceneEnd(this: u32) {
    let dev_ptr = *std::mem::transmute::<u32, *const *mut IDirect3DDevice9>(this + 0x38A8);
    let dev: &IDirect3DDevice9 = dev_ptr.as_ref().unwrap();

    if !HACK_INITIALIZED {
        init_triangle(dev);
        HACK_INITIALIZED = true;
    }

    let rect = d3d9types::D3DRECT {
        x1: 0,
        y1: 0,
        x2: 512,
        y2: 512,
    };

    dev.Clear(
        1,
        &rect,
        d3d9types::D3DCLEAR_TARGET | d3d9types::D3DCLEAR_ZBUFFER,
        0xFF000000,
        1.0,
        0,
    );

    draw_triangle(dev);

    hook_CGxDeviceD3d__ISceneEnd.disable().unwrap();
    hook_CGxDeviceD3d__ISceneEnd.call(this);
    hook_CGxDeviceD3d__ISceneEnd.enable().unwrap();
}

pub fn init() {
    unsafe {
        hook_CGWorldFrame__RenderWorld.enable().unwrap();
        hook_sub_76D240.enable().unwrap();
        hook_sub_787220.enable().unwrap();
        hook_CGxDeviceD3d__ISceneEnd.enable().unwrap();

        // std::memory::transmute::<u32, *mut IDirect3DDevice9>(0xC0ED38)
    }
}
