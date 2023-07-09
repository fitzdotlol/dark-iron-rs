#![feature(abi_thiscall)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

mod console;

use once_cell::sync::Lazy;
use retour::GenericDetour;

use std::os::raw::c_void;

use windows::Win32::{
    Foundation::{BOOL, HANDLE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

type def_sub_46B840 = unsafe extern "thiscall" fn(u32) -> u32;

static hook_sub_46B840: Lazy<GenericDetour<def_sub_46B840>> = Lazy::new(|| {
    return unsafe {
        GenericDetour::new(
            std::mem::transmute::<u32, def_sub_46B840>(0x0046B840),
            sub_46B840,
        )
        .unwrap()
    };
});

fn init_extension() {
    // enable console
    unsafe { *std::mem::transmute::<u32, *mut u32>(0xC4EC20) = 1 };

    console::console_write("wow112_extension loaded!", console::ConsoleColor::Warning);
}

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
