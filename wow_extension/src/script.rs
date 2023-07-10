use std::ffi::{c_char, c_void};

use wow_mem::detour_fn;

//int __fastcall FrameScript::Register(const char *name, int a2)
#[detour_fn(0x00704120)]
pub extern "fastcall" fn FrameScript__Register(name: *const c_char, func: *const c_void)
{
    unsafe {
        hook_FrameScript__Register.disable().unwrap();
        hook_FrameScript__Register.call(name, func);
        hook_FrameScript__Register.enable().unwrap();
    }
}

pub fn init() {
    unsafe {
        hook_FrameScript__Register.enable().unwrap();
    }
}
