#![allow(dead_code)]

use bytemuck::cast_slice;
use fs2::FileExt;
use memmap2::{MmapMut, MmapOptions};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::hint::black_box;
use std::io::{self, Read, Seek, SeekFrom, Write};

// Base address of PCIe2CPP BAR CSRs.
const BAR_CONFIG_BASE_PCIE_INTERNAL: u32 = 0x30000; // When accessed by PCIe internal target.
pub const BAR_CONFIG_BASE_CONFIG_SNOOP: u32 = 0xA00; // When accessed by config snoop i/f
                                                     // Offset from BAR config base address to expansion BAR CSRs.
const EXPANSION_BAR_BASE_OFFSET: u32 = 0x0;
// Offset from expansion BAR CSR base address per physical BAR.
const EXPANSION_BAR_PHYS_OFFSET: u32 = 0x40;
// Offset from physical BAR expansion BAR CSR base address per expansion BAR.
const EXPANSION_BAR_CSR_OFFSET: u32 = 0x8;

// Physical BAR for CPP transactions.
// Physical BARs 0 and 1 are reserved for the NSP and application firmware.
// For debugging, we may only configure physical BAR 2 to avoid any conflict.
const CPP_EXPANSION_BAR_PHYSICAL_BAR: u32 = 2;
// Maximum number of expansion BARs.
const CPP_MAX_NUM_EXPANSION_BARS: u32 = 8;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MapType {
    Fixed,
    Bulk,
    Target,
    General,
    Explicit,
}

pub fn init_device_bars(pci_bdf: &str) {
    let pcie_cfg_path = format!("/sys/bus/pci/devices/{}/config", pci_bdf);
    let mut pcie_cfg_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&pcie_cfg_path)
        .expect(&format!("Failed to open file {}", &pcie_cfg_path));
    pcie_cfg_file
        .seek(SeekFrom::Start(4))
        .expect(&format!("File {} seek failed", pcie_cfg_path));
    let mut buf = [0u8; 1];
    pcie_cfg_file
        .read(&mut buf)
        .expect(&format!("File {} read failed", &pcie_cfg_path));
    let cfg_val = buf[0] | 0x06;
    pcie_cfg_file
        .seek(SeekFrom::Start(4))
        .expect(&format!("File {} seek failed", &pcie_cfg_path));
    pcie_cfg_file
        .write(&[cfg_val])
        .expect(&format!("File {} write failed", &pcie_cfg_path));
}

pub struct ExpansionBar {
    pci_bdf: String,
    phys_bar: u8,
    phys_bar_path: String,
    exp_bar: u8,
    pub exp_bar_map: MapType,
    exp_bar_cached_cfg: [u32; 2],
    pub exp_bar_base_addr: u64,
    pub exp_bar_size: u64,
    lock_file: File,
    mmap_file: Option<File>,
    mmap_region: Option<MmapMut>,
}

impl ExpansionBar {
    pub fn new(pci_bdf_str: &str, bar_mapping: Option<(u8, u8)>) -> Self {
        let (phys_bar, exp_bar, lock_file) = if let Some(bar_map) = bar_mapping {
            let lock_file_dir = format!("/var/run/nfp_tools/{}", pci_bdf_str);
            let lock_file_name = format!("exp_bar{}-{}_lock", bar_map.0, bar_map.1);
            let full_path = format!("{}/{}", lock_file_dir, lock_file_name);

            match Self::acquire_lock_file(&full_path) {
                Ok(file) => {
                    let (phys_bar, exp_bar) = bar_map;
                    (phys_bar, exp_bar, file)
                }
                Err(_) => {
                    panic!("exp_bar{}-{} should not be locked!", bar_map.0, bar_map.1);
                }
            }
        } else {
            Self::allocate_exp_bar(pci_bdf_str)
        };

        let phys_bar_path = format!(
            "/sys/bus/pci/devices/{}/{}",
            pci_bdf_str,
            format!("resource{}", 2 * phys_bar)
        );

        let metadata = fs::metadata(&phys_bar_path).expect("Error getting file metadata!");
        let phys_bar_size = metadata.len() as u64;
        let exp_bar_size = phys_bar_size / 8;
        let exp_bar_offset = (exp_bar as u64) * exp_bar_size;
        let exp_bar_base_addr = 0;

        let file = OpenOptions::new()
            .read(true)
            .write(true) // Open the file in read-write mode
            .open(&phys_bar_path)
            .expect("Failed to open mmap file in read-write mode");

        let mmap = unsafe {
            MmapOptions::new()
                .offset(exp_bar_offset)
                .len(exp_bar_size as usize)
                .map_mut(&file)
                .expect("Failed to map expansion BAR region")
        };

        ExpansionBar {
            pci_bdf: pci_bdf_str.to_string(),
            phys_bar,
            phys_bar_path,
            exp_bar,
            exp_bar_map: MapType::Fixed,
            exp_bar_cached_cfg: [0; 2],
            exp_bar_base_addr,
            exp_bar_size,
            lock_file,
            mmap_file: Some(file),
            mmap_region: Some(mmap),
        }
    }

