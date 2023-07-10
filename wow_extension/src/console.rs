#![allow(dead_code)]

use std::ffi::{c_char, CString};
use wow_mem::hook_fn;

macro_rules! ptr {
    ($address:expr, $type:ty) => {
        *($address as *mut $type)
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

#[hook_fn(0x0063F9E0)]
extern "fastcall" fn ConsoleCommandRegister(
    cmd: *const c_char,
    handler: ConsoleCommandHandler,
    category: CommandCategory,
    help: *const c_char,
) -> u32 {
}

#[hook_fn(0x0063CB50)]
extern "fastcall" fn ConsoleWriteRaw(text: *const c_char, color: ConsoleColor) {}

// TODO: think of a clever way to wrap the handler so we can use rust types there
// FIXME: WoW seems to expect static strings here. I'm sure there's a not-stupid solution,
//        but you only register a command once, so we're just gonna leak memory here for now...
pub fn console_command_register(
    cmd: &str,
    handler: ConsoleCommandHandler,
    category: CommandCategory,
    help: &str,
) -> u32 {
    let c_cmd = CString::new(cmd).unwrap();
    let c_help = CString::new(help).unwrap();

    return ConsoleCommandRegister(c_cmd.into_raw(), handler, category, c_help.into_raw());
}

pub fn console_write(text: &str, color: ConsoleColor) {
    let str = CString::new(text).unwrap();
    ConsoleWriteRaw(str.as_ptr(), color);
}

pub fn init() {
    // enable console
    unsafe { ptr!(0x00C4EC20, u32) = 1 };
}
