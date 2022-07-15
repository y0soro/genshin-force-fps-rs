pub mod module;

use core::char;
use core::ffi::c_void;
use core::mem;
use core::ptr;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, STILL_ACTIVE};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Module32FirstW, Module32NextW, MODULEENTRY32W, TH32CS_SNAPMODULE,
};
use windows::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};
use windows::Win32::System::Threading::{
    CreateProcessW, GetExitCodeProcess, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOW,
};

#[derive(Debug)]
pub struct Process {
    handle: HANDLE,
    pid: u32,
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
            pid: ps_info.dwProcessId,
        })
    }

    pub fn get_module(&self, name: &str) -> Result<module::Module, String> {
        let mut entry = MODULEENTRY32W::default();
        unsafe {
            let snap_h = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, self.pid)
                .or_else(|o| Err(o.message().to_string_lossy()))?;

            entry.dwSize = mem::size_of::<MODULEENTRY32W>() as u32;
            if !Module32FirstW(snap_h, &mut entry).as_bool() {
                CloseHandle(snap_h);
                return Err("no module available".to_owned());
            }

            loop {
                if entry.th32ProcessID != self.pid {
                    continue;
                }
                let module_name = w_to_str(&entry.szModule);

                if module_name == name {
                    break;
                }

                if !Module32NextW(snap_h, &mut entry).as_bool() {
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
    s.encode_utf16().chain(::core::iter::once(0)).collect()
}

fn w_to_str(wide: &[u16]) -> String {
    let i = wide.iter().cloned().take_while(|&c| c != 0);
    char::decode_utf16(i)
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}
