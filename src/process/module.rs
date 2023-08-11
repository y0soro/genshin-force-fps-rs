use core::ffi::c_void;
use std::io::Cursor;

use patternscan;
use windows::Win32::System::Memory::{VirtualFree, MEM_RELEASE};

#[derive(Debug)]
pub struct Module {
    pub(super) base_addr: *mut c_void,
    pub(super) base_size: usize,
    pub(super) snapshot_mem: *mut c_void,
}

impl Module {
    pub fn pattern_scan(&self, pattern: &str) -> Option<*mut u8> {
        unsafe {
            let mem_slice = ::core::slice::from_raw_parts_mut(
                self.snapshot_mem as *mut u8,
                self.base_size as usize,
            );

            let offset = patternscan::scan_first_match(Cursor::new(mem_slice), pattern).ok()??;
            Some(self.base_addr.add(offset) as _)
        }
    }

    pub fn snapshot_addr(&self, ps_addr: *mut u8) -> *mut u8 {
        unsafe {
            let offset = ps_addr.offset_from(self.base_addr as _);
            if offset < 0 || offset >= self.base_size as isize {
                panic!(
                    "{:?} out of bounds, [{:?}, {:?}]",
                    ps_addr,
                    self.base_addr,
                    self.base_addr.offset(self.base_size as _)
                );
            }
            self.snapshot_mem.offset(offset) as *mut u8
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            VirtualFree(self.snapshot_mem, 0, MEM_RELEASE);
        }
    }
}
