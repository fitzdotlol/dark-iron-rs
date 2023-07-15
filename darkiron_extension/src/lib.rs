#![feature(abi_thiscall, slice_flatten)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

mod config;
mod console;
mod gl;
mod graphics;
mod math;
mod script;

use std::ffi::{c_char, c_void, CStr, CString};

use config::CONFIG;
use console::console_write;
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


// const CHAR *__fastcall sub_703BF0(const char *varName, signed int a2, signed int a3)
#[detour_fn(0x00703BF0)]
unsafe extern "fastcall" fn sub_703BF0(var_name: *const c_char, a2: u32, a3: u32) -> *const i8 {
    hook_sub_703BF0.disable().unwrap();
    let ret = hook_sub_703BF0.call(var_name, a2, a3);
    hook_sub_703BF0.enable().unwrap();

    let cfg = &CONFIG;
    if (&cfg.server_alert_url).is_none() {
        return ret;
    }

    let var = CStr::from_ptr(var_name).to_str().unwrap();

    // FIXME: this is a leak. kinda stupid but what can I do? (please tell me)
    if var != "SERVER_ALERT_URL" {
        return ret;
    }

    let new_url = cfg.server_alert_url.as_ref().unwrap();
    let new_url = CString::new(new_url.as_str()).unwrap();

    new_url.into_raw()
}

static mut ExtensionLoaded: bool = false;

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH && !ExtensionLoaded {
        graphics::init();
        hook_sub_46B840.enable().unwrap();
        hook_sub_703BF0.enable().unwrap();
        ExtensionLoaded = true;
    }

    return BOOL::from(true);
}
