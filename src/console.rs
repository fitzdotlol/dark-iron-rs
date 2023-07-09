#![allow(dead_code)]

use std::ffi::{c_char, CString};

#[repr(u32)]
pub enum CommandCategory {
    Debug = 0x0,
    Graphics = 0x1,
    Console = 0x2,
    Combat = 0x3,
    Game = 0x4,
    Default = 0x5,
    Net = 0x6,
    Sound = 0x7,
    GM = 0x8,
}

#[repr(u32)]
pub enum ConsoleColor {
    Default = 0x0,
    Input = 0x1,
    Echo = 0x2,
    Error = 0x3,
    Warning = 0x4,
    Global = 0x5,
    Admin = 0x6,
    Highlight = 0x7,
    Background = 0x8,
}

type def_ConsoleWriteRaw = extern "fastcall" fn(*const c_char, ConsoleColor);

pub fn console_write(text: &str, color: ConsoleColor) {
    let func = unsafe { std::mem::transmute::<u32, def_ConsoleWriteRaw>(0x0063CB50) };
    let str = CString::new(text).unwrap();

    func(str.as_ptr(), color);
}
