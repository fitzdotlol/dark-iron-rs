use std::sync::Arc;

use image::DynamicImage;
use windows::Win32::Graphics::OpenGL::{GL_TEXTURE_MIN_FILTER, GL_TEXTURE_MAG_FILTER, GL_NEAREST, GL_LINEAR, GL_RGBA8, GL_UNSIGNED_BYTE, GL_RGBA};

use crate::console::console_write;

use super::gl;

#[derive(Debug, Default)]
pub struct Texture {
    pub gl_id: u32,
}

impl Texture {
    pub fn load(path: &str) -> Arc<Self> {
        let img = image::io::Reader::open(path).unwrap().decode().unwrap();
        Self::from_image(&img)
    }

    pub fn from_image(img: &DynamicImage) -> Arc<Self> {
        let pixels = img.clone().into_rgba8();

        let gl_id = gl::gen_texture();
        gl::bind_texture(gl::TEXTURE_2D, gl_id);

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

        gl::check_error();
        let text = format!("[gx] Texture.from_image -> {:?}", img);
        console_write(&text, crate::console::ConsoleColor::Warning);

        Arc::new(Self { gl_id })
    }

    pub fn bind(&self) {
        gl::bind_texture(gl::TEXTURE_2D, self.gl_id);
    }
}
