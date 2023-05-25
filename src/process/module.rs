use core::ffi::c_void;
use std::io::Cursor;

use patternscan;
use windows::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32W;
use windows::Win32::System::Memory::{VirtualFree, MEM_RELEASE};

#[derive(Debug)]
pub struct Module {
    pub(super) entry: MODULEENTRY32W,
    pub(super) snapshot_mem: *mut c_void,
}

impl Module {
    pub fn pattern_scan(&self, pattern: &str) -> Option<*mut u8> {
        unsafe {
            let mem_slice = ::core::slice::from_raw_parts_mut(
                self.snapshot_mem as *mut u8,
                self.entry.modBaseSize as usize,
            );

            let offset = patternscan::scan_first_match(Cursor::new(mem_slice), pattern).ok()??;
            Some(self.entry.modBaseAddr.add(offset))
        }
    }

    pub fn snapshot_addr(&self, ps_addr: *mut u8) -> *mut u8 {
        unsafe {
            let offset = ps_addr.offset_from(self.entry.modBaseAddr);
            if offset < 0 || offset >= self.entry.modBaseSize as isize {
                panic!(
                    "{:?} out of bounds, [{:?}, {:?}]",
                    ps_addr,
                    self.entry.modBaseAddr,
                    self.entry.modBaseAddr.offset(self.entry.modBaseSize as _)
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
