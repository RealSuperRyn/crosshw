#[derive(Clone)]
pub struct FrameBuf {
    pub fb: u64,
    pub model: FBModel,
    pub mode: FBMode,
}
#[derive(Clone)]
#[repr(u8)]
pub enum FBModel {
    RGB = 0,
    BGR = 1,
}
#[derive(Clone)]
pub struct FBMode {
    pub bitsperpixel: u16,
    pub width: u64,
    pub height: u64,
}
impl FrameBuf {
    pub unsafe fn set_pixel(&self, color: u32, x: usize, y: usize) {
        *((self.fb as *mut u8).add(y * self.mode.width as usize + x) as *mut u32) = color;
    }
}
