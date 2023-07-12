#![allow(dead_code, unused_imports)]
use std::ffi::c_char;
use std::mem::size_of;
use winapi::ctypes::c_void;
use winapi::shared::d3d9types::{D3DCOLOR, D3DMATRIX, D3DPRESENT_PARAMETERS, D3DDEVICE_CREATION_PARAMETERS};
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
    color: u32,
}

static mut TRIS_INITIALIZED: bool = false;

static mut buf_ptr: *mut IDirect3DVertexBuffer9 = std::ptr::null_mut();
const FVF: u32 = d3d9types::D3DFVF_XYZ | d3d9types::D3DFVF_DIFFUSE;

unsafe fn init_tris(dev: &IDirect3DDevice9) {
    // let mut ctx = imgui::Context::create();
    // imgui_dx9_renderer::Renderer::new_raw(&mut ctx, *dev);

    let left = 50.0;
    let right = left + 150.0;
    let top = 50.0;
    let bottom = top + 150.0;

    let verts = vec![
        Vertex { x: left, y: bottom, z: 1.0,  color: 0xFF_8E67D6 },
        Vertex { x: left, y: top, z: 1.0, color: 0xFF_8E67D6 },
        Vertex { x: right, y: top, z: 1.0, color: 0xFF_8E67D6 },
        Vertex { x: right, y: top, z: 1.0, color: 0xFF_8E67D6 },
        Vertex { x: right, y: bottom, z: 1.0, color: 0xFF_8E67D6 },
        Vertex { x: left, y: bottom, z: 1.0, color: 0xFF_8E67D6 },
    ];

    let res = IDirect3DDevice9::CreateVertexBuffer(
        dev,
        (verts.len() * std::mem::size_of::<Vertex>()) as u32,
        0,
        FVF,
        d3d9types::D3DPOOL_MANAGED,
        &mut buf_ptr,
        0 as *mut *mut winapi::ctypes::c_void,
    );

    let text = format!("res = {}", res);
    console_write(&text, crate::console::ConsoleColor::Warning);

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


fn create_orthographic_projection(near: f32, far: f32) -> D3DMATRIX {
    let mut projection = D3DMATRIX {
        m: [[0.0; 4]; 4],
    };

    let r = unsafe { &*std::mem::transmute::<u32, *const d3d9types::D3DRECT>(0x00884E20) };

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

// m: [[c_float; 4]; 4],
const tri_view_matrix: D3DMATRIX = D3DMATRIX {
    m: [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ],
};

unsafe fn draw_tris(dev: &IDirect3DDevice9) {
    // dev.SetTransform(d3d9types::D3DTS_VIEW, &tri_view_matrix);
    // dev.SetTransform(d3d9types::D3DTS_WORLD, &tri_view_matrix);

    let mut old_projection: D3DMATRIX = D3DMATRIX {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    dev.GetTransform(d3d9types::D3DTS_PROJECTION, &mut old_projection);
    let projection = create_orthographic_projection(0.01, 10000.0);
    dev.SetTransform(d3d9types::D3DTS_PROJECTION, &projection);

    // dev.SetRenderState(d3d9types::D3DRS_ZENABLE, 0);
    // dev.SetRenderState(d3d9types::D3DRS_SCISSORTESTENABLE, 0);
    // dev.SetRenderState(d3d9types::D3DRS_ALPHABLENDENABLE, 0);
    // dev.SetRenderState(d3d9types::D3DRS_BLENDOP, d3d9types::D3DBLENDOP_ADD);
    // dev.SetRenderState(d3d9types::D3DRS_SRCBLEND, d3d9types::D3DBLEND_ONE);
    // dev.SetRenderState(d3d9types::D3DRS_DESTBLEND, d3d9types::D3DBLEND_ZERO);
    // dev.SetRenderState(d3d9types::D3DRS_COLORVERTEX, 0);
    // dev.SetRenderState(d3d9types::D3DRS_TEXTUREFACTOR, 0xFFFF_FFFF);
    // dev.SetRenderState(d3d9types::D3DRS_LIGHTING, 0);

    // let win_rect = unsafe { &*std::mem::transmute::<u32, *const d3d9types::D3DRECT>(0x00884E20) };

    // let mut create_params = D3DDEVICE_CREATION_PARAMETERS {
    //     AdapterOrdinal: 0,
    //     DeviceType: 0,
    //     hFocusWindow: std::ptr::null_mut(),
    //     BehaviorFlags: 0,
    // };

    // dev.GetCreationParameters(&mut create_params);

    // let mut params = D3DPRESENT_PARAMETERS {
    //     BackBufferWidth: win_rect.x2 as u32,
    //     BackBufferHeight: win_rect.y2 as u32,
    //     BackBufferFormat: d3d9types::D3DFMT_R8G8B8,
    //     BackBufferCount: 1,
    //     MultiSampleType: d3d9types::D3DMULTISAMPLE_NONE,
    //     MultiSampleQuality: 0,
    //     SwapEffect: 0,
    //     hDeviceWindow: create_params.hFocusWindow,
    //     Windowed: 1,
    //     EnableAutoDepthStencil: 0,
    //     AutoDepthStencilFormat: 0,
    //     Flags: 0,
    //     FullScreen_RefreshRateInHz: 60,
    //     PresentationInterval: 1,
    // };
    // dev.Reset(&mut params);

    // dev.SetPixelShader(std::ptr::null_mut());
    // dev.SetVertexShader(std::ptr::null_mut());
    
    dev.SetFVF(FVF);
    dev.SetStreamSource(0, buf_ptr, 0, std::mem::size_of::<Vertex>() as u32);
    dev.DrawPrimitive(d3d9types::D3DPT_TRIANGLELIST, 0, 2);

    dev.SetTransform(d3d9types::D3DTS_PROJECTION, &old_projection);
}

static mut TRIANGLE_ENABLED: bool = true;

#[detour_fn(0x005A17A0)]
unsafe extern "thiscall" fn CGxDeviceD3d__ISceneEnd(this: u32) {
    let dev_ptr = *std::mem::transmute::<u32, *const *mut IDirect3DDevice9>(this + 0x38A8);
    let dev: &IDirect3DDevice9 = dev_ptr.as_ref().unwrap();

    if !TRIS_INITIALIZED {
        init_tris(dev);
        TRIS_INITIALIZED = true;
    }

    if TRIANGLE_ENABLED {
        dev.Clear(
            1,
            std::mem::transmute::<u32, *const d3d9types::D3DRECT>(0x00884E20),
            d3d9types::D3DCLEAR_TARGET | d3d9types::D3DCLEAR_ZBUFFER,
            0xFF_000000,
            1.0,
            0,
        );
        
        draw_tris(dev);
    }

    hook_CGxDeviceD3d__ISceneEnd.disable().unwrap();
    hook_CGxDeviceD3d__ISceneEnd.call(this);
    hook_CGxDeviceD3d__ISceneEnd.enable().unwrap();
}

pub fn toggle() {
    unsafe { TRIANGLE_ENABLED = !TRIANGLE_ENABLED };
}

pub fn init() {
    unsafe {
        hook_CGWorldFrame__RenderWorld.enable().unwrap();
        hook_sub_76D240.enable().unwrap();
        hook_sub_787220.enable().unwrap();
        hook_CGxDeviceD3d__ISceneEnd.enable().unwrap();
    }
}
