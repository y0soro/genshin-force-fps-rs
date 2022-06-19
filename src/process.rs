pub mod module;

use core::ffi::c_void;
use core::mem;
use core::ptr;
use std::ffi::CStr;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, BOOL, HANDLE, STILL_ACTIVE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows::Win32::System::Threading::{
    CreateProcessW, GetExitCodeProcess, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOW,
};

use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32First, Module32Next, MODULEENTRY32, TH32CS_SNAPMODULE,
};

#[derive(Debug)]
pub struct Process {
    handle: HANDLE,
    pid: u32,
}

impl Process {
    pub fn create(exec_path: &str, args: &str) -> Result<Self, String> {
        let exec_wd = &exec_path[..exec_path.rfind("\\").unwrap()];

        let exec_path = str_to_w_vec(exec_path);
        let mut args = str_to_w_vec(args);
        let exec_wd = str_to_w_vec(exec_wd);
        let ps_info = &mut PROCESS_INFORMATION::default();

        unsafe {
            let ok = CreateProcessW(
                PCWSTR(exec_path.as_ptr()),
                PWSTR(args.as_mut_ptr()),
                ptr::null(),
                ptr::null(),
                BOOL::from(false),
                PROCESS_CREATION_FLAGS(0),
                ptr::null(),
                PCWSTR(exec_wd.as_ptr()),
                &STARTUPINFOW::default(),
                ps_info,
            )
            .as_bool();
            if !ok {
                return Err(format!("failed to create process: {:?}", GetLastError()));
            }
            CloseHandle(ps_info.hThread);
        };

        let mut res: Self = unsafe { ::core::mem::zeroed() };

        res.handle = ps_info.hProcess;
        res.pid = ps_info.dwProcessId;

        Ok(res)
    }

    pub fn get_module(&self, name: &str) -> Result<module::Module, String> {
        let mut entry = MODULEENTRY32::default();
        unsafe {
            let snap_h = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, self.pid)
                .or_else(|o| Err(o.message().to_string_lossy()))?;

            entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;
            if !Module32First(snap_h, &mut entry).as_bool() {
                CloseHandle(snap_h);
                return Err("no module available".to_owned());
            }

            loop {
                if entry.th32ProcessID != self.pid {
                    continue;
                }
                let module_name = CStr::from_ptr(entry.szModule.as_ptr() as *const i8)
                    .to_str()
                    .unwrap();
                if module_name == name {
                    break;
                }

                if !Module32Next(snap_h, &mut entry).as_bool() {
                    return Err("module not found".to_owned());
                }
            }

            let snapshot_mem = VirtualAlloc(
                ptr::null(),
                entry.modBaseSize as usize,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            );
            if snapshot_mem.is_null() {
                return Err("failed to allocate snapshot mem".to_owned());
            }

            let ok = ReadProcessMemory(
                self.handle,
                entry.modBaseAddr as *const c_void,
                snapshot_mem,
                entry.modBaseSize as usize,
                ptr::null_mut(),
            )
            .as_bool();

            if !ok {
                VirtualFree(snapshot_mem, 0, MEM_RELEASE);
                return Err("failed to read module memory".to_owned());
            }

            Ok(module::Module {
                entry,
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

fn str_to_w_vec(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = OsString::from(s).encode_wide().collect();
    v.push(0u16);
    v
}
