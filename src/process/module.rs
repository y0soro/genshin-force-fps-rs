use core::ffi::c_void;
use std::io::Cursor;

use patternscan;
use windows::Win32::System::Diagnostics::ToolHelp::MODULEENTRY32;
use windows::Win32::System::Memory::{VirtualFree, MEM_RELEASE};

#[derive(Debug)]
pub struct Module {
    pub(super) entry: MODULEENTRY32,
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
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            VirtualFree(self.snapshot_mem, 0, MEM_RELEASE);
        }
    }
}
