#![allow(dead_code)]

use crate::libs::expansion_bar::ExpansionBar;
use crate::libs::rfpc::{Rfpc, RfpcCsr, RfpcReg};
use crate::libs::xpb_bus::{xpb_read, xpb_write};

use std::thread;
use std::time::{Duration, Instant};

/// RISC-V DEBUG MODULE REGISTERS.
/// These are defined by the RISC-V debug standard, specified in section 3.12
/// of the document "RISC-V External Debug Support" version 0.13.2
/// (available from the RISC-V foundation website: https://riscv.org/).
/// Note: The provided address map for the DMI uses 32-bit word addresses,
/// whereas the NFP's XPB uses byte addresses. The DMI addresses are
/// therefore multiplied by 4 here to obtain the XPB addresses.
const RISCV_DBG_DATA0: u32 = 0x10;
const RISCV_DBG_DATA1: u32 = 0x14;
const RISCV_DBG_DATA2: u32 = 0x18;
const RISCV_DBG_DATA3: u32 = 0x1c;
const RISCV_DBG_DATA4: u32 = 0x20;
const RISCV_DBG_DATA5: u32 = 0x24;
const RISCV_DBG_DATA6: u32 = 0x28;
const RISCV_DBG_DATA7: u32 = 0x2c;
const RISCV_DBG_DATA8: u32 = 0x30;
const RISCV_DBG_DATA9: u32 = 0x34;
const RISCV_DBG_DATA10: u32 = 0x38;
const RISCV_DBG_DATA11: u32 = 0x3c;
const RISCV_DBG_DMCONTROL: u32 = 0x40;
const RISCV_DBG_DMSTATUS: u32 = 0x44;
const RISCV_DBG_HARTINFO: u32 = 0x48;
const RISCV_DBG_HALTSUM1: u32 = 0x4c;
const RISCV_DBG_HAWINDOWSEL: u32 = 0x50;
const RISCV_DBG_HAWINDOW: u32 = 0x54;
const RISCV_DBG_ABSTRACTCS: u32 = 0x58;
const RISCV_DBG_COMMAND: u32 = 0x5c;
const RISCV_DBG_ABSTRACTAUTO: u32 = 0x60;
const RISCV_DBG_CONFSTRPTR0: u32 = 0x64;
const RISCV_DBG_CONFSTRPTR1: u32 = 0x68;
const RISCV_DBG_CONFSTRPTR2: u32 = 0x6c;
const RISCV_DBG_CONFSTRPTR3: u32 = 0x70;
const RISCV_DBG_NEXTDM: u32 = 0x74;
const RISCV_DBG_PROGBUF0: u32 = 0x80;
const RISCV_DBG_PROGBUF1: u32 = 0x84;
const RISCV_DBG_PROGBUF2: u32 = 0x88;
const RISCV_DBG_PROGBUF3: u32 = 0x8c;
const RISCV_DBG_PROGBUF4: u32 = 0x90;
const RISCV_DBG_PROGBUF5: u32 = 0x94;
const RISCV_DBG_PROGBUF6: u32 = 0x98;
const RISCV_DBG_PROGBUF7: u32 = 0x9c;
const RISCV_DBG_PROGBUF8: u32 = 0xa0;
const RISCV_DBG_PROGBUF9: u32 = 0xa4;
const RISCV_DBG_PROGBUF10: u32 = 0xa8;
const RISCV_DBG_PROGBUF11: u32 = 0xac;
const RISCV_DBG_PROGBUF12: u32 = 0xb0;
const RISCV_DBG_PROGBUF13: u32 = 0xb4;
const RISCV_DBG_PROGBUF14: u32 = 0xb8;
const RISCV_DBG_PROGBUF15: u32 = 0xbc;
const RISCV_DBG_AUTHDATA: u32 = 0xc0;
const RISCV_DBG_HALTSUM2: u32 = 0xd0;
const RISCV_DBG_HALTSUM3: u32 = 0xd4;
const RISCV_DBG_SBADDRESS3: u32 = 0xdc;
const RISCV_DBG_SBCS: u32 = 0xe0;
const RISCV_DBG_SBADDRESS0: u32 = 0xe4;
const RISCV_DBG_SBADDRESS1: u32 = 0xe8;
const RISCV_DBG_SBADDRESS2: u32 = 0xec;
const RISCV_DBG_SBDATA0: u32 = 0xf0;
const RISCV_DBG_SBDATA1: u32 = 0xf4;
const RISCV_DBG_SBDATA2: u32 = 0xf8;
const RISCV_DBG_SBDATA3: u32 = 0xfc;
const RISCV_DBG_HALTSUM0: u32 = 0x100;

