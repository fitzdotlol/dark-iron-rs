use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};

use darkiron_macro::{detour_fn, hook_fn};
use once_cell::sync::Lazy;

use crate::{config::CONFIG, fatal_error, mem};
use log::info;

pub static BASE_ARCHIVE_NAMES: Lazy<Vec<&str>> = Lazy::new(|| {
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
        "dbc.MPQ",
    ]
});

static BASE_ARCHIVE_INDICES: Lazy<HashMap<&str, u32>> = Lazy::new(|| {
    let mut j = 0;

    HashMap::from_iter(BASE_ARCHIVE_NAMES.iter().map(|name| {
        let pair = (*name, j);
        j += 1;
        pair
    }))
});

//signed int __stdcall sub_648FB0(_DWORD *a1, int (__fastcall *a2)(char *, int), int a3)
#[detour_fn(0x00648FB0)]
extern "system" fn sub_648FB0(mpq_ptr: u32, callback: u32, a3: u32) -> u32 {
    if mpq_ptr == 0 {
        return 0;
    }

    // super!(sub_648FB0)(mpq_ptr, callback, a3);

    let r = unsafe {
        hook_sub_648FB0.disable().unwrap();
        let ret = hook_sub_648FB0.call(mpq_ptr, callback, a3);
        hook_sub_648FB0.enable().unwrap();

        ret
    };

    r
}

// XXX: what even is this for? it *is* called, but changing the values
// didn't seem to cause any change. ditto for just returning 2.
#[detour_fn(0x004022C0)]
extern "fastcall" fn sub_4022C0(archive_idx: u32) -> u32 {
    // return 2;

    let name = BASE_ARCHIVE_NAMES[archive_idx as usize];
    let name_cstring = CString::new(name).unwrap();
    return sub_4022D0(name_cstring.as_ptr());
}

fn get_mpq_path(datapath_idx: u32, archive_name: &str) -> String {
    let data_cfg = &CONFIG.data;

    let path = if datapath_idx == 1 {
        "..\\Data\\"
    } else {
        data_cfg.path.as_str()
    };

    // TODO: path join? this is probably find tho idk
    return format!("{}{}", path, archive_name);
}

//int __stdcall sub_648DD0(char *RootPathName, int a2, int a3, int a4)
#[hook_fn(0x00648DD0)]
extern "system" fn sub_648DD0(a1: *const c_char, a2: u32, a3: u32, a4: u32) -> u32 {}

// sub_64DF50
fn get_error_code() -> u32 {
    unsafe { *mem::ptr(0x00C53D40) }
}

#[detour_fn(0x00403B00)]
extern "fastcall" fn yy_maybeLoadMpq(archive_name: *const c_char, a2: u32, a3: u32) -> u32 {
    let data_cfg = &CONFIG.data;

    let mpq = unsafe { CStr::from_ptr(archive_name) };
    let old_name = mpq.to_str().unwrap();
    let mpq_str = match old_name {
        str => {
            if !BASE_ARCHIVE_INDICES.contains_key(str) {
                return 2;
            }

            let idx = BASE_ARCHIVE_INDICES.get(str).unwrap();
            let a = data_cfg.base_archives.get(*idx as usize);
            if a.is_none() {
                return 2;
            }

            a.unwrap().as_str()
        }
    };

    let archive_list_ptr = unsafe { *mem::ptr::<u32>(0x008826BC) };

    for i in 0..2 {
        let path = get_mpq_path(i, mpq_str);
        let path_cstring = CString::new(path).unwrap();
        let x = sub_648DD0(path_cstring.as_ptr(), a2, 0, archive_list_ptr + 4 * a3);

        if x != 0 {
            info!("[mpq] loaded base archive {mpq_str}");
            return 1;
        }

        let mut err = get_error_code();

        if err == 38 {
            err = 0x85100083;
        }

        if err == 0x8510006C || err == 0x85100083 {
            let text = format!("Failed to open archive: {mpq_str}");
            fatal_error(&text, err);
        }
    }

    return 0;
}

#[hook_fn(0x0042A4E0)]
extern "thiscall" fn sub_42A4E0(a1: *const c_char) -> u32 {}

// FIXME: I screwed this up I think. why only return 2?
// I think I missed a return when refactoring
#[detour_fn(0x004022D0)]
extern "thiscall" fn sub_4022D0(archive_name: *const c_char) -> u32 {
    let mpq = unsafe { CStr::from_ptr(archive_name) };
    let mpq_str = mpq.to_str().unwrap();

    for i in 0..2 {
        let path = get_mpq_path(i, mpq_str);
        let path_cstring = CString::new(path).unwrap();

        if sub_42A4E0(path_cstring.as_ptr()) != 0 {
            break;
        }
    }

    return 2;
}

pub fn init() {
    // let mut mpq_names: Vec<u32> = Vec::new();
    // mpq_names.resize(10, 0);

    // unsafe {
    //     let mpq_names_ptr = mem::ptr::<u32>(0x0082E12C);
    //     std::ptr::copy_nonoverlapping(mpq_names_ptr, mpq_names.as_mut_ptr(), 10);

    //     for i in 0..10 {
    //         let name = mem::ptr::<c_char>(mpq_names[i]);
    //         let name_cstr = CStr::from_ptr(name);
    //         info!("{i}: {:?}", name_cstr.to_str().unwrap());
    //     }
    // }

    unsafe {
        hook_sub_4022C0.enable().unwrap();
        hook_yy_maybeLoadMpq.enable().unwrap();

        // fix null base mpq deref
        hook_sub_648FB0.enable().unwrap();
    }
}
