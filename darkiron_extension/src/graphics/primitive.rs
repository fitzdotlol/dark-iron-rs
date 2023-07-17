use std::sync::Arc;

use super::{gl, texture::Texture};

#[derive(Debug)]
#[repr(C)]
pub struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
    color: [u8; 4],
}

#[derive(Default, Debug)]
pub struct Primitive {
    pub verts: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vbo: u32,
    pub ibo: u32,
    pub texture: Arc<Texture>,
    pub rotation: f32,
}

impl Primitive {
    pub fn rect(dim: f32) -> Self {
        let mut prim = Primitive::default();

        prim.verts = vec![
            Vertex {
                pos: [-dim, -dim, 0.0],
                uv: [0.0, 0.0],
                color: [255, 0, 255, 255],
            },
            Vertex {
                pos: [dim, -dim, 0.0],
                uv: [1.0, 0.0],
                color: [0, 255, 255, 255],
            },
            Vertex {
                pos: [dim, dim, 0.0],
                uv: [1.0, 1.0],
                color: [255, 255, 0, 255],
            },
            Vertex {
                pos: [-dim, dim, 0.0],
                uv: [0.0, 1.0],
                color: [255, 255, 255, 255],
            },
        ];

        prim.indices = vec![0, 1, 2, 2, 3, 0];

        prim
    }

    pub fn draw(&self) {
        // gl::rotatef(self.rotation, 0.0, 0.0, 1.0);
        gl::bind_texture(gl::TEXTURE_2D, self.texture.gl_id);

        self.enable_client_state();
        gl::bind_buffer(gl::ARRAY_BUFFER, self.vbo);
        gl::bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
        gl::draw_elements(gl::TRIANGLES, self.indices.len(), gl::UNSIGNED_INT, 0);
        // gl::bind_texture(gl::TEXTURE_2D, 0);
        // gl::bind_buffer(gl::ARRAY_BUFFER, 0);
        // gl::bind_buffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        self.disable_client_state();
    }

    pub fn buffer(&mut self) {
        if self.vbo == 0 {
            self.vbo = gl::gen_buffer();
        }
        gl::bind_buffer(gl::ARRAY_BUFFER, self.vbo);
        gl::buffer_data(gl::ARRAY_BUFFER, &self.verts, gl::STATIC_DRAW);

        if self.ibo == 0 {
            self.ibo = gl::gen_buffer();
        }
        gl::bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.ibo);
        gl::buffer_data(gl::ELEMENT_ARRAY_BUFFER, &self.indices, gl::STATIC_DRAW);

        gl::bind_buffer(gl::ARRAY_BUFFER, 0);
        // gl::bind_buffer(gl::ELEMENT_ARRAY_BUFFER, 0);
    }

    fn enable_client_state(&self) {
        gl::enable_client_state(gl::VERTEX_ARRAY);
        gl::enable_client_state(gl::TEXTURE_COORD_ARRAY);
        gl::enable_client_state(gl::COLOR_ARRAY);

        gl::vertex_pointer::<Vertex>(0);
        gl::tex_coord_pointer::<Vertex>(12);
        gl::color_pointer::<Vertex>(4, gl::UNSIGNED_BYTE, 20);
    }

    fn disable_client_state(&self) {
        gl::disable_client_state(gl::VERTEX_ARRAY);
        gl::disable_client_state(gl::TEXTURE_COORD_ARRAY);
        gl::disable_client_state(gl::COLOR_ARRAY);
    }
}
