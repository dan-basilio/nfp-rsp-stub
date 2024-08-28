#![allow(dead_code)]
use std::fmt;

use clap::ValueEnum;

use crate::libs::cpp_bus::{CppBus, CppIsland, CppLength, CppTarget};
use crate::libs::expansion_bar::{ExpansionBar, MapType};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum MuMemoryEngine {
    Atomic32,
    Bulk32,
    Bulk64,
}

impl MuMemoryEngine {
    /// Return the CPP (action, token) combinations for CPP read.
    pub fn read_command(&self) -> (u8, u8) {
        match self {
            MuMemoryEngine::Atomic32 => (34, 0),
            MuMemoryEngine::Bulk32 => (28, 0),
            MuMemoryEngine::Bulk64 => (0, 0),
        }
    }

    /// Return the CPP (action, token) combinations for CPP write.
    pub fn write_command(&self) -> (u8, u8) {
        match self {
            MuMemoryEngine::Atomic32 => (4, 0),
            MuMemoryEngine::Bulk32 => (31, 0),
            MuMemoryEngine::Bulk64 => (1, 0),
        }
    }

    /// Return the CPP length enum for the CPP read/write.
    pub fn cpp_length(&self) -> CppLength {
        match self {
            MuMemoryEngine::Atomic32 => CppLength::Len32,
            MuMemoryEngine::Bulk32 => CppLength::Len32,
            MuMemoryEngine::Bulk64 => CppLength::Len64,
        }
    }
}

impl fmt::Display for MuMemoryEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MuMemoryEngine::Atomic32 => write!(f, "Atomic32"),
            MuMemoryEngine::Bulk32 => write!(f, "Bulk32"),
            MuMemoryEngine::Bulk64 => write!(f, "Bulk64"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum MemoryType {
    Emem,
    Ctm,
    Cls,
}

impl MemoryType {
    /// Return the island ID and aliases for the enum variant.
    pub fn locality(&self) -> &'static str {
        match self {
            MemoryType::Emem => "global",
            MemoryType::Ctm => "island",
            MemoryType::Cls => "island",
        }
    }
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryType::Emem => write!(f, "Emem"),
            MemoryType::Ctm => write!(f, "Ctm"),
            MemoryType::Cls => write!(f, "Cls"),
        }
    }
}

pub fn mem_read(
    exp_bar: &mut ExpansionBar,
    cpp_island: CppIsland,
    mem_type: MemoryType,
    engine: MuMemoryEngine,
    address: u64,
    length: u64,
) -> Vec<u32> {
    // Ensure expansion BAR gets configured with Fixed mapping.
    if exp_bar.exp_bar_map != MapType::Fixed {
        exp_bar.exp_bar_map = MapType::Fixed;
    }

    // Instantiate Cpp bus with allocated expansion BAR.
    let mut cpp_bus = CppBus::new(exp_bar);

    match mem_type {
        MemoryType::Emem | MemoryType::Ctm => {
            let (action, token) = engine.read_command();
            cpp_bus.read(
                cpp_island,
                CppTarget::Mem,
                action,
                token,
                engine.cpp_length(),
                address,
                length,
            )
        }
        MemoryType::Cls => cpp_bus.read(
            cpp_island,
            CppTarget::Cls,
            0,
            0,
            CppLength::Len32,
            address,
            length,
        ),
    }
}

pub fn mem_write(
    exp_bar: &mut ExpansionBar,
    cpp_island: CppIsland,
    mem_type: MemoryType,
    engine: MuMemoryEngine,
    address: u64,
    values: Vec<u32>,
) {
    // Ensure expansion BAR gets configured with Fixed mapping.
    if exp_bar.exp_bar_map != MapType::Fixed {
        exp_bar.exp_bar_map = MapType::Fixed;
    }

    // Instantiate Cpp bus with allocated expansion BAR.
    let mut cpp_bus = CppBus::new(exp_bar);

    match mem_type {
        MemoryType::Emem | MemoryType::Ctm => {
            let (action, token) = engine.write_command();
            cpp_bus.write(
                cpp_island,
                CppTarget::Mem,
                action,
                token,
                engine.cpp_length(),
                address,
                values,
            )
        }
        MemoryType::Cls => cpp_bus.write(
            cpp_island,
            CppTarget::Cls,
            1,
            0,
            CppLength::Len32,
            address,
            values,
        ),
    }
}
