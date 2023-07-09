#![feature(abi_thiscall)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

mod console;

use once_cell::sync::Lazy;
use retour::GenericDetour;

use std::{ffi::c_char, os::raw::c_void};

use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

macro_rules! ptr {
    ($address:expr, $type:ty) => {
        *($address as *mut $type)
    };
}

extern "fastcall" fn cmd_test(_cmd: *const c_char, _args: *const c_char) -> u32 {
    console::console_write("this is only a test", console::ConsoleColor::Error);
    return 0;
}

fn init_extension() {
    unsafe {
        // enable console
        ptr!(0x00C4EC20, u32) = 1;

        // Fix InvalidPtrCheck for callbacks outside of .text section
        ptr!(0x00884800, u32) = 0x00000001;
        ptr!(0x00884C00, u32) = 0x7FFFFFFF;
    }

    console::console_write("wow112_extension loaded!", console::ConsoleColor::Warning);
    console::console_command_register("test", cmd_test, console::CommandCategory::Debug, "uhhh");
}

type def_sub_46B840 = unsafe extern "thiscall" fn(u32) -> u32;

static hook_sub_46B840: Lazy<GenericDetour<def_sub_46B840>> = Lazy::new(|| unsafe {
    GenericDetour::new(
        std::mem::transmute::<u32, def_sub_46B840>(0x0046B840),
        sub_46B840,
    )
    .unwrap()
});

unsafe extern "thiscall" fn sub_46B840(a1: u32) -> u32 {
    init_extension();

    hook_sub_46B840.disable().unwrap();
    return hook_sub_46B840.call(a1);
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
