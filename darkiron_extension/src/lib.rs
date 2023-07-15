#![feature(abi_thiscall, slice_flatten)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

mod config;
mod console;
mod gl;
mod graphics;
mod math;
mod script;

use std::ffi::{c_char, c_void};

use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

use darkiron_macro::detour_fn;

pub mod mem {
    pub unsafe fn ptr<T>(addr: u32) -> *mut T {
        addr as *mut T
    }

    pub unsafe fn set<T>(addr: u32, value: T) {
        *(addr as *mut T) = value
    }
}

extern "fastcall" fn cmd_test(_cmd: *const c_char, _args: *const c_char) -> u32 {
    console::console_write("this is only a test", console::ConsoleColor::Warning);
    graphics::toggle();
    return 0;
}

fn init_extension() {
    unsafe {
        // Fix InvalidPtrCheck for callbacks outside of .text section
        mem::set(0x00884800, 0x00000001u32);
        mem::set(0x00884C00, 0x7FFFFFFFu32);
    }

    script::init();
    console::init();

    console::console_write("Dark Iron extension loaded!", console::ConsoleColor::Admin);
    console::console_command_register("test", cmd_test, console::CommandCategory::Debug, None);
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
        graphics::init();
        hook_sub_46B840.enable().unwrap();
        ExtensionLoaded = true;
    }

    return BOOL::from(true);
}