    fn acquire_lock_file(lock_path: &str) -> io::Result<File> {
        // Attempt to create and open the file
        let lock_file = File::create(lock_path)?;

        // Attempt to lock the file (using an exclusive lock)
        lock_file.try_lock_exclusive()?;

        Ok(lock_file)
    }

    fn allocate_exp_bar(pci_bdf: &str) -> (u8, u8, File) {
        let lock_file_dir = format!("/var/run/nfp_tools/{}", pci_bdf);
        fs::create_dir_all(&lock_file_dir)
            .expect(&format!("Failed to create dir {}", &lock_file_dir));

        let mut bar_locks: Vec<(u8, u8, String)> = Vec::new();
        for exp_bar in 0..CPP_MAX_NUM_EXPANSION_BARS {
            let lock_file = format!("exp_bar{}-{}_lock", CPP_EXPANSION_BAR_PHYSICAL_BAR, exp_bar);
            let full_path = format!("{}/{}", lock_file_dir, lock_file);
            // Attempt to create the file only if it does not already exist
            bar_locks.push((
                CPP_EXPANSION_BAR_PHYSICAL_BAR as u8,
                exp_bar as u8,
                full_path,
            ));
        }

        for (phys_bar, exp_bar, lock_path) in bar_locks {
            match Self::acquire_lock_file(&lock_path) {
                Ok(file) => return (phys_bar, exp_bar, file),
                Err(_) => {
                    // Continue to next lock if this one fails
                }
            }
        }

        panic!("No expansion BARs available!");
    }

    fn exp_bar_config_write(&self, cfg_reg0: u32, cfg_reg1: u32) {
        let phys_bar_path = format!("/sys/bus/pci/devices/{}/resource0", self.pci_bdf);

        let metadata = fs::metadata(&phys_bar_path).expect("Error getting file metadata!");
        let phys_bar_size = metadata.len() as u64;
        let exp_bar_size = phys_bar_size / 8;

        let file = OpenOptions::new()
            .read(true)
            .write(true) // Open the file in read-write mode
            .open(&phys_bar_path)
            .expect("Failed to open mmap file in read-write mode");

        let mut mmap = unsafe {
            MmapOptions::new()
                .offset(0)
                .len(exp_bar_size as usize)
                .map_mut(&file)
                .expect("Failed to map expansion BAR region")
        };

        let offset = EXPANSION_BAR_BASE_OFFSET
            + ((self.phys_bar as u32) * EXPANSION_BAR_PHYS_OFFSET)
            + ((self.exp_bar as u32) * EXPANSION_BAR_CSR_OFFSET);

        // Write cfg_reg0 into mmap region
        mmap[offset as usize..(offset + 4) as usize].copy_from_slice(cast_slice(&[cfg_reg0]));

        // Read back cfg_reg0 to prevent optimization
        let _cfg_bytes = mmap[offset as usize..(offset + 4) as usize].to_vec();
        black_box(_cfg_bytes);

        // Write cfg_reg1 into mmap region
        mmap[(offset + 4) as usize..(offset + 8) as usize].copy_from_slice(cast_slice(&[cfg_reg1]));

        // Read back cfg_reg1 to prevent optimization
        let _cfg_bytes = mmap[(offset + 4) as usize..(offset + 8) as usize].to_vec();
        black_box(_cfg_bytes);
    }

    fn exp_bar_config_write_via_snoop(&self, cfg_reg0: u32, cfg_reg1: u32) {
        let pcie_cfg_path = format!("/sys/bus/pci/devices/{}/config", &self.pci_bdf);

        let exp_bar_csr_addr = BAR_CONFIG_BASE_CONFIG_SNOOP
            + EXPANSION_BAR_BASE_OFFSET
            + (self.phys_bar as u32) * EXPANSION_BAR_PHYS_OFFSET
            + (self.exp_bar as u32) * EXPANSION_BAR_CSR_OFFSET;

        let mut pcie_cfg_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&pcie_cfg_path)
            .expect(&format!("Failed to open file {}", &pcie_cfg_path));

        pcie_cfg_file
            .seek(SeekFrom::Start(exp_bar_csr_addr as u64))
            .expect(&format!("File {} seek failed", pcie_cfg_path));

