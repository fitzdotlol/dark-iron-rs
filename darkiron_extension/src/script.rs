use std::ffi::{c_char, c_void};

use darkiron_macro::{detour_fn, enable_detour};

#[detour_fn(0x00704120)]
pub extern "fastcall" fn FrameScript__Register(name: *const c_char, func: *const c_void)
{
    original!(name, func);
}

pub fn init() {
    enable_detour!(FrameScript__Register);
}
