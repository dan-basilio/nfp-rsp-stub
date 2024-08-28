#![allow(dead_code)]
use clap::ValueEnum;
use std::fmt::{Debug, Display, Formatter, Result};

use crate::libs::cpp_bus::CppIsland;

pub trait RfpcReg: Display + Debug {
    fn reg_addr(&self) -> u64;
}

#[derive(Clone, Debug, ValueEnum)]
pub enum RfpcGpr {
    X0,  // Hard wired to zero.
    X1,  // Return address.
    X2,  // Stack pointer.
    X3,  // Global pointer.
    X4,  // Thread pointer (TLS).
    X5,  // Temporary register 0.
    X6,  // Temporary register 1.
    X7,  // Temporary register 2.
    X8,  // Saved register 0 / frame pointer.
    X9,  // Saved register 1.
    X10, // Function argument 0 / return value 0.
    X11, // Function argument 1 / return value 1.
    X12, // Function argument 2.
    X13, // Function argument 3.
    X14, // Function argument 4.
    X15, // Function argument 5.
    X16, // Function argument 6.
    X17, // Function argument 7.
    X18, // Saved register 2.
    X19, // Saved register 3.
    X20, // Saved register 4.
    X21, // Saved register 5.
    X22, // Saved register 6.
    X23, // Saved register 7.
    X24, // Saved register 8.
    X25, // Saved register 9.
    X26, // Saved register 10.
    X27, // Saved register 11.
    X28, // Temporary register 3.
    X29, // Temporary register 4.
    X30, // Temporary register 5.
    X31, // Temporary register 6.
}

impl RfpcReg for RfpcGpr {
    fn reg_addr(&self) -> u64 {
        match self {
            RfpcGpr::X0 => 0x1000,
            RfpcGpr::X1 => 0x1001,
            RfpcGpr::X2 => 0x1002,
            RfpcGpr::X3 => 0x1003,
            RfpcGpr::X4 => 0x1004,
            RfpcGpr::X5 => 0x1005,
            RfpcGpr::X6 => 0x1006,
            RfpcGpr::X7 => 0x1007,
            RfpcGpr::X8 => 0x1008,
            RfpcGpr::X9 => 0x1009,
            RfpcGpr::X10 => 0x100A,
            RfpcGpr::X11 => 0x100B,
            RfpcGpr::X12 => 0x100C,
            RfpcGpr::X13 => 0x100D,
            RfpcGpr::X14 => 0x100E,
            RfpcGpr::X15 => 0x100F,
            RfpcGpr::X16 => 0x1010,
            RfpcGpr::X17 => 0x1011,
            RfpcGpr::X18 => 0x1012,
            RfpcGpr::X19 => 0x1013,
            RfpcGpr::X20 => 0x1014,
            RfpcGpr::X21 => 0x1015,
            RfpcGpr::X22 => 0x1016,
            RfpcGpr::X23 => 0x1017,
            RfpcGpr::X24 => 0x1018,
            RfpcGpr::X25 => 0x1019,
            RfpcGpr::X26 => 0x101A,
            RfpcGpr::X27 => 0x101B,
            RfpcGpr::X28 => 0x101C,
            RfpcGpr::X29 => 0x101D,
            RfpcGpr::X30 => 0x101E,
            RfpcGpr::X31 => 0x101F,
        }
    }
}

