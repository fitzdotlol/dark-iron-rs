use std::{
    ffi::CStr,
    io::Cursor,
    sync::{Arc, RwLock},
};

use windows::Win32::{
    Foundation::HWND,
    Graphics::{Gdi::HDC, OpenGL::*},
};

use crate::graphics::{create_orthographic_projection, gl, primitive::Primitive, texture::Texture};
use crate::{console::console_write, math::RectI};

type Context = Arc<RwLock<HGLRC>>;

#[derive(Debug, Default)]
pub struct UIState {
    pub enabled: bool,
    pub window: HWND,
    pub dc: HDC,
    pub ctx: Context,
    pub primitives: Vec<Primitive>,
    initialized: bool,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            enabled: true,
            window: HWND(0),
            dc: HDC(0),
            ctx: Arc::new(RwLock::new(HGLRC(0))),
            primitives: Vec::new(),
            initialized: false,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            let mut c = self.ctx.write().unwrap();

            let old_context = wglGetCurrentContext();

            if c.is_invalid() {
                match wglCreateContext(self.dc) {
                    Ok(ctx) => {
                        *c = ctx;
                        wglMakeCurrent(self.dc, ctx);

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

            glEnable(gl::TEXTURE_2D);
            glEnable(GL_BLEND);
            glEnable(GL_DEPTH);
            glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

            let mut prim2 = Primitive::rect(150.0);
            prim2.texture = Texture::load("logo.png");
            prim2.buffer();
            self.primitives.push(prim2);

            let mut prim = Primitive::rect(250.0);
            prim.texture = Texture::load("logo_white.png");
            prim.buffer();
            self.primitives.push(prim);

            gl::check_error();

            wglMakeCurrent(self.dc, old_context);
        }
    }

    pub async fn add_rect_from_url(&mut self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let res = reqwest::get(url).await?;
        let data = res.bytes().await?;

        let img = image::io::Reader::new(Cursor::new(data))
            .with_guessed_format()?
            .decode()?;

        unsafe {
            let c = self.ctx.read().unwrap();
            let old_context = wglGetCurrentContext();
            wglMakeCurrent(self.dc, *c);

            let mut prim = Primitive::rect(50.0);
            prim.texture = Texture::from_image(&img);
            prim.buffer();
            self.primitives.push(prim);

            wglMakeCurrent(self.dc, old_context);
        }

        Ok(())
    }

    pub fn draw(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }

        if !self.initialized {
            (*self).init();
            (*self).initialized = true;
        }

        unsafe {
            let c = self.ctx.read().unwrap();
            let old_context = wglGetCurrentContext();
            wglMakeCurrent(self.dc, *c);

            let projection = create_orthographic_projection(0.01, 10000.0);
            let proj_mtx = projection.m.flatten();

            glMatrixMode(GL_PROJECTION);
            glLoadMatrixf(proj_mtx.as_ptr());
            glMatrixMode(GL_MODELVIEW);
            glPushMatrix();
            glLoadIdentity();

            // window rect
            let r = &*std::mem::transmute::<u32, *const RectI>(0x00884E20);

            for prim in self.primitives.iter() {
                glPushMatrix();
                glTranslatef(r.x2 as f32 / 2.0, r.y2 as f32 / 2.0, 0.0);
                prim.draw();
                glPopMatrix();
            }

            wglMakeCurrent(self.dc, old_context);
        }

        Ok(())
    }
}