/// RISC-V DEBUG MODULE REGISTER FIELD MASKS.
const RISCV_DBG_DMCONTROL_HALTREQ: u32 = 1 << 31;
const RISCV_DBG_DMCONTROL_RESUMEREQ: u32 = 1 << 30;
const RISCV_DBG_DMCONTROL_HARTRESET: u32 = 1 << 29;
const RISCV_DBG_DMCONTROL_ACKHAVERESET: u32 = 1 << 28;
const RISCV_DBG_DMCONTROL_HASEL: u32 = 1 << 26;
const RISCV_DBG_DMCONTROL_HARTSELLO: u32 = 0x3FF << 16;
const RISCV_DBG_DMCONTROL_HARTSELHI: u32 = 0x3FF << 6;
const RISCV_DBG_DMCONTROL_SETRESETHALTREQ: u32 = 1 << 3;
const RISCV_DBG_DMCONTROL_CLRRESETHALTREQ: u32 = 1 << 2;
const RISCV_DBG_DMCONTROL_NDMRESET: u32 = 1 << 1;
const RISCV_DBG_DMCONTROL_DMACTIVE: u32 = 1 << 0;

const RISCV_DBG_DMSTATUS_IMPEBREAK: u32 = 1 << 22;
const RISCV_DBG_DMSTATUS_ALLHAVERESET: u32 = 1 << 19;
const RISCV_DBG_DMSTATUS_ANYHAVERESET: u32 = 1 << 18;
const RISCV_DBG_DMSTATUS_ALLRESUMEACK: u32 = 1 << 17;
const RISCV_DBG_DMSTATUS_ANYRESUMEACK: u32 = 1 << 16;
const RISCV_DBG_DMSTATUS_ALLNONEXISTENT: u32 = 1 << 15;
const RISCV_DBG_DMSTATUS_ANYNONEXISTENT: u32 = 1 << 14;
const RISCV_DBG_DMSTATUS_ALLUNAVAIL: u32 = 1 << 13;
const RISCV_DBG_DMSTATUS_ANYUNAVAIL: u32 = 1 << 12;
const RISCV_DBG_DMSTATUS_ALLRUNNING: u32 = 1 << 11;
const RISCV_DBG_DMSTATUS_ANYRUNNING: u32 = 1 << 10;
const RISCV_DBG_DMSTATUS_ALLHALTED: u32 = 1 << 9;
const RISCV_DBG_DMSTATUS_ANYHALTED: u32 = 1 << 8;
const RISCV_DBG_DMSTATUS_AUTHENTICATED: u32 = 1 << 7;
const RISCV_DBG_DMSTATUS_AUTHBUSY: u32 = 1 << 6;
const RISCV_DBG_DMSTATUS_HASRESETHALTREQ: u32 = 1 << 5;
const RISCV_DBG_DMSTATUS_CONFSTRPTRVALID: u32 = 1 << 4;
const RISCV_DBG_DMSTATUS_VERSION: u32 = 0xF;

const RISCV_DBG_ABSTRACTCS_PROGBUFSIZE: u32 = 0x1F << 24;
const RISCV_DBG_ABSTRACTCS_BUSY: u32 = 1 << 12;
const RISCV_DBG_ABSTRACTCS_CMDERR: u32 = 0x7 << 8;
const RISCV_DBG_ABSTRACTCS_DATACOUNT: u32 = 0xF;

