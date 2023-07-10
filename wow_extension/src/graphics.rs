use std::ffi::{c_char, c_void};
use winapi::shared::{d3d9, d3d9types, minwindef::BOOL};
use wow_mem::detour_fn;

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

#[detour_fn(0x005A17A0)]
unsafe extern "thiscall" fn CGxDeviceD3d__ISceneEnd(this: u32) {
    let _dev_ptr = *std::mem::transmute::<u32, *const *mut d3d9::IDirect3DDevice9>(this + 0x38A8);
    
    // TODO: do something cool here, I guess

    // let dev = dev_ptr.as_ref().unwrap();
    // d3d9::IDirect3DDevice9::ShowCursor(dev, BOOL::from(false));

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
    }
}
