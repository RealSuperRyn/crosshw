use core::cell::SyncUnsafeCell;
extern crate std;
use std::io::Write;
///Used to store read-only temporary information. Overwrites the information at the start when it's full.
const LOG_BUFFER_SIZE: usize = 0xFFFF;
pub struct LogBuffer<T: Copy> {
    buf: SyncUnsafeCell<[T; LOG_BUFFER_SIZE + 1]>,
    start_offset: usize,
    end_offset: usize,
    since_last_flush: usize,
}
impl<T: Copy> LogBuffer<T> {
    pub fn append_buf(&mut self, buf: &[T]) {
        for i in buf.iter() {
            self.append_byte(*i);
        }
    }
    pub fn append_byte(&mut self, byte: T) {
        let buf = self.buf.get_mut();
        buf[self.end_offset] = byte;
        if self.start_offset == self.end_offset {
            self.start_offset = ((self.start_offset + 1) % LOG_BUFFER_SIZE);
        }
        self.end_offset = ((self.end_offset + 1) % LOG_BUFFER_SIZE);
    }
    pub fn flush() {}
}
