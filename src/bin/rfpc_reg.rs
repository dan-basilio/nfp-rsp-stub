use clap::{ArgGroup, Parser};
use clap_num::maybe_hex;

use nfp_debug_tools::libs::common::validate_nfp_bdf;
use nfp_debug_tools::libs::cpp_bus::CppIsland;
use nfp_debug_tools::libs::expansion_bar::{init_device_bars, ExpansionBar};
use nfp_debug_tools::libs::rfpc::{Rfpc, RfpcCsr, RfpcGpr, RfpcReg};
use nfp_debug_tools::libs::rfpc_debugger::{read_rfpc_reg, write_rfpc_reg};

/// Struct representing the CLI arguments
#[derive(Parser, Debug)]
#[command(
    about = "Read and write RFPC registers (GPRs and CSRs).",
    long_about = None,
    after_help = "Example usage: reg -Z 0000:65:00.0 -i rfpc0 -u 0 -r 0 -c 0 -s mhartid -v 0x9000"
)]
#[command(group(ArgGroup::new("register")
    .required(true)
    .args(&["gpr", "csr"])))]
struct Cli {
    #[arg(short = 'Z', long = "pci-bdf", required = true, value_parser = validate_nfp_bdf)]
    pci_bdf: String,

    #[arg(short = 'i', long = "island", required = true)]
    island: CppIsland,

    #[arg(short = 'u', long = "cluster", required = true)]
    cluster: u8,

    #[arg(short = 'r', long = "group", required = true)]
    group: u8,

    #[arg(short = 'c', long = "core", required = true)]
    core: u8,

    #[arg(short = 's', long = "csr")]
    csr: Option<RfpcCsr>,

    #[arg(short = 'p', long = "gpr")]
    gpr: Option<RfpcGpr>,

    #[arg(short = 'v', long = "value", value_parser = maybe_hex::<u64>)]
    value: Option<u64>,
}

fn main() {
    let cli = Cli::parse();

    // Initialize the PCIe BARs in the PCIe config. space.
    init_device_bars(&cli.pci_bdf);

    // Allocate a new expansion BAR for the PCIe device.
    let mut exp_bar = ExpansionBar::new(&cli.pci_bdf, None);

    let rfpc = Rfpc {
        island: cli.island,
        cluster: cli.cluster,
        group: cli.group,
        core: cli.core,
    };

    // Check whether we're dealing with a GPR or CSR register.
    let reg_addr: Box<dyn RfpcReg> = if let Some(csr_reg) = cli.csr {
        Box::new(csr_reg)
    } else if let Some(gpr_reg) = cli.gpr {
        Box::new(gpr_reg)
    } else {
        panic!("Error: Either CSR or GPR must be provided.");
    };

    if let Some(value) = cli.value {
        // Value provided - write to the register
        write_rfpc_reg(&mut exp_bar, &rfpc, &reg_addr, value);
    } else {
        // Read from the register
        let val = read_rfpc_reg(&mut exp_bar, &rfpc, &reg_addr);
        println!("{}:{} = 0x{:016x}", rfpc, reg_addr, val);
    }
}
