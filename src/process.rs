use crate::error::{Error, Result};
use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HMODULE};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Memory::{
    VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_EXECUTE_READ,
    PAGE_EXECUTE_READWRITE, PAGE_READONLY, PAGE_READWRITE,
};
use windows::Win32::System::ProcessStatus::{
    EnumProcessModulesEx, GetModuleBaseNameW, GetModuleInformation, LIST_MODULES_ALL, MODULEINFO,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

pub struct Process {
    handle: HANDLE,
    pid: u32,
    name: String,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub base: usize,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub base: usize,
    pub size: usize,
}

impl Process {
    pub fn open(pid: u32) -> Result<Self> {
        let handle = unsafe {
            OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, false, pid)?
        };

        if handle.is_invalid() {
            return Err(Error::OpenProcessFailed(format!("PID: {}", pid)));
        }

        Ok(Self { handle, pid, name: String::new() })
    }

    pub fn open_by_name(name: &str) -> Result<Self> {
        let pid = Self::find_pid_by_name(name)?;
        let mut process = Self::open(pid)?;
        process.name = name.to_string();
        Ok(process)
    }

    fn find_pid_by_name(name: &str) -> Result<u32> {
        use windows::Win32::System::ProcessStatus::EnumProcesses;

        let mut pids = [0u32; 2048];
        let mut bytes_returned = 0u32;

        unsafe {
            EnumProcesses(pids.as_mut_ptr(), (pids.len() * 4) as u32, &mut bytes_returned)?;
        }

        let count = bytes_returned as usize / 4;
        let target_lower = name.to_lowercase();

        for &pid in &pids[..count] {
            if pid == 0 { continue; }

            let handle = unsafe {
                match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) {
                    Ok(h) => h,
                    Err(_) => continue,
                }
            };

            if handle.is_invalid() { continue; }

            let mut module = HMODULE::default();
            let mut needed = 0u32;

            let success = unsafe {
                EnumProcessModulesEx(
                    handle, &mut module, mem::size_of::<HMODULE>() as u32,
                    &mut needed, LIST_MODULES_ALL,
                ).is_ok()
            };

            if success {
                let mut name_buf = [0u16; 260];
                let len = unsafe { GetModuleBaseNameW(handle, module, &mut name_buf) } as usize;

                if len > 0 {
                    let process_name = OsString::from_wide(&name_buf[..len])
                        .to_string_lossy()
                        .to_lowercase();

                    if process_name == target_lower || process_name.contains(&target_lower) {
                        unsafe { CloseHandle(handle).ok() };
                        return Ok(pid);
                    }
                }
            }

            unsafe { CloseHandle(handle).ok() };
        }

        Err(Error::ProcessNotFound(name.to_string()))
    }

    pub fn pid(&self) -> u32 { self.pid }
    pub fn handle(&self) -> HANDLE { self.handle }

    pub fn read(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0;

        let success = unsafe {
            ReadProcessMemory(
                self.handle, address as *const _, buffer.as_mut_ptr() as *mut _,
                size, Some(&mut bytes_read),
            )
        };

        if success.is_err() || bytes_read == 0 {
            return Err(Error::ReadMemoryFailed(address));
        }

        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    pub fn read_value<T: Copy>(&self, address: usize) -> Result<T> {
        let data = self.read(address, mem::size_of::<T>())?;
        Ok(unsafe { *(data.as_ptr() as *const T) })
    }

    pub fn modules(&self) -> Result<Vec<ModuleInfo>> {
        let mut modules = [HMODULE::default(); 1024];
        let mut needed = 0u32;

        unsafe {
            EnumProcessModulesEx(
                self.handle, modules.as_mut_ptr(),
                mem::size_of_val(&modules) as u32, &mut needed, LIST_MODULES_ALL,
            )?;
        }

        let count = needed as usize / mem::size_of::<HMODULE>();
        let mut result = Vec::with_capacity(count);

        for &module in &modules[..count] {
            let mut name_buf = [0u16; 260];
            let len = unsafe { GetModuleBaseNameW(self.handle, module, &mut name_buf) } as usize;
            if len == 0 { continue; }

            let name = OsString::from_wide(&name_buf[..len]).to_string_lossy().to_string();

            let mut info = MODULEINFO::default();
            unsafe {
                GetModuleInformation(
                    self.handle, module, &mut info, mem::size_of::<MODULEINFO>() as u32,
                )?;
            }

            result.push(ModuleInfo {
                name,
                base: info.lpBaseOfDll as usize,
                size: info.SizeOfImage as usize,
            });
        }

        Ok(result)
    }

    pub fn find_module(&self, name: &str) -> Result<ModuleInfo> {
        let name_lower = name.to_lowercase();
        self.modules()?
            .into_iter()
            .find(|m| m.name.to_lowercase() == name_lower)
            .ok_or_else(|| Error::ModuleNotFound(name.to_string()))
    }

    pub fn get_module_base(&self, name: &str) -> Result<usize> {
        Ok(self.find_module(name)?.base)
    }

    pub fn memory_regions(&self) -> Vec<MemoryRegion> {
        let mut regions = Vec::new();
        let mut address = 0usize;

        loop {
            let mut info = MEMORY_BASIC_INFORMATION::default();
            let result = unsafe {
                VirtualQueryEx(
                    self.handle, Some(address as *const _),
                    &mut info, mem::size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };

            if result == 0 { break; }

            if info.State == MEM_COMMIT {
                let protect = info.Protect;
                if protect == PAGE_READONLY || protect == PAGE_READWRITE
                    || protect == PAGE_EXECUTE_READ || protect == PAGE_EXECUTE_READWRITE
                {
                    regions.push(MemoryRegion {
                        base: info.BaseAddress as usize,
                        size: info.RegionSize,
                    });
                }
            }

            address = info.BaseAddress as usize + info.RegionSize;
        }

        regions
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle).ok(); }
    }
}
