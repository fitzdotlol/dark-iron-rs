#![feature(abi_thiscall, slice_flatten, fn_traits, c_variadic)]
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]

mod config;
mod console;
mod data;
mod graphics;
mod math;
mod script;
mod ui;

use std::{ffi::{c_char, c_void, CString}, panic};

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
            let fmt = CString::new("Test test").unwrap();
            unsafe { SErrDisplayAppFatal(0x69420, fmt.as_ptr()); }
        }
    }

    return 0;
}

extern "fastcall" fn cmd_ui(_cmd: *const c_char, _args: *const c_char) -> u32 {
    console_write("toggling ui", ConsoleColor::Warning);
    graphics::toggle_ui();
    return 0;
}

// use tokio;
// #[tokio::main]
// async fn init_discord() -> Result<(), Box<dyn std::error::Error>> {
//     let (wheel, handler) = Wheel::new(Box::new(|err| {
//         let text = format!("[discord] Error: {:?}", err);
//         console::console_write(&text, ConsoleColor::Error);
//     }));

//     let mut user = wheel.user();

//     let discord = match discord_sdk::Discord::new(
//         DiscordApp::PlainId(1130041465406509096),
//         Subscriptions::ACTIVITY,
//         Box::new(handler)
//     ) {
//         Ok(dc) => dc,
//         Err(e) => {
//             let text = format!("[discord] Discord::new -> {:?}", e);
//             console_write(&text, ConsoleColor::Error);
//             panic!();
//         }
//     };

//     user.0.changed().await.unwrap();

//     let user = match &*user.0.borrow() {
//         discord_sdk::wheel::UserState::Connected(user) => user.clone(),
//         discord_sdk::wheel::UserState::Disconnected(err) => panic!("failed to connect to Discord: {}", err),
//     };

//     let text = format!("[discord] connected as user: {:?}", user);
//     console_write(&text, ConsoleColor::Admin);

//     if user.avatar.is_some() {
//         let pfp = user.avatar.unwrap();
//         let id = hex::encode(pfp.0);
//         let url = format!("https://cdn.discordapp.com/avatars/{}/{}.png", user.id.0, id);

//         let text = format!("  * pfp: {}", url);
//         console_write(&text, ConsoleColor::Admin);

//         graphics::add_rect_from_url(&url).await?;
//     }

//     let rp = discord_sdk::activity::ActivityBuilder::default()
//     .details("Fruit Tarts".to_owned())
//     .state("Pop Snacks".to_owned())
//     // .assets(
//     //     discord_sdk::activity::Assets::default()
//     //         .large("the".to_owned(), Some("u mage".to_owned()))
//     //         .small("the".to_owned(), Some("i mage".to_owned())),
//     // )
//     // .button(discord_sdk::activity::Button {
//     //     label: "discord-sdk by EmbarkStudios".to_owned(),
//     //     url: "https://github.com/EmbarkStudios/discord-sdk".to_owned(),
//     // })
//     .start_timestamp(SystemTime::now());

//     discord.update_activity(rp).await?;

//     Ok(())
// }

// extern "fastcall" fn cmd_discord(_cmd: *const c_char, _args: *const c_char) -> u32 {
//     init_discord();
//     return 0;
// }

fn init_extension() {
    unsafe {
        // Fix InvalidPtrCheck for callbacks outside of .text section
        mem::set(0x00884800, 0x00000001u32);
        mem::set(0x00884C00, 0x7FFFFFFFu32);
    }

    console::init();
    data::init();
    script::init();

    console_write("Dark Iron extension loaded!", console::ConsoleColor::Admin);
    console::console_command_register("ui", cmd_ui, console::CommandCategory::Graphics, None);
    console::console_command_register("err", cmd_err, console::CommandCategory::Debug, None);
    // console::console_command_register("discord", cmd_discord, console::CommandCategory::Net, None);
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
        graphics::init();
        hook_sub_46B840.enable().unwrap();
        hook_httpGetRequest.enable().unwrap();
        ExtensionLoaded = true;
    }

    return BOOL::from(true);
}