impl Display for RfpcGpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            RfpcGpr::X0 => write!(f, "x0"),
            RfpcGpr::X1 => write!(f, "x1"),
            RfpcGpr::X2 => write!(f, "x2"),
            RfpcGpr::X3 => write!(f, "x3"),
            RfpcGpr::X4 => write!(f, "x4"),
            RfpcGpr::X5 => write!(f, "x5"),
            RfpcGpr::X6 => write!(f, "x6"),
            RfpcGpr::X7 => write!(f, "x7"),
            RfpcGpr::X8 => write!(f, "x8"),
            RfpcGpr::X9 => write!(f, "x9"),
            RfpcGpr::X10 => write!(f, "x10"),
            RfpcGpr::X11 => write!(f, "x11"),
            RfpcGpr::X12 => write!(f, "x12"),
            RfpcGpr::X13 => write!(f, "x13"),
            RfpcGpr::X14 => write!(f, "x14"),
            RfpcGpr::X15 => write!(f, "x15"),
            RfpcGpr::X16 => write!(f, "x16"),
            RfpcGpr::X17 => write!(f, "x17"),
            RfpcGpr::X18 => write!(f, "x18"),
            RfpcGpr::X19 => write!(f, "x19"),
            RfpcGpr::X20 => write!(f, "x20"),
            RfpcGpr::X21 => write!(f, "x21"),
            RfpcGpr::X22 => write!(f, "x22"),
            RfpcGpr::X23 => write!(f, "x23"),
            RfpcGpr::X24 => write!(f, "x24"),
            RfpcGpr::X25 => write!(f, "x25"),
            RfpcGpr::X26 => write!(f, "x26"),
            RfpcGpr::X27 => write!(f, "x27"),
            RfpcGpr::X28 => write!(f, "x28"),
            RfpcGpr::X29 => write!(f, "x29"),
            RfpcGpr::X30 => write!(f, "x30"),
            RfpcGpr::X31 => write!(f, "x31"),
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum RfpcCsr {
    Mstatus,
    Misa,
    Medeleg,
    Mideleg,
    Mie,
    Mtvec,
    Mscratch,
    Mepc,
    Mcause,
    Mtval,
    Mip,
    Dcsr,
    Dpc,
    Dscratch0,
    Dscratch1,
    Mlmemprot,
    Mafstatus,
    Mcycle,
    Minstret,
    Cycle,
    Time,
    Instret,
    Mvendorid,
    Marchid,
    Mimpid,
    Mhartid,
}

impl RfpcReg for RfpcCsr {
    fn reg_addr(&self) -> u64 {
        match self {
            RfpcCsr::Mstatus => 0x300,
            RfpcCsr::Misa => 0x301,
            RfpcCsr::Medeleg => 0x302,
            RfpcCsr::Mideleg => 0x303,
            RfpcCsr::Mie => 0x304,
            RfpcCsr::Mtvec => 0x305,
            RfpcCsr::Mscratch => 0x340,
            RfpcCsr::Mepc => 0x341,
            RfpcCsr::Mcause => 0x342,
            RfpcCsr::Mtval => 0x343,
            RfpcCsr::Mip => 0x344,
            RfpcCsr::Dcsr => 0x7b0,
            RfpcCsr::Dpc => 0x7b1,
            RfpcCsr::Dscratch0 => 0x7b2,
            RfpcCsr::Dscratch1 => 0x7b3,
            RfpcCsr::Mlmemprot => 0x7c0,
            RfpcCsr::Mafstatus => 0x7c1,
            RfpcCsr::Mcycle => 0xb00,
            RfpcCsr::Minstret => 0xb02,
            RfpcCsr::Cycle => 0xc00,
            RfpcCsr::Time => 0xc01,
            RfpcCsr::Instret => 0xc02,
            RfpcCsr::Mvendorid => 0xf11,
            RfpcCsr::Marchid => 0xf12,
            RfpcCsr::Mimpid => 0xf13,
            RfpcCsr::Mhartid => 0xf14,
        }
    }
}

