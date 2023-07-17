use std::ffi::{c_char, CStr, CString};

use darkiron_macro::{detour_fn, hook_fn};
use once_cell::sync::Lazy;

use crate::{console::console_write, mem};

// int __fastcall sub_4022C0(int a1)
// {
//   return sub_4022D0(*(&sp_archiveNames + a1));
// }

static ARCHIVE_NAMES: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "model.MPQ",
        "texture.MPQ",
        "terrain.MPQ",
        "wmo.MPQ",
        "sound.MPQ",
        "misc.MPQ",
        "interface.MPQ",
        "fonts.MPQ",
        "speech.MPQ",
        "test.MPQ",
    ]
});

// #[hook_fn(0x00403B00)]
// extern "fastcall" fn yy_maybeLoadMpq(archive_name: *const c_char, a2: u32, a3: u32) -> u32 {}

#[hook_fn(0x004022D0)]
extern "fastcall" fn sub_4022D0(a1: *const c_char) -> u32 {}

#[detour_fn(0x004022C0)]
extern "fastcall" fn sub_4022C0(archive_idx: u32) -> u32 {
    let name = ARCHIVE_NAMES[archive_idx as usize];
    let name_cstring = CString::new(name).unwrap();
    return sub_4022D0(name_cstring.as_ptr());
}

// #[detour_fn(0x004022D0)]
// extern "fastcall" fn yy_maybeLoadMpq(archive_name: *const c_char, a2: u32, a3: u32) -> u32
// {
//     // dword_8826BC = archive list

//     let mut buf: Vec<u8> = Vec::new();
//     buf.resize(260);

//     for i in 0..2 {
//         sub_402320(buf.as_mut_ptr(), buf.len(), i, archive_name);
//         let x = sub_648DD0(buf.as_ptr(), a2, 0, dword_8826BC + 4 * a3);

//         if x != 0 {
//             return 1;
//         }

//         // let err = sub_64DF50();

//         // s_lastErrCode
//         let err: u32 = unsafe { *mem::ptr(0x00C53D40) };

//         if err == 38 {
//             err = 0x85100083;
//         }

//         if (err == 0x8510006C || err == 0x85100083) {
//             SErrDisplayAppFatal(err, "Failed to open archive %s", buf);
//         }
//     }

//     return 0;
// }

// #[detour_fn(0x004022D0)]
// extern "thiscall" fn sub_4022D0(archive_name: *const c_char) -> u32 {
//     let mut buf: Vec<u8> = Vec::new();
//     buf.resize(260, 0);


//     for i in 0..2 {
//         sub_402320(buf.as_ptr(), buf.len(), i, archive_name);

//         if sub_42A4E0(buf.as_ptr()) {
//             break;
//         }
//     }

//     return 2;
// }


pub fn init() {
    let mut mpq_names: Vec<u32> = Vec::new();
    mpq_names.resize(10, 0);

    //sp_archiveNames =

    unsafe {
        let mpq_names_ptr = mem::ptr::<u32>(0x0082E12C);
        std::ptr::copy_nonoverlapping(mpq_names_ptr, mpq_names.as_mut_ptr(), 10);

        for i in 0..10 {
            // let text = format!("{i}, {}", mpq_names[i]);
            let name = mem::ptr::<c_char>(mpq_names[i]);
            let name_cstr = CStr::from_ptr(name);
            let text = format!("{i}: {:?}", name_cstr.to_str().unwrap());
            console_write(&text, crate::console::ConsoleColor::Warning);
        }
    }

    unsafe {
        hook_sub_4022C0.enable().unwrap();
    }
}
