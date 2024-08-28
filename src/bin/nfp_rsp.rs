use clap::Parser;

use ctrlc;
use nfp_debug_tools::libs::common::validate_nfp_bdf;
use nfp_debug_tools::libs::cpp_bus::CppIsland;
use nfp_debug_tools::libs::expansion_bar::{init_device_bars, ExpansionBar};
use nfp_debug_tools::libs::rsp_server_stub::RspServer;
use nfp_debug_tools::libs::xpb_bus::xpb_write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Argument parser for CLI arguments.
#[derive(Parser, Debug)]
#[command(
    about = "Start an RSP debug server to connect to an NFP RISC-V debugger.",
    long_about = None,
    after_help = "Example usage: nfp-rsp -Z 0000:65:00.0 -i rfpc0 -u 0 -g 0 -c 0"
)]
struct Cli {
    #[arg(short = 'Z', long = "pci-bdf", required = true, value_parser = validate_nfp_bdf)]
    pci_bdf: String,

    #[arg(short = 'i', long = "island")]
    island: Option<CppIsland>,

    #[arg(short = 'u', long = "cluster")]
    cluster: Option<u8>,

    #[arg(short = 'g', long = "group")]
    group: Option<u8>,

    #[arg(short = 'c', long = "core")]
    core: Option<u8>,
}

fn main() {
    let cli = Cli::parse();

    // Initialize the PCIe BARs in the PCIe config space.
    init_device_bars(&cli.pci_bdf);

    // Allocate a new expansion BAR for the PCIe device.
    let mut exp_bar = ExpansionBar::new(&cli.pci_bdf, None);

    // Use an atomic flag to handle ctrl+c termination.
    let running = Arc::new(AtomicBool::new(true));

    // Handle ctrl+c to gracefully exit.
    ctrlc::set_handler({
        let running = running.clone();
        move || {
            println!("\n\nKeyboard interrupt received (ctrl+C). Exiting.");
            running.store(false, Ordering::SeqCst);
        }
    })
    .expect("Error setting Ctrl-C handler");

    // Use defaults for inputs not provided.
    let island = if let Some(island) = cli.island {
        island
    } else {
        CppIsland::Rfpc0
    };

    let cluster = if let Some(cluster) = cli.cluster {
        cluster
    } else {
        0
    };

    let group = if let Some(group) = cli.group {
        group
    } else {
        0
    };

    let core = if let Some(core) = cli.core { core } else { 0 };

    // Disable memory access control for specified RFPC group.
    let grp_base_addr = 0x280000 + (0xE0000 * cluster as u32) + (0x100 * group as u32);
    xpb_write(&mut exp_bar, &island, grp_base_addr, vec![0x7], true);
    xpb_write(&mut exp_bar, &island, grp_base_addr + 0x40, vec![0], true);
    xpb_write(
        &mut exp_bar,
        &island,
        grp_base_addr + 0x44,
        vec![0xFF0159],
        true,
    );

    // Create an instance of RspServer.
    let mut rsp_server = RspServer::new(&mut exp_bar, island, cluster, group, core);

    // Run the server in the main thread.
    rsp_server.run(running);
}