impl Display for RfpcCsr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            RfpcCsr::Mstatus => write!(f, "mstatus"),
            RfpcCsr::Misa => write!(f, "misa"),
            RfpcCsr::Medeleg => write!(f, "medeleg"),
            RfpcCsr::Mideleg => write!(f, "mideleg"),
            RfpcCsr::Mie => write!(f, "mie"),
            RfpcCsr::Mtvec => write!(f, "mtvec"),
            RfpcCsr::Mscratch => write!(f, "mscratch"),
            RfpcCsr::Mepc => write!(f, "mepc"),
            RfpcCsr::Mcause => write!(f, "mcause"),
            RfpcCsr::Mtval => write!(f, "mtval"),
            RfpcCsr::Mip => write!(f, "mip"),
            RfpcCsr::Dcsr => write!(f, "dcsr"),
            RfpcCsr::Dpc => write!(f, "dpc"),
            RfpcCsr::Dscratch0 => write!(f, "dscratch0"),
            RfpcCsr::Dscratch1 => write!(f, "dscratch1"),
            RfpcCsr::Mlmemprot => write!(f, "mlmemport"),
            RfpcCsr::Mafstatus => write!(f, "mafstatus"),
            RfpcCsr::Mcycle => write!(f, "mcycle"),
            RfpcCsr::Minstret => write!(f, "minstret"),
            RfpcCsr::Cycle => write!(f, "cycle"),
            RfpcCsr::Time => write!(f, "time"),
            RfpcCsr::Instret => write!(f, "instret"),
            RfpcCsr::Mvendorid => write!(f, "mvendorid"),
            RfpcCsr::Marchid => write!(f, "marchid"),
            RfpcCsr::Mimpid => write!(f, "mimpid"),
            RfpcCsr::Mhartid => write!(f, "mhartid"),
        }
    }
}

#[derive(Clone)]
pub struct Rfpc {
    pub island: CppIsland,
    pub cluster: u8,
    pub group: u8,
    pub core: u8,
}

impl Rfpc {
    pub fn new(island: CppIsland, cluster: u8, group: u8, core: u8) -> Self {
        if cluster > 2 {
            panic!("Cluster number out of range")
        } else if group > 3 {
            panic!("Group number out of range")
        } else if core > 7 {
            panic!("Core number out of range")
        };

        Rfpc {
            island,
            cluster,
            group,
            core,
        }
    }

    pub fn from_island_group_core(island: CppIsland, group: u8, core: u8) -> Self {
        let cluster = group / 4;
        let group = group % 4;

        Rfpc {
            island,
            cluster,
            group,
            core,
        }
    }

    pub fn dm_xpb_base(&self) -> u32 {
        match self.cluster {
            0 => 0x240000,
            1 => 0x320000,
            2 => 0x400000,
            _ => panic!("Invalid Cluster"),
        }
    }

    pub fn group_ctl_xpb_base(&self) -> (u32, u32) {
        let cluster = match self.cluster {
            0 => 0x280000,
            1 => 0x360000,
            2 => 0x440000,
            _ => panic!("Invalid cluster ID {}", self.cluster),
        };

        let group = match self.group {
            0 => 0x000,
            1 => 0x080,
            2 => 0x100,
            3 => 0x180,
            _ => panic!("Invalid group ID {}", self.group),
        };

        (cluster, group)
    }

    pub fn dm_hartsel(&self) -> (u32, u32) {
        let hartsel: u32 = 8 * (self.group as u32) + (self.core as u32);
        let hartsello: u32 = hartsel & 0x3FF;
        let hartselhi: u32 = (hartsel >> 10) & 0x3FF;
        (hartsello, hartselhi)
    }

    pub fn imb_port(&self) -> u8 {
        match self.cluster {
            0 => match self.group {
                0 | 1 => 4,
                2 | 3 => 7,
                _ => panic!("Invalid group"),
            },
            1 => match self.group {
                0 | 1 => 8,
                2 | 3 => 11,
                _ => panic!("Invalid group"),
            },
            2 => match self.group {
                0 | 1 => 12,
                2 | 3 => 13,
                _ => panic!("Invalid group"),
            },
            _ => panic!("Invalid cluster"),
        }
    }

    pub fn cpp_core_num(&self) -> u8 {
        (8 * self.group + self.core) % 16
    }
}

impl Display for Rfpc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "i{}.cl{}.g{}.c{}",
            self.island.id(),
            self.cluster,
            self.group,
            self.core
        )
    }
}

impl PartialEq for Rfpc {
    fn eq(&self, other: &Self) -> bool {
        self.island == other.island
            && self.cluster == other.cluster
            && self.group == other.group
            && self.core == other.core
    }
}
