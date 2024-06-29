use crate::misc::uflags::Flags16;
#[cfg(target_arch = "x86_64")]
use core::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};
use x86_64::{
    addr::PhysAddr,
    addr::VirtAddr,
    structures::paging::{frame::PhysFrame, FrameAllocator, Size4KiB},
};
//Page directory entry attributes. (x86_64)
pub const PRESENT: u16 = 1; //Sets whether or not the page is present. If the MMU accesses a non-present page, it causes a page fault.
pub const WRITE_ENABLE: u16 = 1 << 1; //If this flag is set, it'll allow write access. If not, the page will be read-only.
pub const USER_ACCESSIBLE: u16 = 1 << 2; //If this flag is set, the user can use the page. Otherwise, only the kernel can do so.
pub const WRITE_THROUGH_ENABLE: u16 = 1 << 3; //If this flag is set, the page uses write-through caching. Otherwise, it uses write-back.
pub const CACHE_DISABLE: u16 = 1 << 4; //If this flag is set, caching is disabled for the page. Otherwise, it isn't.
pub const ACCESSED: u16 = 1 << 5; //Set by the MMU if the page entry was accessed when translating a virtual address.
pub const WRITTEN_TO: u16 = 1 << 6; //The "dirty" bit, set by the MMU if the page was written to.
pub const AVAILABLE_1: u16 = 1 << 7; //Free bit for use by the OS.
pub const LARGE_PAGE: u16 = 1 << 8; //Set if the page is 4MiB. Must be aligned to 4MiB for that.
pub const AVAILABLE_2: u16 = 1 << 9; //Free bit for use by the OS.
pub const AVAILABLE_3: u16 = 1 << 10; //Free bit for use by the OS.
pub const AVAILABLE_4: u16 = 1 << 11; //Free bit for use by the OS.

pub const OFFSET: u64 = 0xB0000000;

