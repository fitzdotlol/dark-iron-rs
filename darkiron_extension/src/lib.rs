#![feature(abi_thiscall, slice_flatten, fn_traits, c_variadic)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

use simplelog::*;

mod config;
mod console;
mod data;
mod graphics;
mod math;
mod script;

use std::{ffi::{c_char, c_void, CString}, panic, fs::File};

use config::CONFIG;

use console::{console_write, ConsoleColor};
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
#[hook_fn(0x0064CF80)]
unsafe extern "C" fn SErrDisplayAppFatal(err: u32, fmt: *const c_char, ...) {}

extern "fastcall" fn cmd_err(_cmd: *const c_char, _args: *const c_char) -> u32 {
    match panic::catch_unwind(|| {
        console_write("some text", ConsoleColor::Admin);
        panic!("oh no!");
    }) {
        Ok(_) => (),
        Err(e) => {
            fatal_error(format!("{e:?}").as_str(), 0x69420)

        }
    }

    return 0;
}

pub fn fatal_error(text: &str, code: u32) {
    let fmt = CString::new(text).unwrap();
    unsafe { SErrDisplayAppFatal(code, fmt.as_ptr()); }
}

fn init_extension() {
    unsafe {
        // Fix InvalidPtrCheck for callbacks outside of .text section
        mem::set(0x00884800, 0x00000001u32);
        mem::set(0x00884C00, 0x7FFFFFFFu32);
    }

    console::init();
    script::init();

    console_write("Dark Iron extension loaded!", console::ConsoleColor::Admin);
    console::console_command_register("err", cmd_err, console::CommandCategory::Debug, None);
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
extern "fastcall" fn sub_7A6CC0(
    url: *const c_char,
    callback: HTTPCallback,
    a3: u32,
    a4: u32,
) -> u32 {
}

#[detour_fn(0x007A6EE0)]
unsafe extern "fastcall" fn httpGetRequest(
    _old_url: *const c_char,
    callback: HTTPCallback,
    a3: u32,
) -> u32 {
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
        CombinedLogger::init(
            vec![
                TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
                WriteLogger::new(LevelFilter::Info, Config::default(), File::create("Logs/darkiron.log").unwrap()),
            ]
        ).unwrap();
    

        graphics::init();
        data::init();

        hook_sub_46B840.enable().unwrap();
        hook_httpGetRequest.enable().unwrap();
        ExtensionLoaded = true;
    }

    return BOOL::from(true);
}
