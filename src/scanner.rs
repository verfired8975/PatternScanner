use crate::error::Result;
use crate::pattern::Pattern;
use crate::process::{MemoryRegion, ModuleInfo, Process};

pub struct Scanner {
    process: Process,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub address: usize,
    pub module: Option<String>,
    pub offset: Option<usize>,
}

impl ScanResult {
    pub fn rip_relative(&self, process: &Process, inst_len: usize) -> Result<usize> {
        let data = process.read(self.address + inst_len - 4, 4)?;
        let rel = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        Ok((self.address + inst_len).wrapping_add(rel as usize))
    }

    pub fn add(&self, offset: isize) -> usize {
        (self.address as isize + offset) as usize
    }
}

impl Scanner {
    pub fn attach(process_name: &str) -> Result<Self> {
        let process = Process::open_by_name(process_name)?;
        Ok(Self { process })
    }

    pub fn from_process(process: Process) -> Self {
        Self { process }
    }

    pub fn process(&self) -> &Process {
        &self.process
    }

    pub fn scan(&self, pattern: &Pattern) -> Result<Vec<ScanResult>> {
        let regions = self.process.memory_regions();
        let modules = self.process.modules().unwrap_or_default();
        self.scan_regions(pattern, &regions, &modules)
    }

    pub fn scan_module(&self, pattern: &Pattern, module_name: &str) -> Result<Vec<ScanResult>> {
        let module = self.process.find_module(module_name)?;
        let region = MemoryRegion { base: module.base, size: module.size };
        self.scan_regions(pattern, &[region], &[module])
    }

    pub fn scan_range(&self, pattern: &Pattern, start: usize, size: usize) -> Result<Vec<ScanResult>> {
        let region = MemoryRegion { base: start, size };
        self.scan_regions(pattern, &[region], &[])
    }

    pub fn find(&self, pattern: &Pattern) -> Result<Option<ScanResult>> {
        Ok(self.scan(pattern)?.into_iter().next())
    }

    pub fn find_in(&self, pattern: &Pattern, module: &str) -> Result<Option<ScanResult>> {
        Ok(self.scan_module(pattern, module)?.into_iter().next())
    }

    fn scan_regions(
        &self,
        pattern: &Pattern,
        regions: &[MemoryRegion],
        modules: &[ModuleInfo],
    ) -> Result<Vec<ScanResult>> {
        let mut results = Vec::new();
        let chunk = 0x10000; // 64kb parcalar halinde okuyoz yoksa cok yavas

        for region in regions {
            let mut offset = 0;

            while offset < region.size {
                let read_size = (region.size - offset).min(chunk + pattern.len());
                let addr = region.base + offset;

                if let Ok(data) = self.process.read(addr, read_size) {
                    for i in 0..data.len().saturating_sub(pattern.len()) {
                        if pattern.matches(&data, i) {
                            let found = addr + i;
                            let (module, mod_offset) = Self::which_module(found, modules);

                            results.push(ScanResult {
                                address: found,
                                module,
                                offset: mod_offset,
                            });
                        }
                    }
                }

                offset += chunk;
            }
        }

        Ok(results)
    }

    fn which_module(addr: usize, modules: &[ModuleInfo]) -> (Option<String>, Option<usize>) {
        for m in modules {
            if addr >= m.base && addr < m.base + m.size {
                return (Some(m.name.clone()), Some(addr - m.base));
            }
        }
        (None, None)
    }
}