pub union PageEntry {
    address: u64,
    flags: Flags16,
}
impl PageEntry {
    pub fn from_u64(num: u64) -> PageEntry {
        PageEntry { address: num }
    }
    pub fn address(&self) -> u64 {
        unsafe { self.address & !4095 }
    }
    pub fn flags(&self) -> Flags16 {
        unsafe { self.flags.truncate_bits(4) }
    }
    pub unsafe fn set_address(&mut self, addr: u64) {
        self.address = addr & !4095 //Force-aligns the address to 4KiB
    }
    pub unsafe fn set_flags(&mut self, flags: Flags16) {
        self.flags = flags.truncate_bits(4)
    }
    pub unsafe fn zero(&mut self, offset: u64) {
        //You must make sure the page isn't in use to use this correctly
        let ptr = (self.address() + offset) as *mut u8;
        for i in 0..4095 {
            unsafe {
                *(ptr.wrapping_add(i)) = 0;
            }
        }
    }
    pub fn init_page<T: FrameAllocator<Size4KiB>>(&mut self, fralloc: &mut T) {
        unsafe {
            self.set_address(fralloc.allocate_frame().unwrap().start_address().as_u64());
            self.set_flags(Flags16::from_u16(PRESENT));
        }
    }
    pub fn init_page_with_paddr(&mut self, paddr: u64) {
        unsafe {
            self.set_address(paddr);
            self.set_flags(Flags16::from_u16(PRESENT));
        }
    }
    pub fn as_raw_parts(&self) -> (u64, u16) {
        unsafe { (self.address(), self.flags().0) }
    }
}
#[repr(align(4096))]
pub struct PageTable {
    table: [PageEntry; 512],
}
impl Index<usize> for PageTable {
    type Output = PageEntry;
    fn index(&self, index: usize) -> &PageEntry {
        &self.table[index]
    }
}
impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut PageEntry {
        &mut self.table[index]
    }
}
pub struct PageHierarchy {
    root: *mut u8,
}
impl PageHierarchy {
    pub unsafe fn new<T: FrameAllocator<Size4KiB>>(
        fralloc: &mut T,
        physpagecount: usize,
    ) -> PageHierarchy {
        //Call this from the bootloader. It will create the system's page hierarchy.
        let mut hierarchy = PageHierarchy {
            root: fralloc.allocate_frame().unwrap().start_address().as_u64() as *mut u8,
        };
        loop {}
        let zerotable = &mut *(hierarchy.get_table_at_vaddr(fralloc, 0) as *mut PageTable);
        zerotable[0].init_page(fralloc);
        unsafe { zerotable[0].zero(0) };
        hierarchy.init_direct_mapping(fralloc, physpagecount, OFFSET);
        hierarchy
    }
    pub unsafe fn enable_paging(&mut self) {
        //x86_64::registers::control::Cr4::write_raw(1 << 5);
        self.root = self.root.wrapping_add(OFFSET as usize);
        x86_64::registers::control::Cr3::write_raw(
            PhysFrame::from_start_address(PhysAddr::new(self.root as u64)).unwrap(),
            0,
        );
    }
    pub fn init_direct_mapping<T: FrameAllocator<Size4KiB>>(
        &mut self,
        fralloc: &mut T,
        memory_page_count: usize,
        offset: u64,
    ) {
        assert_eq!(
            offset & (!4095),
            offset,
            "Page direct mapping offset not aligned!"
        );
        for i in 0..memory_page_count - 1 {
            let table = unsafe {
                &mut *(self.get_table_at_vaddr(fralloc, offset + (0x1000u64 * i as u64))
                    as *mut PageTable)
            };
            let indices = PageHierarchy::vaddr_into_indices(offset + (0x1000u64 * i as u64));
            table[indices.3].init_page_with_paddr(0x1000 * i as u64);
        }
    }
    pub fn vaddr_into_indices(virt_address: u64) -> (usize, usize, usize, usize) {
        (
            ((virt_address & (511 << 39)) >> 39) as usize,
            ((virt_address & (511 << 30)) >> 30) as usize,
            ((virt_address & (511 << 21)) >> 21) as usize,
            ((virt_address & (511 << 12)) >> 12) as usize,
        )
    }
    pub fn get_table_at_vaddr<T: FrameAllocator<Size4KiB>>(
        &mut self,
        fralloc: &mut T,
        virt_address: u64,
    ) -> *mut u8 {
        let indices = PageHierarchy::vaddr_into_indices(virt_address);
        let root = unsafe { &mut *(self.root as *mut PageTable) };
        let second_level: &mut PageTable = if root[indices.0].flags() == Flags16::from_u16(PRESENT)
        {
            unsafe { &mut *(root[indices.0].address() as *mut PageTable) }
        } else {
            unsafe {
                root[indices.0].init_page(fralloc);
                &mut *(root[indices.0].address() as *mut PageTable)
            }
        };
        let third_level: &mut PageTable =
            if second_level[indices.1].flags() == Flags16::from_u16(PRESENT) {
                unsafe { &mut *(second_level[indices.1].address() as *mut PageTable) }
            } else {
                unsafe {
                    second_level[indices.1].init_page(fralloc);
                    &mut *(second_level[indices.1].address() as *mut PageTable)
                }
            };
        if third_level[indices.2].flags() == Flags16::from_u16(PRESENT) {
            third_level[indices.2].address() as *mut u8
        } else {
            third_level[indices.2].init_page(fralloc);
            third_level[indices.2].address() as *mut u8
        }
    }
}
//We use 64-bit pointers here
//#[repr(align(4096))]
//pub struct PageTable ([u64; 512]);
//impl PageTable {
//
//}

//IMPLEMENT FOLLOWING COMMENTED CODE PROPERLY: after entering Stage 2 (bootvoid -> kernelspace)

//Struct definition is universal for now. May change later when adding additional architectures.
//pub struct FrameAlloc {
//	kernel_end_addr: usize,
//}
//#[cfg(target_arch = "x86_64")]
//impl FrameAlloc {}
//#[cfg(target_arch = "x86_64")]
//unsafe impl FrameAllocator<Size4KiB> for FrameAlloc {
//		fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
//
//		}
//}
