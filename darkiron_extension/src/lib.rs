#![feature(abi_thiscall, slice_flatten)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

mod config;
mod console;
mod gl;
mod graphics;
mod math;
mod script;

use std::ffi::{c_char, c_void, CString};

use config::CONFIG;

use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

use darkiron_macro::{detour_fn, hook_fn};

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

// char __fastcall sub_46A580(int a1, char *a2, unsigned int a3, char a4, int a5)
type HTTPCallback = extern "fastcall" fn(u32, *const c_char, length: u32, u32, u32) -> u32;

#[hook_fn(0x007A6CC0)]
extern "fastcall" fn sub_7A6CC0(url: *const c_char, callback: HTTPCallback, a3: u32, a4: u32) -> u32 {}

#[detour_fn(0x007A6EE0)]
unsafe extern "fastcall" fn httpGetRequest(_old_url: *const c_char, callback: HTTPCallback, a3: u32) -> u32 {
    if CONFIG.server_alert_url.is_none() {
        return 0;
    }

    let url = CONFIG.server_alert_url.as_ref().unwrap();
    let url_cstring = CString::new(url.as_str()).unwrap();

    return sub_7A6CC0(url_cstring.as_ptr(), callback, a3, 5000);
}

static mut ExtensionLoaded: bool = false;

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH && !ExtensionLoaded {
        graphics::init();
        hook_sub_46B840.enable().unwrap();
        hook_httpGetRequest.enable().unwrap();
        ExtensionLoaded = true;
    }

    return BOOL::from(true);
}
