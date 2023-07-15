#![allow(dead_code)]
use std::ffi::{c_char, c_void, CStr};

use windows::core::PCSTR;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateBitmap, CreateCompatibleBitmap, GetDC, HDC};
use windows::Win32::Graphics::OpenGL::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreateWindowExA, SendMessageA, CW_USEDEFAULT, HMENU, ICONINFO, ICON_BIG,
    ICON_SMALL, WM_SETICON, WS_CAPTION, WS_EX_LEFT, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_OVERLAPPED,
    WS_SYSMENU, WS_THICKFRAME,
};
use darkiron_macro::detour_fn;

use crate::console::console_write;
use crate::math::{Matrix4, RectI};

// #[detour_fn(0x00482D70)]
// unsafe extern "thiscall" fn CGWorldFrame__RenderWorld(this: *const c_void) {
//     hook_CGWorldFrame__RenderWorld.disable().unwrap();
//     hook_CGWorldFrame__RenderWorld.call(this);
//     hook_CGWorldFrame__RenderWorld.enable().unwrap();
// }

// // menu scene render callback
// #[detour_fn(0x76D240)]
// unsafe extern "thiscall" fn sub_76D240(this: *const c_void) {
//     hook_sub_76D240.disable().unwrap();
//     hook_sub_76D240.call(this);
//     hook_sub_76D240.enable().unwrap();
// }

// // breaking news render callback??? that makes no sense, but that's what it looks like.
// #[detour_fn(0x787220)]
// unsafe extern "thiscall" fn sub_787220(this: *const c_void) {
//     hook_sub_787220.disable().unwrap();
//     hook_sub_787220.call(this);
//     hook_sub_787220.enable().unwrap();
// }

static mut UI_INITIALIZED: bool = false;
static mut UI_ENABLED: bool = true;
static mut UI_TEX: u32 = 0;

fn create_orthographic_projection(near: f32, far: f32) -> Matrix4 {
    let mut projection = Matrix4 { m: [[0.0; 4]; 4] };

    let r = unsafe { &*std::mem::transmute::<u32, *const RectI>(0x00884E20) };

    let left = r.x1 as f32;
    let right = r.x2 as f32;
    let top = r.y1 as f32;
    let bottom = r.y2 as f32;

    projection.m[0][0] = 2.0 / (right - left);
    projection.m[1][1] = 2.0 / (top - bottom);
    projection.m[2][2] = 1.0 / (far - near);
    projection.m[3][0] = -(right + left) / (right - left);
    projection.m[3][1] = -(top + bottom) / (top - bottom);
    projection.m[3][2] = -near / (far - near);
    projection.m[3][3] = 1.0;

    projection
}

static TextureBindTargets: [u32; 3] = [GL_TEXTURE_2D, GL_TEXTURE_CUBE_MAP, GL_TEXTURE_RECTANGLE];

struct GxTextureFormat {
    internal_format: u32,
    format: u32,
    ty: u32,
    unk: u32,
}

const GL_UNSIGNED_INT_8_8_8_8_REV: u32 = 0x8367;
const GL_UNSIGNED_SHORT_4_4_4_4_REV: u32 = 0x8365;
const GL_UNSIGNED_SHORT_1_5_5_5_REV: u32 = 0x8366;
const GL_DSDT_NV: u32 = 0x86F5;
const GL_BGRA: u32 = 0x80E1;
const GL_UNSIGNED_SHORT_5_6_5: u32 = 0x8363;
const GL_COMPRESSED_RGBA_S3TC_DXT1_EXT: u32 = 0x83F1;
const GL_COMPRESSED_RGBA_S3TC_DXT3_EXT: u32 = 0x83F2;
const GL_COMPRESSED_RGBA_S3TC_DXT5_EXT: u32 = 0x83F3;
const GL_DSDT8_NV: u32 = 0x8709;
const GL_TEXTURE_CUBE_MAP: u32 = 0x8513;
const GL_TEXTURE_RECTANGLE: u32 = 0x84F5;

static TextureFormats: [GxTextureFormat; 9] = [
    GxTextureFormat {
        internal_format: GL_NONE,
        format: GL_NONE,
        ty: GL_NONE,
        unk: 0,
    },
    GxTextureFormat {
        internal_format: GL_RGBA8,
        format: GL_BGRA,
        ty: GL_UNSIGNED_INT_8_8_8_8_REV,
        unk: 4,
    },
    GxTextureFormat {
        internal_format: GL_RGBA4,
        format: GL_BGRA,
        ty: GL_UNSIGNED_SHORT_4_4_4_4_REV,
        unk: 2,
    },
    GxTextureFormat {
        internal_format: GL_RGB5_A1,
        format: GL_BGRA,
        ty: GL_UNSIGNED_SHORT_1_5_5_5_REV,
        unk: 2,
    },
    GxTextureFormat {
        internal_format: GL_RGB5,
        format: GL_RGB,
        ty: GL_UNSIGNED_SHORT_5_6_5,
        unk: 2,
    },
    GxTextureFormat {
        internal_format: GL_COMPRESSED_RGBA_S3TC_DXT1_EXT,
        format: GL_NONE,
        ty: GL_NONE,
        unk: 2,
    },
    GxTextureFormat {
        internal_format: GL_COMPRESSED_RGBA_S3TC_DXT3_EXT,
        format: GL_NONE,
        ty: GL_NONE,
        unk: 0,
    },
    GxTextureFormat {
        internal_format: GL_COMPRESSED_RGBA_S3TC_DXT5_EXT,
        format: GL_NONE,
        ty: GL_NONE,
        unk: 0,
    },
    GxTextureFormat {
        internal_format: GL_DSDT8_NV,
        format: GL_DSDT_NV,
        ty: GL_BYTE,
        unk: 2,
    },
];

