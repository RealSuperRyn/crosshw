#![feature(sync_unsafe_cell)]
#![no_std]
#![allow(dead_code, unused_imports)]
pub mod boot;
pub mod elf;
pub mod logging;
pub mod memory;
pub mod misc;
mod tests {
    use crate::memory::paging::*;
    use crate::misc::uflags;
    #[test]
    fn checkalignment() {
        assert_eq!(
            core::mem::size_of::<crate::memory::paging::PageTable>(),
            4096
        );
        assert_eq!(core::mem::size_of::<crate::memory::paging::PageEntry>(), 8);
    }
    #[test]
    fn uniontest() {
        let foo = PageEntry::from_u64(8192u64 | PRESENT as u64 | AVAILABLE_4 as u64);
        assert_eq!(foo.flags().0, PRESENT | AVAILABLE_4);
        assert_eq!(foo.address(), 8192u64);
    }
}