const RISCV_DBG_DCSR_XDEBUGVER: u32 = 0xF << 28;
const RISCV_DBG_DCSR_EBREAKM: u32 = 0x1 << 15;
const RISCV_DBG_DCSR_EBREAKS: u32 = 0x1 << 13;
const RISCV_DBG_DCSR_EBREAKU: u32 = 0x1 << 12;
const RISCV_DBG_DCSR_STEPIE: u32 = 0x1 << 11;
const RISCV_DBG_DCSR_STOPCOUNT: u32 = 0x1 << 10;
const RISCV_DBG_DCSR_STOPTIME: u32 = 0x1 << 9;
const RISCV_DBG_DCSR_CAUSE: u32 = 0x7 << 6;
const RISCV_DBG_DCSR_MPRVEN: u32 = 0x1 << 4;
const RISCV_DBG_DCSR_NMIP: u32 = 0x1 << 3;
const RISCV_DBG_DCSR_STEP: u32 = 0x1 << 2;
const RISCV_DBG_DCSR_PRV: u32 = 0x3 << 0;

pub fn read_rfpc_reg(exp_bar: &mut ExpansionBar, rfpc: &Rfpc, reg: &Box<dyn RfpcReg>) -> u64 {
    let reg_addr = reg.reg_addr();

    rfpc_dbg_halt(exp_bar, rfpc);
    let val = rfpc_dbg_read_reg(exp_bar, rfpc, reg_addr);
    rfpc_dbg_resume(exp_bar, rfpc);

    val
}

pub fn write_rfpc_reg(exp_bar: &mut ExpansionBar, rfpc: &Rfpc, reg: &Box<dyn RfpcReg>, value: u64) {
    let reg_addr = reg.reg_addr();

    rfpc_dbg_halt(exp_bar, rfpc);
    rfpc_dbg_write_reg(exp_bar, rfpc, reg_addr, value);
    rfpc_dbg_resume(exp_bar, rfpc);
}

