#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Point3D {
    pub sync_word: u16,
    pub x_mm: i16,
    pub y_mm: i16,
    pub z_mm: i16,
    pub quality: u16
}