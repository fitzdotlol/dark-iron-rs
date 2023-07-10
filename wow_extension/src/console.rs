#![allow(dead_code)]

use std::ffi::{c_char, CString};

macro_rules! hook_fn {
    ($type:ty, $address:expr) => {
        unsafe { std::mem::transmute::<u32, $type>($address) }
    };
}

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

pub type ConsoleCommandHandler =
    extern "fastcall" fn(cmd: *const c_char, args: *const c_char) -> u32;

type def_ConsoleCommandRegister = extern "fastcall" fn(
    cmd: *const c_char,
    handler: ConsoleCommandHandler,
    category: CommandCategory,
    help: *const c_char,
) -> u32;

// TODO: think of a clever way to wrap the handler so we can use rust types there
// FIXME: WoW seems to expect static strings here. I'm sure there's a not-stupid solution,
//        but you only register a command once, so we're just gonna leak memory here for now...
pub fn console_command_register(
    cmd: &str,
    handler: ConsoleCommandHandler,
    category: CommandCategory,
    help: &str,
) -> u32 {
    let func = hook_fn!(def_ConsoleCommandRegister, 0x0063F9E0);
    let c_cmd = CString::new(cmd).unwrap();
    let c_help = CString::new(help).unwrap();

    return func(c_cmd.into_raw(), handler, category, c_help.into_raw());
}

type def_ConsoleWriteRaw = extern "fastcall" fn(*const c_char, ConsoleColor);

pub fn console_write(text: &str, color: ConsoleColor) {
    let func = hook_fn!(def_ConsoleWriteRaw, 0x0063CB50);
    let str = CString::new(text).unwrap();

    func(str.as_ptr(), color);
}