pub fn rfpc_dbg_halt(exp_bar: &mut ExpansionBar, rfpc: &Rfpc) {
    let (hartsello, _) = rfpc.dm_hartsel();
    let mut dmcontrol = hartsello << 16;

    dmcontrol |= RISCV_DBG_DMCONTROL_DMACTIVE;
    dmcontrol |= RISCV_DBG_DMCONTROL_HALTREQ;

    // Write halt request to dmcontrol to initiate halt.
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DMCONTROL,
        vec![dmcontrol],
        true,
    );

    // Poll dmstatus until RFPC is halted.
    let start_time = Instant::now();
    let timeout_duration = Duration::new(10, 0);
    loop {
        if start_time.elapsed() > timeout_duration {
            panic!("Timeout reached when waiting for RFPC core to halt after halt initiate!");
        }

        let dmstatus = xpb_read(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DMSTATUS,
            1,
            true,
        )[0];
        if dmstatus & RISCV_DBG_DMSTATUS_ALLHALTED != 0 {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

pub fn rfpc_dbg_resume(exp_bar: &mut ExpansionBar, rfpc: &Rfpc) {
    let (hartsello, _) = rfpc.dm_hartsel();
    let mut dmcontrol = hartsello << 16;

    dmcontrol |= RISCV_DBG_DMCONTROL_DMACTIVE;
    dmcontrol |= RISCV_DBG_DMCONTROL_RESUMEREQ;

    // Write resume request to dmcontrol to initiate resume.
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DMCONTROL,
        vec![dmcontrol],
        true,
    );

    // Poll dmstatus until RFPC has resumed.
    let start_time = Instant::now();
    let timeout_duration = Duration::new(10, 0);
    loop {
        if start_time.elapsed() > timeout_duration {
            panic!("Timeout reached when trying to resume RFPC core after resume initiate!");
        }
        let dmstatus = xpb_read(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DMSTATUS,
            1,
            true,
        )[0];
        if dmstatus & RISCV_DBG_DMSTATUS_ALLRUNNING != 0 {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

pub fn rfpc_dbg_single_step(exp_bar: &mut ExpansionBar, rfpc: &Rfpc) {
    let mut dcsr_reg = rfpc_dbg_read_reg(exp_bar, rfpc, RfpcCsr::Dcsr.reg_addr());
    dcsr_reg |= RISCV_DBG_DCSR_STEP as u64;
    rfpc_dbg_write_reg(exp_bar, rfpc, RfpcCsr::Dcsr.reg_addr(), dcsr_reg);

    // Write resume request to dmcontrol to initiate resume.
    let (hartsello, _) = rfpc.dm_hartsel();
    let mut dmcontrol = hartsello << 16;
    dmcontrol |= RISCV_DBG_DMCONTROL_DMACTIVE;
    dmcontrol |= RISCV_DBG_DMCONTROL_RESUMEREQ;
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DMCONTROL,
        vec![dmcontrol],
        true,
    );

    // Poll dmstatus until RFPC is halted.
    let start_time = Instant::now();
    let timeout_duration = Duration::new(10, 0);
    loop {
        if start_time.elapsed() > timeout_duration {
            panic!("Timeout reached when wating for RFPC core halt after step!");
        }

        let dmstatus = xpb_read(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DMSTATUS,
            1,
            true,
        )[0];
        if dmstatus & RISCV_DBG_DMSTATUS_ALLHALTED != 0 {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    let mut dcsr_reg = rfpc_dbg_read_reg(exp_bar, rfpc, RfpcCsr::Dcsr.reg_addr());
    let cause = (dcsr_reg as u32 & RISCV_DBG_DCSR_CAUSE) >> 6;
    if cause != 0x4 {
        panic!("The RFPC core did not single step!");
    }
    dcsr_reg &= !RISCV_DBG_DCSR_STEP as u64;
    rfpc_dbg_write_reg(exp_bar, rfpc, RfpcCsr::Dcsr.reg_addr(), dcsr_reg);
}

pub fn rfpc_dbg_continue(exp_bar: &mut ExpansionBar, rfpc: &Rfpc) {
    let mut dcsr_reg = rfpc_dbg_read_reg(exp_bar, rfpc, RfpcCsr::Dcsr.reg_addr());
    dcsr_reg |= (RISCV_DBG_DCSR_EBREAKM | RISCV_DBG_DCSR_EBREAKU) as u64;
    rfpc_dbg_write_reg(exp_bar, rfpc, RfpcCsr::Dcsr.reg_addr(), dcsr_reg);

    // Write resume request to dmcontrol to initiate resume.
    let (hartsello, _) = rfpc.dm_hartsel();
    let mut dmcontrol = hartsello << 16;
    dmcontrol |= RISCV_DBG_DMCONTROL_DMACTIVE;
    dmcontrol |= RISCV_DBG_DMCONTROL_RESUMEREQ;
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DMCONTROL,
        vec![dmcontrol],
        true,
    );

    // Poll dmstatus until RFPC is halted.
    let start_time = Instant::now();
    let timeout_duration = Duration::new(40, 0);
    loop {
        if start_time.elapsed() > timeout_duration {
            panic!("Timeout reached when wating for RFPC core halt after step!");
        }

        let dmstatus = xpb_read(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DMSTATUS,
            1,
            true,
        )[0];
        if dmstatus & RISCV_DBG_DMSTATUS_ALLHALTED != 0 {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    let mut dcsr_reg = rfpc_dbg_read_reg(exp_bar, rfpc, RfpcCsr::Dcsr.reg_addr());
    let cause = (dcsr_reg as u32 & RISCV_DBG_DCSR_CAUSE) >> 6;
    if cause != 0x1 {
        panic!("The RFPC core did not breakpoint, cause = 0x{:x}!", cause);
    }
    dcsr_reg &= !(RISCV_DBG_DCSR_EBREAKM | RISCV_DBG_DCSR_EBREAKU) as u64;
    rfpc_dbg_write_reg(exp_bar, rfpc, RfpcCsr::Dcsr.reg_addr(), dcsr_reg);
}

fn abstract_cmd_busy_wait(exp_bar: &mut ExpansionBar, rfpc: &Rfpc) {
    let mut abstractcs: u32;
    let start_time = Instant::now();
    let timeout_duration = Duration::new(10, 0);
    loop {
        if start_time.elapsed() > timeout_duration {
            panic!("Timeout reached in rfpc_dbg_abstractcmd()!");
        }
        abstractcs = xpb_read(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_ABSTRACTCS,
            1,
            true,
        )[0];
        if (abstractcs & RISCV_DBG_ABSTRACTCS_BUSY) == 0 {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

pub fn rfpc_dbg_read_reg(exp_bar: &mut ExpansionBar, rfpc: &Rfpc, reg_addr: u64) -> u64 {
    let (hartsello, _) = rfpc.dm_hartsel();
    let mut dmcontrol = hartsello << 16;

    dmcontrol |= RISCV_DBG_DMCONTROL_DMACTIVE;
    // Write dmcontrol.
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DMCONTROL,
        vec![dmcontrol as u32],
        true,
    );

    let command = 0x320000 | (reg_addr & 0xFFFF);
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_COMMAND,
        vec![command as u32],
        true,
    );

    abstract_cmd_busy_wait(exp_bar, rfpc);

    // Read the lower 32 bits of the register value.
    let mut reg_val: u64 = xpb_read(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DATA0,
        1,
        true,
    )[0] as u64;

    // Read the upper 32 bits of the register value.
    reg_val |= (xpb_read(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DATA1,
        1,
        true,
    )[0] as u64)
        << 32;

    reg_val
}

pub fn rfpc_dbg_write_reg(exp_bar: &mut ExpansionBar, rfpc: &Rfpc, reg_addr: u64, value: u64) {
    let reg_gpr: bool = ((reg_addr >> 12) & 0xF) == 0x1;
    let (hartsello, _) = rfpc.dm_hartsel();
    let mut dmcontrol = hartsello << 16;

    dmcontrol |= RISCV_DBG_DMCONTROL_DMACTIVE;
    // Write dmcontrol.
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DMCONTROL,
        vec![dmcontrol],
        true,
    );

    // Write lower 32 bits of register value to debug module data0.
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DATA0,
        vec![value as u32 & 0xFFFFFFFF],
        true,
    );

    // Write upper 32 bits of register value to debug module data1.
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DATA1,
        vec![(value >> 32) as u32 & 0xFFFFFFFF],
        true,
    );

    if reg_gpr {
        // Execute ABSTRACT CMD (write values to GPR register specified).
        let gpr = 0x330000 | (reg_addr as u32);
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_COMMAND,
            vec![gpr],
            true,
        );
        abstract_cmd_busy_wait(exp_bar, rfpc);
        return;
    } else {
        // Execute ABSTRACT CMD (write values in DATA0 and DATA1 to X11 for CSR write).
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_COMMAND,
            vec![0x33100B],
            true,
        );
    }

    abstract_cmd_busy_wait(exp_bar, rfpc);

    // Write csrw instruction to progbuf0.
    let csr_write_instr: u32 = 0x00059073 | ((reg_addr as u32 & 0xFFF) << 20);
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_PROGBUF0,
        vec![csr_write_instr],
        true,
    );

    // Execute ABSTRACT CMD (execute progbuf0).
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_COMMAND,
        vec![0x360000],
        true,
    );

    abstract_cmd_busy_wait(exp_bar, rfpc);
}