        // Write using little-endian format
        pcie_cfg_file
            .write_all(&cfg_reg0.to_le_bytes())
            .expect(&format!("File {} write failed", &pcie_cfg_path));
        pcie_cfg_file
            .write_all(&cfg_reg1.to_le_bytes())
            .expect(&format!("File {} write failed", &pcie_cfg_path));
    }

    pub fn expansion_bar_cfg(
        &mut self,
        tgt_island_id: u8,
        target: u8,
        action: u8,
        token: u8,
        base_addr: u64,
        cpp_len: u8,
    ) {
        let (mut cfg0, mut cfg1): (u32, u32) = (0, 0);

        cfg0 |= 1 << 31; // Enable bit.
        cfg0 |= ((self.exp_bar_map as u32) & 0x7) << 20; // Map type.

        // Early return for explicit mapping.
        if self.exp_bar_map == MapType::Explicit {
            if cfg0 != self.exp_bar_cached_cfg[0] || cfg1 != self.exp_bar_cached_cfg[1] {
                self.exp_bar_config_write(cfg0, cfg1);
                self.exp_bar_cached_cfg[0] = cfg0;
                self.exp_bar_cached_cfg[1] = cfg1;
            }
            return;
        }

        // Check if the base address is valid.
        if (64 - base_addr.leading_zeros()) > 48 {
            panic!("Provided base_addr is too long for a CPP address!")
        }

        let base_addr_width = match self.exp_bar_map {
            MapType::Fixed => 32,
            MapType::Bulk => 38,
            MapType::Target => 40,
            MapType::General => 44,
            _ => 32,
        };

        let lowest_bit = base_addr & base_addr.wrapping_neg(); // Two's complement negation.
        let bit_length = 64 - lowest_bit.leading_zeros();

        if (0..(48 - base_addr_width)).contains(&(bit_length - 1)) {
            panic!(
                "Expansion BAR uses a {}-bit base address. \
                 The lower {} bits of address {:#010x} would be truncated.",
                base_addr_width,
                48 - base_addr_width,
                base_addr
            );
        }

        let mut addr_idx = 48; // Track position in base address.

        // CPP length field.
        cfg0 |= (cpp_len as u32 & 0x3) << 16;

        // Mode address / target island ID field.
        cfg0 |= (tgt_island_id as u32 & 0x7F) << 24;

        // CPP target / base address [43:40] field.
        match self.exp_bar_map {
            MapType::General => {
                cfg0 |= (((base_addr >> (addr_idx - 4)) & 0xF) << 12) as u32;
                addr_idx -= 4;
            }
            _ => {
                cfg0 |= (target as u32 & 0xF) << 12;
            }
        };

        // CPP token / base address [39:38] field.
        match self.exp_bar_map {
            MapType::Fixed | MapType::Bulk => {
                cfg0 |= (token as u32 & 0x3) << 8;
            }
            _ => {
                cfg0 |= (((base_addr >> (addr_idx - 2)) & 0x3) << 8) as u32;
                addr_idx -= 2;
            }
        };

        // CPP action / base address [37:32] field.
        match self.exp_bar_map {
            MapType::Fixed => {
                cfg0 |= action as u32 & 0x3F;
            }
            _ => {
                cfg0 |= ((base_addr >> (addr_idx - 6)) & 0x3F) as u32;
                addr_idx -= 6;
            }
        };

        // Base address [31:0] field.
        cfg1 = (base_addr >> (addr_idx - 32)) as u32;

        // Write configuration if it has changed.
        if cfg0 != self.exp_bar_cached_cfg[0] || cfg1 != self.exp_bar_cached_cfg[1] {
            self.exp_bar_config_write(cfg0, cfg1);
            self.exp_bar_cached_cfg[0] = cfg0;
            self.exp_bar_cached_cfg[1] = cfg1;
        }
    }

    pub fn read(&self, offset: u64, length: u64) -> Vec<u8> {
        if let Some(ref mmap) = self.mmap_region {
            // Ensure offset and length are valid
            if offset + length > mmap.len() as u64 {
                panic!("Requested region exceeds mapped region!");
            }
            // Return a copied vector from the mmap
            mmap[offset as usize..(offset + length) as usize].to_vec()
        } else {
            panic!("Memory region not mapped!");
        }
    }

    pub fn write(&mut self, write_bytes: &[u8], offset: u64) {
        if let Some(ref mut mmap) = self.mmap_region {
            // Ensure offset and length are valid
            if offset + write_bytes.len() as u64 > mmap.len() as u64 {
                panic!("Requested region exceeds mapped region!");
            }
            // Directly copy the bytes into the mmap region
            mmap[offset as usize..(offset as usize + write_bytes.len())]
                .copy_from_slice(write_bytes);
        } else {
            panic!("Memory region not mapped!");
        }
    }
}

impl Drop for ExpansionBar {
    fn drop(&mut self) {
        // Unlock the file (using a blocking lock to ensure proper unlocking).
        let _ = self.lock_file.unlock();
        self.mmap_file = None;
        self.mmap_region = None;
    }
}

impl fmt::Display for ExpansionBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.phys_bar, self.exp_bar)
    }
}
