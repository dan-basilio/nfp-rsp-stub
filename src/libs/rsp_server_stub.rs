#![allow(dead_code)]

use crate::libs::cpp_bus::CppIsland;
use crate::libs::expansion_bar::ExpansionBar;
use crate::libs::mem_access::{mem_read, mem_write, MemoryType, MuMemoryEngine};
use crate::libs::rfpc::{Rfpc, RfpcCsr, RfpcGpr, RfpcReg};
use crate::libs::rfpc_debugger::{
    rfpc_dbg_continue, rfpc_dbg_read_memory, rfpc_dbg_read_reg, rfpc_dbg_single_step,
    rfpc_dbg_write_memory, rfpc_dbg_write_reg,
};
use bytemuck::cast_slice;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

const LOCAL_HOST_IP: &str = "127.0.0.1";
const PORT: u16 = 12727;

// Define the function type enum.
#[derive(Clone)]
enum FuncType<'a> {
    Ascii(String),
    NoArg(fn(&mut RspServer<'a>) -> String),
    WithArg(fn(&mut RspServer<'a>, Vec<u8>) -> String),
}

pub struct RspServer<'a> {
    exp_bar: &'a mut ExpansionBar,
    cmd_resp_map: HashMap<String, Option<FuncType<'a>>>,
    server_kv_support: HashMap<String, String>,
    server_v_support: Vec<String>,
    client_kv_support: HashMap<String, String>,
    client_v_support: Vec<String>,
    breakpoints: HashMap<u64, u64>,
    disable_ack: bool,
    rfpc: Rfpc,
}

impl<'a> RspServer<'a> {
    /// Creates a new instance of the `RspServer`.
    ///
    /// # Parameters
    ///
    /// * `exp_bar - A mutable reference to an `ExpansionBar`.
    ///
    /// # Returns
    ///
    /// `RspServer` instance.
    pub fn new(
        exp_bar: &'a mut ExpansionBar,
        island: CppIsland,
        cluster: u8,
        group: u8,
        core: u8,
    ) -> Self {
        let mut cmd_resp_map: HashMap<String, Option<FuncType>> = HashMap::new();
        cmd_resp_map.insert(
            "!".to_string(),
            Some(FuncType::NoArg(RspServer::cmd_not_supported)),
        );
        cmd_resp_map.insert(
            "?".to_string(),
            Some(FuncType::Ascii(format!("S{:02x}", 18))),
        );
        cmd_resp_map.insert("c".to_string(), None);
        cmd_resp_map.insert("D".to_string(), None);
        cmd_resp_map.insert(
            "QStartNoAckMode".to_string(),
            Some(FuncType::NoArg(RspServer::toggle_ack)),
        );
        cmd_resp_map.insert("qC".to_string(), Some(FuncType::Ascii("-1".to_string())));
        cmd_resp_map.insert(
            "qOffsets".to_string(),
            Some(FuncType::NoArg(RspServer::load_offsets)),
        );
        cmd_resp_map.insert(
            "qSupported".to_string(),
            Some(FuncType::WithArg(RspServer::supported_features)),
        );
        cmd_resp_map.insert(
            "qAttached".to_string(),
            Some(FuncType::Ascii("1".to_string())),
        );
        cmd_resp_map.insert("H".to_string(), Some(FuncType::Ascii("l".to_string())));
        cmd_resp_map.insert("g".to_string(), Some(FuncType::NoArg(RspServer::read_gprs)));
        cmd_resp_map.insert(
            "p".to_string(),
            Some(FuncType::WithArg(RspServer::read_reg)),
        );
        cmd_resp_map.insert(
            "P".to_string(),
            Some(FuncType::WithArg(RspServer::write_reg)),
        );
        cmd_resp_map.insert(
            "m".to_string(),
            Some(FuncType::WithArg(RspServer::memory_read)),
        );
        cmd_resp_map.insert(
            "M".to_string(),
            Some(FuncType::WithArg(RspServer::memory_write)),
        );
        cmd_resp_map.insert(
            "H".to_string(),
            Some(FuncType::WithArg(RspServer::set_core)),
        );
        cmd_resp_map.insert(
            "s".to_string(),
            Some(FuncType::WithArg(RspServer::single_step)),
        );
        cmd_resp_map.insert(
            "S".to_string(),
            Some(FuncType::NoArg(RspServer::single_step_sig)),
        );
        cmd_resp_map.insert("c".to_string(), Some(FuncType::WithArg(RspServer::cont)));
        cmd_resp_map.insert("D".to_string(), Some(FuncType::Ascii("detach".to_string())));
        cmd_resp_map.insert(
            "Z0".to_string(),
            Some(FuncType::WithArg(RspServer::set_breakpoint)),
        );
        cmd_resp_map.insert(
            "z0".to_string(),
            Some(FuncType::WithArg(RspServer::clear_breakpoint)),
        );
        cmd_resp_map.insert("\x03".to_string(), None);
        cmd_resp_map.insert("k".to_string(), None);
        cmd_resp_map.insert(
            "C".to_string(),
            Some(FuncType::WithArg(RspServer::cont_with_sig)),
        );
        cmd_resp_map.insert(
            "vMustReplyEmpty".to_string(),
            Some(FuncType::NoArg(RspServer::cmd_not_supported)),
        );
        cmd_resp_map.insert(
            "X".to_string(),
            Some(FuncType::WithArg(RspServer::memory_write)),
        );

        // Server key->value and value support.
        let mut server_v_support: Vec<String> = Vec::new();
        server_v_support.push("qMemoryRead+".to_string());
        server_v_support.push("swbreak+".to_string());
        let mut server_kv_support: HashMap<String, String> = HashMap::new();
        server_kv_support.insert("PacketSize".to_string(), "100000".to_string());

        // Client key->value and value support.
        let client_kv_support: HashMap<String, String> = HashMap::new();
        let client_v_support: Vec<String> = Vec::new();

        // Keep +/- ACK om until the client disables it
        let disable_ack = false;

        // Initialize breakpoint hash map.
        let breakpoints: HashMap<u64, u64> = HashMap::new();

        // Initialize to Rfpc island, cluster, group and core.
        let rfpc = Rfpc {
            island,
            cluster,
            group,
            core,
        };

        // Return the server struct.
        RspServer {
            exp_bar,
            cmd_resp_map,
            server_kv_support,
            server_v_support,
            client_kv_support,
            client_v_support,
            breakpoints,
            disable_ack,
            rfpc,
        }
    }

    /// Method that returns an empty string if the RSP command is not
    /// supported.
    ///
    /// # Returns
    ///
    /// Empty string.
    fn cmd_not_supported(&mut self) -> String {
        "".to_string()
    }

    /// Returns a concatenated string of the all the GPR register values
    /// in hex.
    ///
    /// # Returns
    ///
    /// Concatenated list of GPR values.
    fn read_gprs(&mut self) -> String {
        let mut gprs = String::new();

        // Iterate over GPR addresses from X0 to X31
        for reg in RfpcGpr::X0.reg_addr()..=RfpcGpr::X31.reg_addr() {
            let reg_val = rfpc_dbg_read_reg(self.exp_bar, &self.rfpc, reg);
            gprs.push_str(&format!("{:016x}", reg_val.swap_bytes()));
        }

        // Iterate over CSR addresses
        for reg in RfpcCsr::Mstatus.reg_addr()..=RfpcCsr::Mtvec.reg_addr() {
            let reg_val = rfpc_dbg_read_reg(self.exp_bar, &self.rfpc, reg);
            gprs.push_str(&format!("{:016x}", reg_val.swap_bytes()));
        }

        for reg in RfpcCsr::Mscratch.reg_addr()..=RfpcCsr::Mip.reg_addr() {
            let reg_val = rfpc_dbg_read_reg(self.exp_bar, &self.rfpc, reg);
            gprs.push_str(&format!("{:016x}", reg_val.swap_bytes()));
        }

        for reg in RfpcCsr::Dcsr.reg_addr()..=RfpcCsr::Dscratch1.reg_addr() {
            let reg_val = rfpc_dbg_read_reg(self.exp_bar, &self.rfpc, reg);
            gprs.push_str(&format!("{:016x}", reg_val.swap_bytes()));
        }

        for reg in RfpcCsr::Mlmemprot.reg_addr()..=RfpcCsr::Mafstatus.reg_addr() {
            let reg_val = rfpc_dbg_read_reg(self.exp_bar, &self.rfpc, reg);
            gprs.push_str(&format!("{:016x}", reg_val.swap_bytes()));
        }

        let reg_val = rfpc_dbg_read_reg(self.exp_bar, &self.rfpc, RfpcCsr::Mcycle.reg_addr());
        gprs.push_str(&format!("{:016x}", reg_val.swap_bytes()));
        let reg_val = rfpc_dbg_read_reg(self.exp_bar, &self.rfpc, RfpcCsr::Minstret.reg_addr());
        gprs.push_str(&format!("{:016x}", reg_val.swap_bytes()));

        for reg in RfpcCsr::Cycle.reg_addr()..=RfpcCsr::Instret.reg_addr() {
            let reg_val = rfpc_dbg_read_reg(self.exp_bar, &self.rfpc, reg);
            gprs.push_str(&format!("{:016x}", reg_val.swap_bytes()));
        }

        for reg in RfpcCsr::Mvendorid.reg_addr()..=RfpcCsr::Mhartid.reg_addr() {
            let reg_val = rfpc_dbg_read_reg(self.exp_bar, &self.rfpc, reg);
            gprs.push_str(&format!("{:016x}", reg_val.swap_bytes()));
        }

        gprs
    }

    /// Receives a concatenated string of RISC-V register values and programs each
    /// of them.
    ///
    /// # Parameters
    ///
    /// * `packet - RSP packet after being parsed.
    ///
    /// # Returns
    ///
    /// "OK" if operation succeeded
    fn write_gprs(&mut self, packet: Vec<u8>) -> String {
        let gpr_count = 32; // Number of GPR registers (X0 to X31)
        let nybble_length = 16;

        // Write the GPR registers.
        for reg_idx in 0..gpr_count {
            let start_idx = reg_idx * nybble_length;
            let reg_value_bytes = &packet[start_idx..start_idx + nybble_length];

            // Convert the byte slice to a string of hex characters
            let reg_value_str = String::from_utf8_lossy(reg_value_bytes);
            let reg_value = u64::from_str_radix(&reg_value_str, 16)
                .expect("Failed to parse nybble string as u64");

            rfpc_dbg_write_reg(
                self.exp_bar,
                &self.rfpc,
                RfpcGpr::X0.reg_addr() + reg_idx as u64,
                reg_value.swap_bytes(),
            );
        }

        // Define CSR register mapping
        let csr_map: HashMap<usize, RfpcCsr> = [
            (32, RfpcCsr::Mstatus),
            (33, RfpcCsr::Misa),
            (34, RfpcCsr::Medeleg),
            (35, RfpcCsr::Mideleg),
            (36, RfpcCsr::Mie),
            (37, RfpcCsr::Mtvec),
            (38, RfpcCsr::Mscratch),
            (39, RfpcCsr::Mepc),
            (40, RfpcCsr::Mcause),
            (41, RfpcCsr::Mtval),
            (42, RfpcCsr::Mip),
            (43, RfpcCsr::Dcsr),
            (44, RfpcCsr::Dpc),
            (45, RfpcCsr::Dscratch0),
            (46, RfpcCsr::Dscratch1),
            (47, RfpcCsr::Mlmemprot),
            (48, RfpcCsr::Mafstatus),
            (49, RfpcCsr::Mcycle),
            (50, RfpcCsr::Minstret),
            (51, RfpcCsr::Cycle),
            (52, RfpcCsr::Time),
            (53, RfpcCsr::Instret),
            (54, RfpcCsr::Mvendorid),
            (55, RfpcCsr::Marchid),
            (56, RfpcCsr::Mimpid),
            (57, RfpcCsr::Mhartid),
        ]
        .iter()
        .cloned()
        .collect();

        // Write CSR values
        for reg_idx in 32..32 + csr_map.len() {
            let start_idx = reg_idx * nybble_length;
            let reg_value_bytes = &packet[start_idx..start_idx + nybble_length];
            let reg_value_str = String::from_utf8_lossy(reg_value_bytes);
            let reg_value = u64::from_str_radix(&reg_value_str, 16)
                .expect("Failed to parse nybble string as u64");

            if let Some(csr) = csr_map.get(&reg_idx) {
                rfpc_dbg_write_reg(
                    self.exp_bar,
                    &self.rfpc,
                    csr.reg_addr() as u64,
                    reg_value.swap_bytes(),
                );
            }
        }

        "OK".to_string()
    }

    /// Read a register from the register map of the core.
    ///
    /// # Parameters
    ///
    /// * `packet - RSP packet after being parsed.
    ///
    /// # Returns
    ///
    /// Concatenated list of register values.
    fn read_reg(&mut self, packet: Vec<u8>) -> String {
        // Attempt to parse the address as an integer.
        let address_str = String::from_utf8_lossy(&packet[1..]);
        let address =
            u64::from_str_radix(&address_str, 16).expect("Failed to parse address as u64");

        // Create an array of GPR addresses.
        let gpr_regs = [
            RfpcGpr::X0,
            RfpcGpr::X1,
            RfpcGpr::X2,
            RfpcGpr::X3,
            RfpcGpr::X4,
            RfpcGpr::X5,
            RfpcGpr::X6,
            RfpcGpr::X7,
            RfpcGpr::X8,
            RfpcGpr::X9,
            RfpcGpr::X10,
            RfpcGpr::X11,
            RfpcGpr::X12,
            RfpcGpr::X13,
            RfpcGpr::X14,
            RfpcGpr::X15,
            RfpcGpr::X16,
            RfpcGpr::X17,
            RfpcGpr::X18,
            RfpcGpr::X19,
            RfpcGpr::X20,
            RfpcGpr::X21,
            RfpcGpr::X22,
            RfpcGpr::X23,
            RfpcGpr::X24,
            RfpcGpr::X25,
            RfpcGpr::X26,
            RfpcGpr::X27,
            RfpcGpr::X28,
            RfpcGpr::X29,
            RfpcGpr::X30,
            RfpcGpr::X31,
        ];

        // Create an array of CSR addresses.
        let csr_regs = [
            RfpcCsr::Mstatus,
            RfpcCsr::Misa,
            RfpcCsr::Medeleg,
            RfpcCsr::Mideleg,
            RfpcCsr::Mie,
            RfpcCsr::Mtvec,
            RfpcCsr::Mscratch,
            RfpcCsr::Mepc,
            RfpcCsr::Mcause,
            RfpcCsr::Mtval,
            RfpcCsr::Mip,
            RfpcCsr::Dcsr,
            RfpcCsr::Dpc,
            RfpcCsr::Dscratch0,
            RfpcCsr::Dscratch1,
            RfpcCsr::Mlmemprot,
            RfpcCsr::Mafstatus,
            RfpcCsr::Mcycle,
            RfpcCsr::Minstret,
            RfpcCsr::Cycle,
            RfpcCsr::Time,
            RfpcCsr::Instret,
            RfpcCsr::Mvendorid,
            RfpcCsr::Marchid,
            RfpcCsr::Mimpid,
            RfpcCsr::Mhartid,
        ];

        // Declare `reg_val` outside the conditional blocks.
        let reg_val = if (0..32).contains(&address) {
            rfpc_dbg_read_reg(
                self.exp_bar,
                &self.rfpc,
                gpr_regs[address as usize].reg_addr(),
            )
        } else if (32..(32 + csr_regs.len() as u64)).contains(&address) {
            rfpc_dbg_read_reg(
                self.exp_bar,
                &self.rfpc,
                csr_regs[(address - 32) as usize].reg_addr(),
            )
        } else {
            panic!("Invalid register address");
        };

        // Format the register value and return as a hex string.
        format!("{:016x}", reg_val.swap_bytes())
    }

    /// Write to a register in the register map of the core.
    ///
    /// # Parameters
    ///
    /// * `packet - RSP packet after being parsed.
    ///
    /// # Returns
    ///
    /// "OK" if operation succeeded
    fn write_reg(&mut self, packet: Vec<u8>) -> String {
        // Find the position of the '='.
        let equals_index = packet.iter().position(|&b| b == b'=').unwrap();

        // Extract the register address and value from the packet.
        let reg_addr = String::from_utf8_lossy(&packet[1..equals_index]);
        let reg_val = String::from_utf8_lossy(&packet[equals_index + 1..]);
        let address = u64::from_str_radix(&reg_addr, 16).expect("Failed to parse address as u8");
        let value = u64::from_str_radix(&reg_val, 16)
            .expect("Failed to parse value as u64")
            .swap_bytes();

        // Create an array of GPR addresses.
        let gpr_regs = [
            RfpcGpr::X0,
            RfpcGpr::X1,
            RfpcGpr::X2,
            RfpcGpr::X3,
            RfpcGpr::X4,
            RfpcGpr::X5,
            RfpcGpr::X6,
            RfpcGpr::X7,
            RfpcGpr::X8,
            RfpcGpr::X9,
            RfpcGpr::X10,
            RfpcGpr::X11,
            RfpcGpr::X12,
            RfpcGpr::X13,
            RfpcGpr::X14,
            RfpcGpr::X15,
            RfpcGpr::X16,
            RfpcGpr::X17,
            RfpcGpr::X18,
            RfpcGpr::X19,
            RfpcGpr::X20,
            RfpcGpr::X21,
            RfpcGpr::X22,
            RfpcGpr::X23,
            RfpcGpr::X24,
            RfpcGpr::X25,
            RfpcGpr::X26,
            RfpcGpr::X27,
            RfpcGpr::X28,
            RfpcGpr::X29,
            RfpcGpr::X30,
            RfpcGpr::X31,
        ];

        // Create an array of CSR addresses.
        let csr_regs = [
            RfpcCsr::Mstatus,
            RfpcCsr::Misa,
            RfpcCsr::Medeleg,
            RfpcCsr::Mideleg,
            RfpcCsr::Mie,
            RfpcCsr::Mtvec,
            RfpcCsr::Mscratch,
            RfpcCsr::Mepc,
            RfpcCsr::Mcause,
            RfpcCsr::Mtval,
            RfpcCsr::Mip,
            RfpcCsr::Dcsr,
            RfpcCsr::Dpc,
            RfpcCsr::Dscratch0,
            RfpcCsr::Dscratch1,
            RfpcCsr::Mlmemprot,
            RfpcCsr::Mafstatus,
            RfpcCsr::Mcycle,
            RfpcCsr::Minstret,
            RfpcCsr::Cycle,
            RfpcCsr::Time,
            RfpcCsr::Instret,
            RfpcCsr::Mvendorid,
            RfpcCsr::Marchid,
            RfpcCsr::Mimpid,
            RfpcCsr::Mhartid,
        ];

        if (0..32).contains(&address) {
            // Write to the GPR register.
            rfpc_dbg_write_reg(
                self.exp_bar,
                &self.rfpc,
                gpr_regs[address as usize].reg_addr(),
                value,
            );
        } else if (32..(32 + csr_regs.len() as u64)).contains(&address) {
            // Write to the CSR register.
            rfpc_dbg_write_reg(
                self.exp_bar,
                &self.rfpc,
                csr_regs[(address - 32) as usize].reg_addr(),
                value,
            );
        } else {
            panic!("Invalid register address");
        };

        "OK".to_string()
    }

    fn set_core(&mut self, _packet: Vec<u8>) -> String {
        "OK".to_string()
    }

    fn single_step(&mut self, packet: Vec<u8>) -> String {
        if packet.len() > 1 {
            let address_str = String::from_utf8_lossy(&packet[1..]);
            let address =
                u64::from_str_radix(&address_str, 16).expect("Failed to parse address as u64");
            rfpc_dbg_write_reg(self.exp_bar, &self.rfpc, RfpcCsr::Dpc.reg_addr(), address);
        }

        rfpc_dbg_single_step(self.exp_bar, &self.rfpc);
        "S05".to_string()
    }

    fn single_step_sig(&mut self) -> String {
        rfpc_dbg_single_step(self.exp_bar, &self.rfpc);
        "S05".to_string()
    }

    fn cont(&mut self, packet: Vec<u8>) -> String {
        if packet.len() > 1 {
            let address_str = String::from_utf8_lossy(&packet[1..]);
            let address =
                u64::from_str_radix(&address_str, 16).expect("Failed to parse address as u64");
            rfpc_dbg_write_reg(self.exp_bar, &self.rfpc, RfpcCsr::Dpc.reg_addr(), address);
        }

        rfpc_dbg_continue(self.exp_bar, &self.rfpc);
        "S05".to_string()
    }

    fn cont_with_sig(&mut self, _packet: Vec<u8>) -> String {
        rfpc_dbg_continue(self.exp_bar, &self.rfpc);
        "S05".to_string()
    }

    fn set_breakpoint(&mut self, packet: Vec<u8>) -> String {
        // Extract the address and kind.
        let buffer_info = String::from_utf8_lossy(&packet[3..]);
        let mut split_iter = buffer_info.splitn(2, ",");

        // Extract and convert the address.
        let address_str = split_iter.next().expect("No address found in packet");
        let address = u64::from_str_radix(address_str, 16).expect("Failed to parse address as u64");

        // Check if the write is to CTM.
        let write_ctm: bool = ((address >> 48) & 0xF) == 0x1;

        // Mask to get the target address.
        let masked_address = address & 0x00000000FFFFFFFF;

        if write_ctm {
            let breakpoint_instr: Vec<u32> = vec![0x00100073];
            // Read the RISC-V instruction at the breakpoint location.
            let riscv_instr = mem_read(
                self.exp_bar,
                CppIsland::Rfpc0,
                MemoryType::Ctm,
                MuMemoryEngine::Atomic32,
                masked_address,
                1,
            );

            // Cache the RISC-V instruction and location.
            self.breakpoints.insert(address, riscv_instr[0] as u64);

            // Write breakpoint instruction to memory.
            mem_write(
                self.exp_bar,
                CppIsland::Rfpc0,
                MemoryType::Ctm,
                MuMemoryEngine::Atomic32,
                masked_address,
                breakpoint_instr,
            );
        } else {
            // Non-CTM case.
            let riscv_instr = rfpc_dbg_read_memory(self.exp_bar, &self.rfpc, masked_address, 1);

            // Cache the RISC-V instruction and location.
            self.breakpoints.insert(address, riscv_instr[0]);
            let bp_instr = (riscv_instr[0] & 0xFFFF_FFFF_0000_0000) | 0x0000_0000_0010_0073;

            rfpc_dbg_write_memory(self.exp_bar, &self.rfpc, masked_address, vec![bp_instr]);
        }

        "OK".to_string()
    }

    fn clear_breakpoint(&mut self, packet: Vec<u8>) -> String {
        // Extract the address and kind.
        let buffer_info = String::from_utf8_lossy(&packet[3..]);
        let mut split_iter = buffer_info.splitn(2, ",");

        // Extract and convert the address.
        let address_str = split_iter.next().expect("No address found in packet");
        let address = u64::from_str_radix(address_str, 16).expect("Failed to parse address as u64");

        // Get the RISC-V instruction at the breakpoint address from cache.
        let riscv_instr = if let Some(instruction) = self.breakpoints.get(&address) {
            vec![*instruction]
        } else {
            panic!("Breakpoint address not found in the cache!");
        };

        // Remove address from hashmap.
        self.breakpoints.remove(&address);

        // Check if the write is to CTM.
        let write_ctm: bool = ((address >> 48) & 0xF) == 0x1;
        let masked_address = address & 0x00000000FFFFFFFF;
        if write_ctm {
            let riscv_instr: Vec<u32> = vec![riscv_instr[0] as u32];
            // Write riscv instruction back to CTM (clear breakpoint).
            mem_write(
                self.exp_bar,
                CppIsland::Rfpc0,
                MemoryType::Ctm,
                MuMemoryEngine::Atomic32,
                masked_address,
                riscv_instr,
            );
        } else {
            // Write riscv instruction back to LMEM (clear breakpoint).
            rfpc_dbg_write_memory(self.exp_bar, &self.rfpc, masked_address, riscv_instr);
        }

        "OK".to_string()
    }

    /// Write memory at a specific target address.
    ///
    /// # Parameters
    ///
    /// * `packet - RSP packet after being parsed.
    ///
    /// # Returns
    ///
    /// * Returns 'OK' on successful memory write.
    fn memory_write(&mut self, packet: Vec<u8>) -> String {
        // Find the position of the colon.
        let colon_index = packet.iter().position(|&b| b == b':').unwrap();

        // Extract the first part (as string), split by the comma.
        let buffer_info = String::from_utf8_lossy(&packet[1..colon_index]);
        let mut split_iter = buffer_info.splitn(2, ",");

        // Extract the address.
        let address = split_iter.next().unwrap();

        // Extract the length.
        let length = split_iter.next().unwrap();

        // Convert the address and length.
        let mut address =
            u64::from_str_radix(&address, 16).expect("Failed to parse address as u64");
        let length = u64::from_str_radix(&length, 16).expect("Failed to parse address as u64");

        // The first loaded segment will always be a length of zero and should return OK.
        if length == 0 {
            return "OK".to_string();
        }

        let write_ctm: bool = ((address >> 48) & 0xF) == 0x1;

        // Extract target address.
        address &= 0x00000000FFFFFFFF;

        // Extract the program bytes (segment after the colon).
        let mut packet_data = packet[colon_index + 1..].to_vec();

        // Append zero bytes to make the length a multiple of 8 bytes.
        let remainder = length as usize % 8;
        if remainder != 0 {
            packet_data.extend(vec![0; 8 - remainder]);
            println!("Program segment not a multiple of u64");
        }

        if write_ctm {
            // Cast the byte slice to u32 vec safely.
            let program_data: Vec<u32> = cast_slice(&packet_data).to_vec();

            // Write program segment to memory.
            mem_write(
                self.exp_bar,
                CppIsland::Rfpc0,
                MemoryType::Ctm,
                MuMemoryEngine::Bulk32,
                address,
                program_data,
            );
        } else {
            // Cast the byte slice to u64 vec safely.
            let program_data: Vec<u64> = cast_slice(&packet_data).to_vec();

            rfpc_dbg_write_memory(self.exp_bar, &self.rfpc, address, program_data);
        }

        "OK".to_string()
    }

    /// Reads a specified number of bytes from memory at a given address.
    ///
    /// # Parameters
    /// - `packet` - containing the address and length in a comma-separated
    ///   format, e.g., `m1234abcd,10`.
    ///
    /// # Returns
    /// Memory content read at the given address in a hexadecimal format.
    ///
    /// # Panics
    /// - If `packet` is not properly formatted or the address and length
    ///   cannot be parsed from hex, the function will panic.
    fn memory_read(&mut self, packet: Vec<u8>) -> String {
        // Extract the first part (as string), split by the comma.
        let mem_info = String::from_utf8_lossy(&packet);
        let mut split_iter = mem_info.splitn(2, ",");

        // Extract the address and length.
        let address = split_iter.next().unwrap();
        let length = split_iter.next().unwrap();

        // Parse the address (remove leading 'm') and length from hex strings to u64.
        let mut address =
            u64::from_str_radix(&address[1..], 16).expect("Failed to parse address as u64");
        let length = u64::from_str_radix(length, 16).expect("Failed to parse length as u64");

        // Determine if we should read from CTM memory.
        let read_ctm = ((address >> 48) & 0xF) == 0x1;
        let mut mem_bytes = String::new();

        // Mask the address to extract the target.
        address &= 0x0000_0000_FFFF_FFFF;

        // Read memory based on the address type.
        if read_ctm {
            let word_len = (length + 3) / 4; // Calculate 32-bit word length
            let read_words: Vec<u32> = mem_read(
                self.exp_bar,
                CppIsland::Rfpc0,
                MemoryType::Ctm,
                MuMemoryEngine::Bulk32,
                address,
                word_len,
            );

            // Swap bytes and convert to byte vector.
            let mut read_bytes: Vec<u8> = cast_slice(
                &read_words
                    .iter()
                    .map(|&word| word.swap_bytes())
                    .collect::<Vec<u32>>(),
            )
            .to_vec();

            // Truncate to requested length.
            read_bytes.truncate(length as usize);

            // Convert bytes to hex string.
            for byte in read_bytes {
                mem_bytes.push_str(&format!("{:02x}", byte));
            }
        } else {
            let word_len = (length + 7) / 8; // Calculate 64-bit word length
            let read_qwords: Vec<u64> =
                rfpc_dbg_read_memory(self.exp_bar, &self.rfpc, address, word_len);

            // Truncate to requested length.
            let mut read_bytes: Vec<u8> = cast_slice(&read_qwords).to_vec();
            read_bytes.truncate(length as usize);

            // Convert bytes to hex string.
            for byte in read_bytes {
                mem_bytes.push_str(&format!("{:02x}", byte));
            }
        }

        mem_bytes
    }

    /// Code is not being relocated because the ELF file is assumed to be
    /// statically linked. Therefore the offsets in the address are the offsets
    /// we use on the chip.
    fn load_offsets(&mut self) -> String {
        "Text=000;Data=000;Bss=000".to_string()
    }

    /// This method parses the input packet to extract the features that
    /// the client supports. It then returns the features that the
    /// server supports.
    ///
    /// # Parameters
    ///
    /// * `packet: Vec<u8>` - The parsed packet received from the
    ///   RSP client.
    ///
    /// # Returns
    ///
    /// A semicolon-separated string of supported values and key-value
    /// pairs.
    fn supported_features(&mut self, packet: Vec<u8>) -> String {
        let colon_index = packet.iter().position(|&b| b == b':').unwrap();
        let args = String::from_utf8_lossy(&packet[colon_index + 1..]).to_string();

        // Process each feature in the args
        for feat in args.split(';') {
            if let Some((k, v)) = feat.split_once('=') {
                self.client_kv_support.insert(k.to_string(), v.to_string());
            } else {
                self.client_v_support.push(feat.to_string());
            }
        }

        // Create a response that includes only the key-value pairs from the server
        let mut response: Vec<String> = self
            .server_kv_support
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Also add server-supported values
        response.extend(
            self.server_v_support
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>(),
        );

        // Return the joined response
        response.join(";")
    }

    /// Disable packet +/- ACK NACK.
    ///
    /// The GDB client can request that, after connection, packet ACK
    /// and NACK is disabled and no longer necessary.
    ///
    /// # Returns
    ///
    /// A String confirming operation completed successfully.
    pub fn toggle_ack(&mut self) -> String {
        if !self.disable_ack {
            self.disable_ack = true;
        }
        "OK".to_string()
    }

    /// Handles an incoming RSP packet and determines what type of
    /// function to call.
    ///
    /// # Parameters
    ///
    /// * `packet - The parsed RSP packet.
    ///
    /// # Returns
    ///
    /// A String Option with return value sent back to the GDB client.
    fn handle_packet(&mut self, packet: Vec<u8>) -> Option<String> {
        // Extract the command by finding the position of the colon
        let colon_index = packet
            .iter()
            .position(|&b| b == b':')
            .unwrap_or(packet.len());
        let rsp_command = String::from_utf8_lossy(&packet[..colon_index]);
        println!("rsp_command =  {}", rsp_command);

        // Try to find the full command in the HashMap
        if let Some(response) = self.cmd_resp_map.get(rsp_command.as_ref()) {
            return match response {
                Some(FuncType::Ascii(resp)) => Some(resp.to_string()),
                Some(FuncType::NoArg(func)) => Some(func(self)),
                Some(FuncType::WithArg(func)) => Some(func(self, packet)),
                None => None,
            };
        }

        // If the full command is not found, check if any key is a prefix of `rsp_command`
        for (key, response) in &self.cmd_resp_map {
            if rsp_command.starts_with(key) {
                return match response {
                    Some(FuncType::Ascii(resp)) => Some(resp.to_string()),
                    Some(FuncType::NoArg(func)) => Some(func(self)),
                    Some(FuncType::WithArg(func)) => Some(func(self, packet)),
                    None => None,
                };
            }
        }

        // If neither the command nor any prefix is found
        println!("Unknown RSP command {}", rsp_command);
        Some(self.cmd_not_supported())
    }

    /// Calculates the checksum for an RSP packet.
    ///
    /// The checksum is calculated by summing the ASCII byte values of
    /// all characters in the data and taking the result modulo 256.
    /// This function is used to verify packet integrity.
    ///
    /// # Parameters
    ///
    /// * `data: &Vec<u8>` - A vector reference with the data.
    ///
    /// # Returns
    ///
    /// `u8` - value representing the computed checksum.
    fn calculate_rsp_checksum(&self, data: &Vec<u8>) -> u8 {
        data.iter().fold(0, |acc, &b| acc.wrapping_add(b))
    }

    /// Parses an incoming RSP packet from a TCP stream.
    ///
    /// This function reads the raw bytes from the provided `TcpStream`
    /// one byte at a time, looking for the start of an RSP packet
    /// (indicated by `$`), then reads the packet contents until it
    /// encounters the end of the packet (indicated by `#`). After
    /// reading the packet, the checksum is validated.
    ///
    /// # Parameters
    ///
    /// * `stream: &mut TcpStream` - Mutable reference to the `TcpStream`.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - If there are no errors during packet parsing.
    /// * `Ok(None)` - If the stream is closed by the client.
    /// * `Err(std::io::Error)` - IO error during packet reading.
    fn parse_rsp_packet(&self, stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
        let mut buffer_orig: Vec<u8> = Vec::new();
        let mut buffer: Vec<u8> = Vec::new();
        let mut byte: [u8; 1] = [0; 1];

        // Read 1 byte at a time until we find a starting '$'.
        while stream.read(&mut byte)? > 0 {
            if byte[0] == b'$' {
                break;
            }
        }

        // Read the rest of the packet until we hit '#', handling escaped characters.
        let mut escaped = false;
        while stream.read(&mut byte)? > 0 && byte[0] != b'#' {
            buffer_orig.push(byte[0]);

            if escaped {
                // Undo the escaping by XORing the byte with 0x20, and add the result to buffer
                buffer.push(byte[0] ^ 0x20);
                escaped = false;
            } else if byte[0] == 0x7d {
                // Escape detected, set the flag and skip adding this byte to buffer
                escaped = true;
            } else {
                // Normal byte, just push it to the buffer
                buffer.push(byte[0]);
            }
        }

        // Read the checksum (two hex characters) after the '#'.
        let mut checksum: [u8; 2] = [0; 2];
        stream.read_exact(&mut checksum)?;

        // Calculate checksum and validate.
        let expected_checksum = self.calculate_rsp_checksum(&buffer_orig);
        let received_checksum =
            u8::from_str_radix(&String::from_utf8_lossy(&checksum), 16).unwrap_or(0);

        if expected_checksum == received_checksum {
            if !self.disable_ack {
                stream.write_all(b"+")?; // Acknowledge valid packet
            }
            Ok(buffer)
        } else {
            // Return an error for checksum mismatch.
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Checksum mismatch",
            ))
        }
    }

    /// Formats a response string into an RSP packet.
    ///
    /// This function constructs a valid RSP packet by prepending a
    /// `$` character, appending a `#` character, and calculating the
    /// checksum.
    ///
    /// # Parameters
    ///
    /// * `response: &str` - The response data to be sent back to the client.
    ///
    /// # Returns
    ///
    /// `String` - A string representing the formatted RSP packet.
    fn format_rsp_packet(&self, response: &str) -> String {
        // Prepend the response with the start character '$'
        let mut packet = format!("${}", response);

        // Append the end character '#'
        packet.push('#');

        // Calculate the checksum
        let checksum = self.calculate_rsp_checksum(&response.as_bytes().to_vec());

        // Append the checksum in hexadecimal format (2 digits)
        packet.push_str(&format!("{:02x}", checksum));

        packet
    }

    /// Runs the RSP server, accepting and handling client connections.
    ///
    /// # Parameters
    ///
    /// * `running : Arc<AtomicBool>` - An atomic boolean flag
    ///   indicating whether the server should continue running. When
    ///   this flag is set to `false`, the server will gracefully shut
    ///   down.
    pub fn run(&mut self, running: Arc<AtomicBool>) {
        // Bind to an address and port.
        let listener =
            TcpListener::bind((LOCAL_HOST_IP, PORT)).expect("Failed to bind to local host!");

        // Set the listener to non-blocking mode.
        listener
            .set_nonblocking(true)
            .expect("Cannot set non-blocking");

        println!("Waiting for GDB connection");

        // Main loop: wait for a connection or check if the server should stop.
        while running.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, addr)) => {
                    println!("Connected to {:?}", addr);
                    // Handle message from the client.
                    while running.load(Ordering::SeqCst) {
                        match self.parse_rsp_packet(&mut stream) {
                            Ok(packet) => {
                                // Handle the packet based on its content.
                                match self.handle_packet(packet) {
                                    Some(resp_data) => {
                                        let resp_send: String;
                                        if resp_data == "detach" {
                                            let ack: String = "OK".to_string();
                                            resp_send = self.format_rsp_packet(&ack);
                                            stream.write_all(resp_send.as_bytes()).unwrap();
                                            sleep(Duration::from_millis(100));
                                            // Set running to false to break out of all loops
                                            running.store(false, Ordering::SeqCst);
                                            break;
                                        } else {
                                            resp_send = self.format_rsp_packet(&resp_data);
                                            println!("Reply: {}", resp_send);
                                            stream.write_all(resp_send.as_bytes()).unwrap();
                                        }
                                    }
                                    None => (), // Do nothing.
                                };
                            }
                            Err(e) => {
                                if !self.disable_ack {
                                    stream.write_all(b"-").unwrap();
                                }
                                println!("Failed to read packet: {}", e);
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection, sleep for a short duration to avoid busy waiting.
                    sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    // Unexpected error.
                    println!("Error accepting connection: {}", e);
                    break;
                }
            }
        }

        println!("Server shutting down gracefully.");
    }
}