pub fn rfpc_dbg_read_memory(
    exp_bar: &mut ExpansionBar,
    rfpc: &Rfpc,
    address: u64,
    length: u64,
) -> Vec<u64> {
    // Write dmcontrol.
    let (hartsello, _) = rfpc.dm_hartsel();
    let dmcontrol = (hartsello << 16) | RISCV_DBG_DMCONTROL_DMACTIVE;
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DMCONTROL,
        vec![dmcontrol],
        true,
    );

    // Save RFPC GPR a0 (X10) temporarily, as it will be overwritten for
    // the memory read process.
    let temp_a0 = rfpc_dbg_read_reg(exp_bar, rfpc, 0x100A);

    // Read from memory one 64-bit word at a time.
    let mut mem_words: Vec<u64> = Vec::new();
    for word_idx in 0..length {
        let byte_addr = address + 8 * word_idx;
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DATA0,
            vec![byte_addr as u32 & 0xFFFFFFFF],
            true,
        );
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DATA1,
            vec![(byte_addr >> 32) as u32 & 0xFFFFFFFF],
            true,
        );
        // Write load memory instruction to debug module progbuf0 register.
        // 0x53503 => `ld a0, (0)a0`  (load double word from mem[a0]).
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_PROGBUF0,
            vec![0x53503],
            true,
        );
        // Execute abstract command: load ((data1 << 32) | data0) into RFPC
        // GPR a0 before executing the instruction in the program buffer.
        // This reads the 64-bit word in memory at word_addr into GPR a0.
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_COMMAND,
            vec![0x37100A],
            true,
        );
        abstract_cmd_busy_wait(exp_bar, rfpc);

        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_COMMAND,
            vec![0x32100A],
            true,
        );
        abstract_cmd_busy_wait(exp_bar, rfpc);

        // Read the lower 32 bits of the register value.
        let mut reg_val: u64 = xpb_read(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DATA0,
            1,
            true,
        )[0] as u64;

        // Read the upper 32 bits of the register value.
        reg_val |= (xpb_read(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DATA1,
            1,
            true,
        )[0] as u64)
            << 32;

        // Read memory word and push to the vector.
        mem_words.push(reg_val);
    }

    // Restore RFPC GPR a0.
    rfpc_dbg_write_reg(exp_bar, rfpc, 0x100A, temp_a0);

    mem_words
}