struct CGxOpenGlTexture {
    unk_00: u32,
    unk_01: u32,
    unk_02: u32,
    unk_03: u32,
    unk_04: u32,
    unk_05: u32,
    unk_06: u32,
    unk_07: u32,
    unk_08: u32,
    unk_09: u32,
    unk_10: u32,
    unk_11: u32,
    bind_target: u32,     // 12, index into TextureBindTargets
    internal_format: u32, // 13
    format: u32,          // 14, index into TextureFormats
    unk_15: u32,
    unk_16: u32,
    unk_17: u32,
    gl_id: u32, // 18
}

#[derive(Debug)]
#[repr(C)]
struct CGxOpenGlWindow {
    unk_00: u32,
    unk_01: u32,
    unk_02: u32,
    unk_03: u32,
    width: i32,  // 4
    height: i32, // 5
    unk_06: u32,
    unk_07: u32,
    unk_08: u32,
    unk_09: u32,
    unk_10: u32,
    unk_11: u32,
    x: i32, // 12
    y: i32, // 13
}

struct CGxDeviceOpenGl {
    // HWND: + 0x39E4
    // HDC: + 0x39F0
}

// FIXME: just use static device pointer for these? 0x00C0ED38
static mut UI_WINDOW: HWND = HWND { 0: 0 };
static mut UI_DC: HDC = HDC { 0: 0 };
static mut UI_CTX: HGLRC = HGLRC { 0: 0 };

unsafe fn set_window_icon(hwnd: HWND) {
    let img = image::io::Reader::open("icon.png")
        .unwrap()
        .decode()
        .unwrap();
    let pixels = img.clone().into_rgba8().as_ptr() as *const c_void;

    let hbmColor = CreateBitmap(img.width() as i32, img.height() as i32, 1, 32, Some(pixels));
    let hbmMask = CreateCompatibleBitmap(UI_DC, img.width() as i32, img.height() as i32);

    let icon_info = ICONINFO {
        fIcon: BOOL::from(true),
        xHotspot: 0,
        yHotspot: 0,
        hbmMask,
        hbmColor,
    };

    let icon = CreateIconIndirect(&icon_info).unwrap();
    let lp = LPARAM(icon.0);

    SendMessageA(hwnd, WM_SETICON, WPARAM(ICON_BIG as usize), lp);
    SendMessageA(hwnd, WM_SETICON, WPARAM(ICON_SMALL as usize), lp);
}

// HWND __fastcall z_recreateOpenglWindow(void *a1, _DWORD *a2)
#[detour_fn(0x0058CF10)]
unsafe extern "fastcall" fn z_recreateOpenglWindow(
    this_addr: u32,
    win: *const CGxOpenGlWindow,
) -> HWND {
    let hinstance = GetModuleHandleA(PCSTR(std::ptr::null())).unwrap();

    let class_name = "GxWindowClassOpenGl\0";
    let win_name = "Dark Iron WoW\0";

    let this: *const c_void = std::mem::transmute(this_addr);

    let hwnd = CreateWindowExA(
        WS_EX_LEFT,
        // WS_EX_APPWINDOW,
        PCSTR(class_name.as_ptr()),
        PCSTR(win_name.as_ptr()),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        (*win).width,
        (*win).height,
        HWND(0),
        HMENU(0),
        hinstance,
        Some(this),
    );

    UI_WINDOW = hwnd;
    UI_DC = GetDC(hwnd);

    set_window_icon(hwnd);

    // const app_user_model_id: &'static str = "DarkIronWoW";
    // let str = HSTRING::from(app_user_model_id);
    // _ = SetCurrentProcessExplicitAppUserModelID(&str);
    // let mut store: *mut IPropertyStore = std::ptr::null_mut();
    // let store_ptr = &mut store as *mut *mut IPropertyStore;
    // let store_ptr2 = store_ptr as *mut *mut c_void;
    // let guid = windows::core::GUID::new().unwrap();

    // // IID_PPV_ARGS()
    // _ = SHGetPropertyStoreForWindow(hwnd, &guid, store_ptr2);

    return hwnd;
}

unsafe fn gl_check_error() {
    let err_code = glGetError();

    if err_code == 0 {
        return;
    }

    let err_str_ptr = gluErrorString(err_code);
    let err_str = CStr::from_ptr(err_str_ptr as *const i8);
    let text = format!("[ui] glTexImage2D: {}", err_str.to_str().unwrap());
    console_write(&text, crate::console::ConsoleColor::Error);
}

