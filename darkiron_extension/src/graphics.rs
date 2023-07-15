#![allow(dead_code)]
use std::ffi::{c_char, c_void, CStr, CString};

use windows::core::PCSTR;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateBitmap, CreateCompatibleBitmap, GetDC, HDC};
use windows::Win32::Graphics::OpenGL::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreateWindowExA, SendMessageA, CW_USEDEFAULT, HMENU, ICONINFO, ICON_BIG,
    ICON_SMALL, WM_SETICON, WS_CAPTION, WS_EX_APPWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX,
    WS_OVERLAPPED, WS_SYSMENU, WS_THICKFRAME,
};

use darkiron_macro::detour_fn;

use crate::config::CONFIG;
use crate::console::console_write;
use crate::math::{Matrix4, RectI};
use crate::gl;


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

static TextureBindTargets: [u32; 3] = [GL_TEXTURE_2D, gl::TEXTURE_CUBE_MAP, gl::TEXTURE_RECTANGLE];

struct GxTextureFormat {
    internal_format: u32,
    format: u32,
    ty: u32,
    unk: u32,
}

static TextureFormats: [GxTextureFormat; 9] = [
    GxTextureFormat {
        internal_format: GL_NONE,
        format: GL_NONE,
        ty: GL_NONE,
        unk: 0,
    },
    GxTextureFormat {
        internal_format: GL_RGBA8,
        format: gl::BGRA,
        ty: gl::UNSIGNED_INT_8_8_8_8_REV,
        unk: 4,
    },
    GxTextureFormat {
        internal_format: GL_RGBA4,
        format: gl::BGRA,
        ty: gl::UNSIGNED_SHORT_4_4_4_4_REV,
        unk: 2,
    },
    GxTextureFormat {
        internal_format: GL_RGB5_A1,
        format: gl::BGRA,
        ty: gl::UNSIGNED_SHORT_1_5_5_5_REV,
        unk: 2,
    },
    GxTextureFormat {
        internal_format: GL_RGB5,
        format: GL_RGB,
        ty: gl::UNSIGNED_SHORT_5_6_5,
        unk: 2,
    },
    GxTextureFormat {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT1_EXT,
        format: GL_NONE,
        ty: GL_NONE,
        unk: 2,
    },
    GxTextureFormat {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT3_EXT,
        format: GL_NONE,
        ty: GL_NONE,
        unk: 0,
    },
    GxTextureFormat {
        internal_format: gl::COMPRESSED_RGBA_S3TC_DXT5_EXT,
        format: GL_NONE,
        ty: GL_NONE,
        unk: 0,
    },
    GxTextureFormat {
        internal_format: gl::DSDT8_NV,
        format: gl::DSDT_NV,
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

// TODO: maybe group into state struct of some sort
static mut UI_INITIALIZED: bool = false;
static mut UI_ENABLED: bool = true;
static mut UI_WINDOW: HWND = HWND(0);
static mut UI_DC: HDC = HDC(0);
static mut UI_CTX: HGLRC = HGLRC(0);
static mut UI_TEX: u32 = 0;
static mut UI_ROT: f32 = 0.0;

unsafe fn set_window_icon(hwnd: HWND) {
    let cfg = CONFIG.clone();

    if cfg.icon.is_none() {
        return;
    }

    let icon_path = cfg.icon.unwrap();

    let img = image::io::Reader::open(icon_path)
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

#[detour_fn(0x0058CF10)]
unsafe extern "fastcall" fn z_recreateOpenglWindow(
    this: *const c_void,
    win: *const CGxOpenGlWindow,
) -> HWND {
    let hinstance = GetModuleHandleA(PCSTR(std::ptr::null())).unwrap();

    let class_name = "GxWindowClassOpenGl\0";

    let hwnd = CreateWindowExA(
        WS_EX_APPWINDOW,
        PCSTR(class_name.as_ptr()),
        PCSTR(std::ptr::null()),
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

    return hwnd;
}

fn gl_check_error() {
    let err_code = unsafe { glGetError() };

    if err_code == 0 {
        return;
    }

    let err_str = unsafe {
        let err_str_ptr = gluErrorString(err_code) as *const c_char;

        CStr::from_ptr(err_str_ptr)
    };

    let text = format!("[ui] glTexImage2D: {}", err_str.to_str().unwrap());
    console_write(&text, crate::console::ConsoleColor::Error);
}

#[repr(C)]
struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
}

unsafe fn init_ui(dev_ptr: u32) {
    let text = format!("[ui] intializing with device ptr: 0x{dev_ptr:X}");
    console_write(&text, crate::console::ConsoleColor::Admin);

    let old_context = wglGetCurrentContext();

    if UI_CTX.is_invalid() {
        match wglCreateContext(UI_DC) {
            Ok(ctx) => {
                UI_CTX = ctx;
                wglMakeCurrent(UI_DC, ctx);

                let c_str = CStr::from_ptr(glGetString(GL_VERSION) as *const i8);
                let text = format!("[ui] created gl context: {}", c_str.to_str().unwrap());
                console_write(&text, crate::console::ConsoleColor::Admin);
            }
            Err(e) => {
                let text = format!("[ui] failed to create gl context: {e:?}");
                console_write(&text, crate::console::ConsoleColor::Error);
            }
        }
    }

    glEnable(GL_TEXTURE_2D);
    glEnable(GL_BLEND);
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

    let dim = 250.0;

    let verts = vec![
        Vertex {
            pos: [-dim, -dim, 0.0],
            uv: [0.0, 0.0],
        },
        Vertex {
            pos: [dim, -dim, 0.0],
            uv: [1.0, 0.0],
        },
        Vertex {
            pos: [dim, dim, 0.0],
            uv: [1.0, 1.0],
        },
        Vertex {
            pos: [-dim, dim, 0.0],
            uv: [0.0, 1.0],
        },
    ];

    let vbo = gl::gen_buffer();
    gl::bind_buffer(gl::ARRAY_BUFFER, vbo);
    gl::buffer_data::<Vertex>(gl::ARRAY_BUFFER, &verts, gl::STATIC_DRAW);

    UI_TEX = load_texture("logo.png");

    wglMakeCurrent(UI_DC, old_context);
}

fn load_texture(path: &str) -> u32 {
    let img = image::io::Reader::open(path).unwrap().decode().unwrap();
    let pixels = img.clone().into_rgba8();

    let tex_id = gl::gen_texture();
    gl::bind_texture(gl::TEXTURE_2D, tex_id);

    let text = format!("[ui] generated texture: {}", tex_id);
    console_write(&text, crate::console::ConsoleColor::Admin);

    gl::tex_parameter_i(gl::TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    gl::tex_parameter_i(gl::TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);

    gl::tex_image_2d(
        gl::TEXTURE_2D,
        0,
        GL_RGBA8,
        img.width(),
        img.height(),
        0,
        GL_RGBA,
        GL_UNSIGNED_BYTE,
        pixels.as_ptr(),
    );

    gl_check_error();

    tex_id
}

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

    glBindTexture(GL_TEXTURE_2D, UI_TEX);

    UI_ROT += 1.0;

    while UI_ROT > 360.0 {
        UI_ROT -= 360.0;
    }

    glTranslatef(400.0, 300.0, 0.0);
    glRotatef(UI_ROT, 0.0, 0.0, 1.0);

    glEnableClientState(GL_VERTEX_ARRAY);
    glVertexPointer(
        3,
        GL_FLOAT,
        std::mem::size_of::<Vertex>() as i32,
        0 as *const c_void,
    );
    glEnableClientState(GL_TEXTURE_COORD_ARRAY);
    glTexCoordPointer(
        2,
        GL_FLOAT,
        std::mem::size_of::<Vertex>() as i32,
        12 as *const c_void,
    );
    gl::draw_arrays(GL_QUADS, 0, 4);

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
extern "fastcall" fn sub_435A50(a1: u32, _windowTitle: *const c_char) -> u32 {
    let cfg = &CONFIG;

    let win_name = match &cfg.title {
        Some(s) => s.as_str(),
        None => "World of Warcraft",
    };

    let win_name = CString::new(win_name).unwrap();

    unsafe {
        hook_sub_435A50.disable().unwrap();
        let ret = hook_sub_435A50.call(a1, win_name.as_ptr());
        hook_sub_435A50.enable().unwrap();

        return ret;
    }
}

pub fn toggle() {
    unsafe { UI_ENABLED = !UI_ENABLED };
}

pub fn init() {
    unsafe {
        // disable direct3d lol
        // TODO: find a less-idiotic way of doing this
        let src = "OpenGL\0";
        let dst_1 = crate::mem::ptr::<u8>(0x0080E138);
        let dst_2 = crate::mem::ptr::<u8>(0x00864F7C);
        std::ptr::copy(src.as_ptr() as *mut u8, dst_1, src.len());
        std::ptr::copy(src.as_ptr() as *mut u8, dst_2, src.len());

        hook_sub_59BA10.enable().unwrap();
        hook_z_recreateOpenglWindow.enable().unwrap();
        hook_sub_435A50.enable().unwrap();
    }
}
