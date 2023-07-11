#![feature(abi_thiscall)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

mod console;
mod script;
mod graphics;

use std::{ffi::c_char, os::raw::c_void};

use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

use wow_mem::detour_fn;

macro_rules! ptr {
    ($address:expr, $type:ty) => {
        *($address as *mut $type)
    };
}


extern "fastcall" fn cmd_test(_cmd: *const c_char, _args: *const c_char) -> u32 {
    console::console_write("this is only a test", console::ConsoleColor::Error);
    graphics::init();
    return 0;
}

fn init_extension() {
    unsafe {
        // Fix InvalidPtrCheck for callbacks outside of .text section
        ptr!(0x00884800, u32) = 0x00000001;
        ptr!(0x00884C00, u32) = 0x7FFFFFFF;
    }

    console::console_write("wow112_extension loaded!", console::ConsoleColor::Warning);
    console::console_command_register("test", cmd_test, console::CommandCategory::Debug, "uhhh");

    script::init();
    console::init();
}

#[detour_fn(0x0046B840)]
unsafe extern "thiscall" fn sub_46B840(a1: u32) {
    init_extension();

    hook_sub_46B840.disable().unwrap();
    hook_sub_46B840.call(a1);
}

static mut ExtensionLoaded: bool = false;

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH && !ExtensionLoaded {
        hook_sub_46B840.enable().unwrap();
        ExtensionLoaded = true;
    }

    return BOOL::from(true);
}