unsafe fn init_ui(dev_ptr: u32) {
    // let mut console = Console {};
    // _ = write!(console, "[ui] intializing with device ptr: 0x{dev_ptr:X}");

    let text = format!("[ui] intializing with device ptr: 0x{dev_ptr:X}");
    console_write(&text, crate::console::ConsoleColor::Warning);

    let old_context: HGLRC = wglGetCurrentContext();

    if UI_CTX.0 == 0 {
        match wglCreateContext(UI_DC) {
            Ok(ctx) => UI_CTX = ctx,
            Err(e) => {
                let text = format!("[ui] failed to create gl context: {e:?}");
                console_write(&text, crate::console::ConsoleColor::Error);
            }
        }
    }

    wglMakeCurrent(UI_DC, UI_CTX);

    glEnable(GL_TEXTURE_2D);
    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

    // glEnable(GL_DEPTH_TEST);

    let img = image::io::Reader::open("logo.png")
        .unwrap()
        .decode()
        .unwrap();
    let pixels = img.clone().into_rgba8();

    //// TODO: get procs so we can use newer gl shit like this
    // glGenBuffers(1, &vbo)
    // glBindBuffer(GL_ARRAY_BUFFER, vbo)
    // glBufferData(GL_ARRAY_BUFFER)

    glGenTextures(1, &mut UI_TEX);
    glBindTexture(GL_TEXTURE_2D, UI_TEX);

    let text = format!("[ui] generated texture: {}", UI_TEX);
    console_write(&text, crate::console::ConsoleColor::Warning);

    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST as i32);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR as i32);

    glTexImage2D(
        GL_TEXTURE_2D,
        0,
        GL_RGBA8 as i32,
        img.width() as i32,
        img.height() as i32,
        0,
        GL_RGBA,
        GL_UNSIGNED_BYTE,
        pixels.as_ptr() as *const c_void,
    );

    gl_check_error();

    wglMakeCurrent(UI_DC, old_context);
}

static mut UI_ROT: f32 = 0.0;

unsafe fn draw_ui() {
    let old_context = wglGetCurrentContext();
    wglMakeCurrent(UI_DC, UI_CTX);

    let projection = create_orthographic_projection(0.01, 10000.0);
    let proj_mtx = projection.m.flatten();

    glMatrixMode(GL_PROJECTION);
    glPushMatrix();
    glLoadMatrixf(proj_mtx.as_ptr());
    glMatrixMode(GL_MODELVIEW);
    glPushMatrix();
    glLoadIdentity();

    let dim = 250.0;
    let left = -dim;
    let right = dim;
    let top = -dim;
    let bottom = dim;

    glBindTexture(GL_TEXTURE_2D, UI_TEX);
    glColor3ub(0xFF, 0xFF, 0xFF);

    UI_ROT += 1.0;

    while UI_ROT > 360.0 {
        UI_ROT -= 360.0;
    }

    glTranslatef(150.0, 50.0, 0.0);
    glTranslatef(dim, dim, 0.0);
    glRotatef(UI_ROT, 0.0, 0.0, 1.0);

    glBegin(GL_QUADS);
    {
        glVertex3f(left, top, 0.0);
        glTexCoord2f(0.0, 0.0);
        glVertex3f(right, top, 0.0);
        glTexCoord2f(1.0, 0.0);
        glVertex3f(right, bottom, 0.0);
        glTexCoord2f(1.0, 1.0);
        glVertex3f(left, bottom, 0.0);
        glTexCoord2f(0.0, 1.0);
    }
    glEnd();

    glPopMatrix();
    glMatrixMode(GL_PROJECTION);
    glPopMatrix();

    wglMakeCurrent(UI_DC, old_context);
}

#[detour_fn(0x0059BA10)]
unsafe extern "thiscall" fn sub_59BA10(dev_ptr: u32, a2: u32) -> u32 {
    if !UI_INITIALIZED {
        init_ui(dev_ptr);
        UI_INITIALIZED = true;
    }

    if UI_ENABLED {
        draw_ui();
    }

    hook_sub_59BA10.disable().unwrap();
    let ret_val = hook_sub_59BA10.call(dev_ptr, a2);
    hook_sub_59BA10.enable().unwrap();

    ret_val
}

//int __fastcall sub_435A50(int a1, char *windowTitle)
#[detour_fn(0x00435A50)]
unsafe extern "fastcall" fn sub_435A50(a1: u32, _windowTitle: *const c_char) -> u32 {
    let title: &'static str = "Dark Iron WoW\0";
    hook_sub_435A50.disable().unwrap();
    let ret = hook_sub_435A50.call(a1, title.as_ptr() as *const i8);
    hook_sub_435A50.enable().unwrap();
    return ret;
}

pub fn toggle() {
    unsafe { UI_ENABLED = !UI_ENABLED };
}

pub fn init() {
    unsafe {
        hook_sub_59BA10.enable().unwrap();
        hook_z_recreateOpenglWindow.enable().unwrap();

        hook_sub_435A50.enable().unwrap();
    }
}
