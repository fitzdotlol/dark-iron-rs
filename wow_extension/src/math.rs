#[repr(C)]
pub struct Matrix4 {
    pub m: [[f32; 4]; 4],
}

#[repr(C)]
pub struct RectI {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}
