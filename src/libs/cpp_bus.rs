#![allow(dead_code)]

use bytemuck::cast_slice;
use clap::ValueEnum;
use std::fmt;

use crate::libs::expansion_bar::ExpansionBar;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum CppIsland {
    Local,
    ChipExec,
    Pcie0,
    Pcie1,
    Nbi0,
    Nbi1,
    Nbi2,
    Nbi3,
    Emu0,
    Rfpc0,
    Rfpc1,
    Rfpc2,
    Rfpc3,
    Rfpc4,
    Rfpc5,
    Rfpc6,
}

impl CppIsland {
    pub fn from_id(id: u8) -> CppIsland {
        match id {
            0 => CppIsland::Local,
            1 => CppIsland::ChipExec,
            2 => CppIsland::Pcie0,
            3 => CppIsland::Pcie1,
            4 => CppIsland::Nbi0,
            5 => CppIsland::Nbi1,
            6 => CppIsland::Nbi2,
            7 => CppIsland::Nbi3,
            8 => CppIsland::Emu0,
            9 => CppIsland::Rfpc0,
            10 => CppIsland::Rfpc1,
            11 => CppIsland::Rfpc2,
            12 => CppIsland::Rfpc3,
            13 => CppIsland::Rfpc4,
            14 => CppIsland::Rfpc5,
            15 => CppIsland::Rfpc6,
            _ => panic!("Invalid island ID: {}", id),
        }
    }

    pub fn id(&self) -> u8 {
        match self {
            CppIsland::Local => 0,
            CppIsland::ChipExec => 1,
            CppIsland::Pcie0 => 2,
            CppIsland::Pcie1 => 3,
            CppIsland::Nbi0 => 4,
            CppIsland::Nbi1 => 5,
            CppIsland::Nbi2 => 6,
            CppIsland::Nbi3 => 7,
            CppIsland::Emu0 => 8,
            CppIsland::Rfpc0 => 9,
            CppIsland::Rfpc1 => 10,
            CppIsland::Rfpc2 => 11,
            CppIsland::Rfpc3 => 12,
            CppIsland::Rfpc4 => 13,
            CppIsland::Rfpc5 => 14,
            CppIsland::Rfpc6 => 15,
        }
    }
}

impl fmt::Display for CppIsland {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CppIsland::Local => write!(f, "local"),
            CppIsland::ChipExec => write!(f, "chipExec"),
            CppIsland::Pcie0 => write!(f, "pcie0"),
            CppIsland::Pcie1 => write!(f, "pcie1"),
            CppIsland::Nbi0 => write!(f, "nbi0"),
            CppIsland::Nbi1 => write!(f, "nbi1"),
            CppIsland::Nbi2 => write!(f, "nbi2"),
            CppIsland::Nbi3 => write!(f, "nbi3"),
            CppIsland::Emu0 => write!(f, "emu0"),
            CppIsland::Rfpc0 => write!(f, "rfpc0"),
            CppIsland::Rfpc1 => write!(f, "rfpc1"),
            CppIsland::Rfpc2 => write!(f, "rfpc2"),
            CppIsland::Rfpc3 => write!(f, "rfpc3"),
            CppIsland::Rfpc4 => write!(f, "rfpc4"),
            CppIsland::Rfpc5 => write!(f, "rfpc5"),
            CppIsland::Rfpc6 => write!(f, "rfpc6"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum CppTarget {
    Nbi,
    Mem,
    Pcie,
    Arm,
    Ct,
    Cls,
}

impl CppTarget {
    pub fn id(&self) -> u8 {
        match self {
            CppTarget::Nbi => 1,
            CppTarget::Mem => 7,
            CppTarget::Pcie => 9,
            CppTarget::Arm => 10,
            CppTarget::Ct => 14,
            CppTarget::Cls => 15,
        }
    }
}

impl fmt::Display for CppTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CppTarget::Nbi => write!(f, "nbi"),
            CppTarget::Mem => write!(f, "mem"),
            CppTarget::Pcie => write!(f, "pcie"),
            CppTarget::Arm => write!(f, "arm"),
            CppTarget::Ct => write!(f, "ct"),
            CppTarget::Cls => write!(f, "cls"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum CppLength {
    Len32,
    Len64,
    NoLen,
}

impl CppLength {
    pub fn id(&self) -> u8 {
        match self {
            CppLength::Len32 => 0,
            CppLength::Len64 => 1,
            CppLength::NoLen => 3,
        }
    }

    pub fn get_bits(&self) -> u8 {
        match self {
            CppLength::Len32 => 32,
            CppLength::Len64 => 64,
            CppLength::NoLen => 0,
        }
    }
}

impl fmt::Display for CppLength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CppLength::Len32 => write!(f, "len32"),
            CppLength::Len64 => write!(f, "len64"),
            CppLength::NoLen => write!(f, "noLen"),
        }
    }
}

pub struct CppBus<'a> {
    pub exp_bar: &'a mut ExpansionBar,
}

impl<'a> CppBus<'a> {
    pub fn new(exp_bar: &'a mut ExpansionBar) -> Self {
        CppBus { exp_bar }
    }

    fn configure_exp_bar(
        &mut self,
        island: CppIsland,
        target: CppTarget,
        action: u8,
        token: u8,
        cpp_len: CppLength,
        address: u64,
    ) -> u64 {
        let log2_bar_size = (self.exp_bar.exp_bar_size as f64).log2().floor() as u64;
        let mask = (1u64 << 48) - (1u64 << log2_bar_size);
        self.exp_bar.exp_bar_base_addr = address & mask;
        self.exp_bar.expansion_bar_cfg(
            island.id(),
            target.id(),
            action,
            token,
            self.exp_bar.exp_bar_base_addr,
            cpp_len.id(),
        );
        address - self.exp_bar.exp_bar_base_addr
    }

    pub fn read(
        &mut self,
        island: CppIsland,
        target: CppTarget,
        action: u8,
        token: u8,
        cpp_len: CppLength,
        address: u64,
        length_words: u64,
    ) -> Vec<u32> {
        let offset = self.configure_exp_bar(island, target, action, token, cpp_len, address);
        let length_bytes: u64 = length_words * 4;
        let read_bytes = self.exp_bar.read(offset, length_bytes);
        let read_words_slice: &[u32] = cast_slice(&read_bytes);
        read_words_slice.to_vec()
    }

    pub fn write(
        &mut self,
        island: CppIsland,
        target: CppTarget,
        action: u8,
        token: u8,
        cpp_len: CppLength,
        address: u64,
        write_words: Vec<u32>,
    ) {
        let offset = self.configure_exp_bar(island, target, action, token, cpp_len, address);
        let write_bytes: Vec<u8> = cast_slice(&write_words).to_vec();
        self.exp_bar.write(&write_bytes, offset);
    }
}