pub fn rfpc_dbg_write_memory(
    exp_bar: &mut ExpansionBar,
    rfpc: &Rfpc,
    address: u64,
    data: Vec<u64>,
) {
    // Write dmcontrol.
    let (hartsello, _) = rfpc.dm_hartsel();
    let dmcontrol = (hartsello << 16) | RISCV_DBG_DMCONTROL_DMACTIVE;
    xpb_write(
        exp_bar,
        &rfpc.island,
        rfpc.dm_xpb_base() + RISCV_DBG_DMCONTROL,
        vec![dmcontrol],
        true,
    );

    // Save RFPC GPRs a0 and a1 temporarily.
    let temp_a0 = rfpc_dbg_read_reg(exp_bar, rfpc, 0x100A);
    let temp_a1 = rfpc_dbg_read_reg(exp_bar, rfpc, 0x100B);

    for (word_idx, data_word) in data.iter().enumerate() {
        let byte_addr = address + (8u64 * word_idx as u64);

        // Write data word to debug module data0/1 registers.
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DATA0,
            vec![*data_word as u32 & 0xFFFFFFFF],
            true,
        );
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DATA1,
            vec![(*data_word >> 32) as u32 & 0xFFFFFFFF],
            true,
        );

        // Execute abstract command to write data word to RFPC GPR a1.
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_COMMAND,
            vec![0x33100B],
            true,
        );
        abstract_cmd_busy_wait(exp_bar, rfpc);

        // Write 64-bit word address to debug module data0/1 registers.
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DATA0,
            vec![byte_addr as u32 & 0xFFFFFFFF],
            true,
        );
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_DATA1,
            vec![(byte_addr >> 32) as u32 & 0xFFFFFFFF],
            true,
        );

        // Write instruction to debug module progbuf0 register.
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_PROGBUF0,
            vec![0xB53023],
            true,
        );

        // Execute abstract command to write data word to RFPC GPR a1.
        xpb_write(
            exp_bar,
            &rfpc.island,
            rfpc.dm_xpb_base() + RISCV_DBG_COMMAND,
            vec![0x37100A],
            true,
        );
        abstract_cmd_busy_wait(exp_bar, rfpc);
    }

    // Restore RFPC GPRs a0 and a1.
    rfpc_dbg_write_reg(exp_bar, rfpc, 0x100A, temp_a0);
    rfpc_dbg_write_reg(exp_bar, rfpc, 0x100B, temp_a1);
}
