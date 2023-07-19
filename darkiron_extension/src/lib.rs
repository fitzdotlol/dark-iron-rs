#![feature(abi_thiscall, slice_flatten, fn_traits, c_variadic)]
#![allow(
    non_upper_case_globals,
    non_snake_case,
    non_camel_case_types,
    dead_code
)]

// use simplelog::*;
use log::{error, info, warn, LevelFilter};
use simplelog::{CombinedLogger, Config as LogConfig, WriteLogger};

mod config;
mod console;
mod data;
mod graphics;
mod math;
mod script;

use std::{
    ffi::{c_char, c_void, CString},
    fs::File,
    panic, thread,
};

use config::CONFIG;

use console::{console_write, ConsoleColor};
use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

use darkiron_macro::{detour_fn, enable_detour, hook_fn};

use crate::console::CommandCategory;

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

extern "fastcall" fn cmd_test(_cmd: *const c_char, _args: *const c_char) -> u32 {
    info!("info test");
    warn!("warn");
    error!("error");
    return 0;
}

extern "fastcall" fn cmd_err(_cmd: *const c_char, _args: *const c_char) -> u32 {
    match panic::catch_unwind(|| {
        console_write("some text", ConsoleColor::Admin);
        panic!("oh no!");
    }) {
        Ok(_) => (),
        Err(e) => fatal_error(format!("{e:?}").as_str(), 0x69420),
    }

    return 0;
}

pub fn fatal_error(text: &str, code: u32) {
    let fmt = CString::new(text).unwrap();
    unsafe {
        SErrDisplayAppFatal(code, fmt.as_ptr());
    }
    panic!();
}

fn init_extension() {
    unsafe {
        // Fix InvalidPtrCheck for callbacks outside of .text section
        mem::set(0x00884800, 0x00000001u32);
        mem::set(0x00884C00, 0x7FFFFFFFu32);
    }

    console::init();
    script::init();

    info!("Dark Iron extension loaded!");

    console::register_command("test", cmd_test, CommandCategory::Debug, None);
    console::register_command("err", cmd_err, CommandCategory::Debug, None);
}

#[detour_fn(0x0046B840)]
extern "thiscall" fn sub_46B840(a1: u32) {
    init_extension();

    unsafe {
        hook_sub_46B840.disable().unwrap();
        hook_sub_46B840.call(a1);
    }
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
    let win_cfg = &CONFIG.window;

    if win_cfg.server_alert_url.is_none() {
        return 0;
    }

    let url = win_cfg.server_alert_url.as_ref().unwrap();
    let url_cstring = CString::new(url.as_str()).unwrap();

    return sub_7A6CC0(url_cstring.as_ptr(), callback, a3, 5000);
}

#[derive(Debug)]
#[repr(u32)]
enum UISignatureResponse {
    Missing = 0,
    Corrupt = 1,
    Modified = 2,
    Ok = 3,
}

#[detour_fn(0x006F10F0)]
extern "thiscall" fn sub_6F10F0(filename: *const c_char, a2: u32, a3: u32) -> UISignatureResponse {
    let res = original!(filename, a2, a3);

    if CONFIG.data.validate_interface {
        return res;
    }

    UISignatureResponse::Ok
}

static mut ExtensionLoaded: bool = false;

fn early_init() {
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Info,
            LogConfig::default(),
            File::create("Logs/darkiron.log").unwrap(),
        ),
        console::ConsoleLogger::new(LevelFilter::Info, LogConfig::default()),
    ])
    .unwrap();

    graphics::init();
    data::init();

    enable_detour!(sub_46B840);
    enable_detour!(httpGetRequest);
    enable_detour!(sub_6F10F0);
}

#[no_mangle]
unsafe extern "system" fn DllMain(_hinst: HANDLE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH && !ExtensionLoaded {
        thread::spawn(early_init);
        ExtensionLoaded = true;
    }

    return BOOL::from(true);
}
