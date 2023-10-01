pub mod module;

use core::ffi::c_void;
use core::mem;
use core::ptr;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, HINSTANCE, MAX_PATH, STILL_ACTIVE,
};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows::Win32::System::ProcessStatus::{
    K32EnumProcessModules, K32GetModuleBaseNameW, K32GetModuleInformation, MODULEINFO,
};
use windows::Win32::System::Threading::{
    CreateProcessW, GetExitCodeProcess, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOW,
};

use crate::utils::*;

#[derive(Debug)]
pub struct Process {
    handle: HANDLE,
    _pid: u32,
}

impl Process {
    pub fn create(exec_path: &str, exec_wd: Option<&str>, args: &str) -> Result<Self, String> {
        let exec_wd_holder: Vec<u16>;
        let exec_wd = if let Some(s) = exec_wd {
            exec_wd_holder = str_to_w_vec(s);
            PCWSTR(exec_wd_holder.as_ptr())
        } else {
            PCWSTR::default()
        };

        let mut args = str_to_w_vec(args);
        let ps_info = &mut PROCESS_INFORMATION::default();

        unsafe {
            let ok = CreateProcessW(
                exec_path,
                PWSTR(args.as_mut_ptr()),
                ptr::null(),
                ptr::null(),
                false,
                PROCESS_CREATION_FLAGS(0),
                ptr::null(),
                exec_wd,
                &STARTUPINFOW::default(),
                ps_info,
            )
            .as_bool();
            if !ok {
                return Err(format!("failed to create process: {:?}", GetLastError()));
            }
            CloseHandle(ps_info.hThread);
        };

        Ok(Self {
            handle: ps_info.hProcess,
            _pid: ps_info.dwProcessId,
        })
    }

    unsafe fn enum_modules(&self) -> Result<Vec<HINSTANCE>, String> {
        let mut lpcbneeded: u32 = 1024;
        let mut lphmodule = vec![];

        while lphmodule.len() < lpcbneeded as _ {
            lphmodule.resize(lpcbneeded as _, HINSTANCE::default());

            let ok = K32EnumProcessModules(
                self.handle,
                lphmodule.as_mut_ptr(),
                (lphmodule.len() * mem::size_of::<HINSTANCE>()) as u32,
                &mut lpcbneeded,
            )
            .as_bool();
            if !ok {
                return Err("failed to enum modules".to_owned());
            }
        }
        lphmodule.truncate(lpcbneeded as _);
        Ok(lphmodule)
    }

    pub fn get_module(&self, name: &str) -> Result<module::Module, String> {
        unsafe {
            let hmodule = 'outer: {
                for hmodule in self.enum_modules()? {
                    let mut module_name: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
                    K32GetModuleBaseNameW(self.handle, hmodule, &mut module_name);
                    let module_name = w_to_str(&module_name);
                    if module_name == name {
                        break 'outer hmodule;
                    }
                }
                return Err("module not found".to_owned());
            };

            let mut info = MODULEINFO::default();
            let ok = K32GetModuleInformation(
                self.handle,
                hmodule,
                &mut info,
                mem::size_of_val(&info) as _,
            )
            .as_bool();
            if !ok {
                return Err("failed to get module info".to_owned());
            }
            let base_addr = info.lpBaseOfDll;
            let base_size = info.SizeOfImage as usize;

            let snapshot_mem = VirtualAlloc(
                ptr::null(),
                base_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            );
            if snapshot_mem.is_null() {
                return Err("failed to allocate snapshot mem".to_owned());
            }

            let ok = ReadProcessMemory(
                self.handle,
                base_addr,
                snapshot_mem,
                base_size,
                ptr::null_mut(),
            )
            .as_bool();
            if !ok {
                VirtualFree(snapshot_mem, 0, MEM_RELEASE);
                return Err("failed to read module memory".to_owned());
            }

            Ok(module::Module {
                base_addr,
                base_size,
                snapshot_mem,
            })
        }
    }

    pub fn read<T>(&self, base: *const u8) -> Result<T, String> {
        unsafe {
            let mut buffer: T = mem::zeroed();

            let ok = ReadProcessMemory(
                self.handle,
                base as *const c_void,
                &mut buffer as *mut _ as *mut c_void,
                mem::size_of_val(&buffer),
                ptr::null_mut(),
            )
            .as_bool();
            if !ok {
                return Err("failed to read memory".to_owned());
            }
            Ok(buffer)
        }
    }

    pub fn write<T>(&self, base: *const u8, value: &T) -> Result<(), String> {
        unsafe {
            let ok = WriteProcessMemory(
                self.handle,
                base as *const c_void,
                value as *const _ as *const c_void,
                mem::size_of_val(value),
                ptr::null_mut(),
            )
            .as_bool();
            if !ok {
                return Err("failed to write memory".to_owned());
            }
        }
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        unsafe {
            let mut exitcode: u32 = 0;
            let ok = GetExitCodeProcess(self.handle, &mut exitcode as *mut _).as_bool();
            if !ok {
                return false;
            }
            exitcode == STILL_ACTIVE.0 as u32
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}
