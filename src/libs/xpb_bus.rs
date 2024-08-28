#![allow(dead_code)]

use crate::libs::common::split_addr48;
use crate::libs::cpp_bus::{CppBus, CppIsland, CppLength, CppTarget};
use crate::libs::expansion_bar::{ExpansionBar, MapType};
use crate::libs::explicit_bar::ExplicitBar;

pub fn xpb_read(
    exp_bar: &mut ExpansionBar,
    island: &CppIsland,
    address: u32,
    length: u64,
    xpbm: bool,
) -> Vec<u32> {
    // Ensure expansion BAR gets configured with Bulk mapping.
    if exp_bar.exp_bar_map != MapType::Bulk {
        exp_bar.exp_bar_map = MapType::Bulk;
    }

    let mut xpb_addr = address & 0x00FFFFFF;
    let mut tgt_island = *island;
    xpb_addr |= (tgt_island as u32 & 0x7F) << 24;
    if xpbm == true {
        xpb_addr |= 1 << 31; // Set global bit
        tgt_island = CppIsland::ChipExec;
    }

    // Instantiate Cpp bus with allocated expansion BAR.
    let mut cpp_bus = CppBus::new(exp_bar);

    let read_words = cpp_bus.read(
        tgt_island,
        CppTarget::Ct,
        0,
        0,
        CppLength::Len32,
        xpb_addr as u64,
        length,
    );

    read_words
}

pub fn xpb_write(
    exp_bar: &mut ExpansionBar,
    island: &CppIsland,
    address: u32,
    write_words: Vec<u32>,
    xpbm: bool,
) {
    // Ensure expansion BAR gets configured with Bulk mapping.
    if exp_bar.exp_bar_map != MapType::Bulk {
        exp_bar.exp_bar_map = MapType::Bulk;
    }

    let mut xpb_addr = address & 0x00FFFFFF;
    let mut tgt_island = *island;
    xpb_addr |= (tgt_island as u32 & 0x7F) << 24;
    if xpbm == true {
        xpb_addr |= 1 << 31; // Set global bit
        tgt_island = CppIsland::ChipExec;
    }

    // Instantiate Cpp bus with allocated expansion BAR.
    let mut cpp_bus = CppBus::new(exp_bar);

    cpp_bus.write(
        tgt_island,
        CppTarget::Ct,
        0,
        0,
        CppLength::Len32,
        xpb_addr as u64,
        write_words,
    );
}

pub fn xpb_explicit_write32(
    expl_bar: &mut ExplicitBar,
    island: &CppIsland,
    address: u32,
    write_words: Vec<u32>,
    xpbm: bool,
) {
    if address.leading_zeros() < 8 {
        panic!("XPB address {:#08x} is wider than 24 bits.", address);
    }

    let mut xpb_addr = address & 0x00FFFFFF;
    let mut tgt_island = island.id();
    xpb_addr |= (tgt_island as u32 & 0x7F) << 24;
    if xpbm == true {
        xpb_addr |= 1 << 31; // Set global bit
        tgt_island = CppIsland::ChipExec.id();
    }

    let (base_addr, offset) = split_addr48(xpb_addr as u64, expl_bar.size() as u64);

    expl_bar.explicit_bar_cfg(
        tgt_island,
        CppTarget::Ct.id(),
        1,
        0,
        base_addr,
        Some(1),
        0,
        0xFF,
        None,
        None,
        None,
        None,
        None,
    );

    expl_bar.run_explicit_cmd(offset, Some(write_words), None, false);
}

pub fn xpb_explicit_read32(
    expl_bar: &mut ExplicitBar,
    island: &CppIsland,
    address: u32,
    xpbm: bool,
) -> u32 {
    if address.leading_zeros() < 8 {
        panic!("XPB address {:#08x} is wider than 24 bits.", address);
    }

    let mut xpb_addr = address & 0x00FFFFFF;
    let mut tgt_island = island.id();
    xpb_addr |= (tgt_island as u32 & 0x7F) << 24;
    if xpbm == true {
        xpb_addr |= 1 << 31; // Set global bit
        tgt_island = CppIsland::ChipExec.id();
    } else {
        xpb_addr |= (tgt_island as u32 & 0x7F) << 24;
    }

    let (base_addr, offset) = split_addr48(xpb_addr as u64, expl_bar.size() as u64);

    expl_bar.explicit_bar_cfg(
        tgt_island,
        CppTarget::Ct.id(),
        0,
        0,
        base_addr,
        Some(1),
        0,
        0xFF,
        None,
        None,
        None,
        None,
        None,
    );

    if let Some(vec) = expl_bar.run_explicit_cmd(offset, None, Some(1), false) {
        vec[0]
    } else {
        panic!("Failed to read from XPB address");
    }
}
