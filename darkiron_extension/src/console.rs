#![allow(dead_code)]

use crate::mem;
use darkiron_macro::hook_fn;
use simplelog::Config as LogConfig;
use simplelog::{Level, LevelFilter, SharedLogger};
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

pub type ConsoleCommandHandler =
    extern "fastcall" fn(cmd: *const c_char, args: *const c_char) -> u32;

pub struct ConsoleLogger {
    level: LevelFilter,
    config: LogConfig,
}

impl ConsoleLogger {
    pub fn new(log_level: LevelFilter, config: LogConfig) -> Box<Self> {
        Box::new(Self {
            level: log_level,
            config,
        })
    }
}

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let color = match record.level() {
                Level::Info => ConsoleColor::Admin,
                Level::Warn => ConsoleColor::Warning,
                Level::Error => ConsoleColor::Error,

                _ => ConsoleColor::Default,
            };

            // try_log(&self.config, record, ddd);

            let text = format!("{}", record.args());
            console_write(&text, color);
        }
    }

    fn flush(&self) {}
}

impl SharedLogger for ConsoleLogger {
    fn level(&self) -> LevelFilter {
        self.level
    }

    fn config(&self) -> Option<&LogConfig> {
        Some(&self.config)
    }

    fn as_log(self: Box<Self>) -> Box<dyn log::Log> {
        Box::new(*self)
    }
}

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

#[hook_fn(0x006395A0)]
extern "fastcall" fn ConsoleSetColor(color_type: ConsoleColor, _a2: u32, color: u32) {}

// TODO: think of a clever way to wrap the handler so we can use rust types there
// FIXME: WoW seems to expect static strings here. I'm sure there's a not-stupid solution,
//        but you only register a command once, so we're just gonna leak memory here for now...
pub fn register_command(
    cmd: &str,
    handler: ConsoleCommandHandler,
    category: CommandCategory,
    help: Option<&str>,
) -> u32 {
    let cmd_ptr = CString::new(cmd).unwrap().into_raw();

    let help_ptr = match help {
        Some(help) => CString::new(help).unwrap().into_raw(),
        None => std::ptr::null_mut(),
    };

    return ConsoleCommandRegister(cmd_ptr, handler, category, help_ptr);
}

pub fn console_write(text: &str, color: ConsoleColor) {
    let str = CString::new(text).unwrap();
    ConsoleWriteRaw(str.as_ptr(), color);
}

pub fn init() {
    // enable console
    unsafe { mem::set(0x00C4EC20, 1u32) };

    ConsoleSetColor(ConsoleColor::Admin, 0, 0xFF00CCFF);
}
