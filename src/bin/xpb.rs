use clap::{ArgAction, Parser};
use clap_num::maybe_hex;

use nfp_debug_tools::libs::common::{hex_parser, validate_nfp_bdf};
use nfp_debug_tools::libs::cpp_bus::CppIsland;
use nfp_debug_tools::libs::expansion_bar::{init_device_bars, ExpansionBar};
use nfp_debug_tools::libs::xpb_bus::{xpb_read, xpb_write};

/// Struct representing the CLI arguments
#[derive(Parser, Debug)]
#[command(
    about = "Read and write data on the CPP bus.",
    long_about = None,
    after_help = "Example usage: xpb -Z 0000:65:00.0 -i rfpc0 -a 0x0 -l 4 -x"
)]
struct Cli {
    #[arg(short = 'Z', long = "pci-bdf", required = true, value_parser = validate_nfp_bdf)]
    pci_bdf: String,

    #[arg(short = 'i', long = "island", required = true)]
    island: CppIsland,

    #[arg(short = 'a', long = "address", required = true, value_parser = maybe_hex::<u32>)]
    address: u32,

    #[arg(short = 'l', long = "length", default_value_t = 1, value_parser = maybe_hex::<u64>)]
    length: u64,

    #[arg(short = 'v', long = "value", action = ArgAction::Append, num_args = 1.., value_parser = hex_parser)]
    values: Vec<u32>,

    #[arg(short = 'x', long = "xpbm", action = ArgAction::SetTrue)]
    xpbm: bool,
}

fn main() {
    let cli = Cli::parse();

    // Initialize the PCIe BARs in the PCIe config. space.
    init_device_bars(&cli.pci_bdf);

    // Allocate a new expansion BAR for the PCIe device.
    let mut exp_bar = ExpansionBar::new(&cli.pci_bdf, None);

    if cli.values.is_empty() {
        // Read over Xpb bus.
        let read_words = xpb_read(&mut exp_bar, &cli.island, cli.address, cli.length, cli.xpbm);
        println!("0x{:08x}", read_words[0]);
    } else {
        // Write over Xpb bus.
        xpb_write(&mut exp_bar, &cli.island, cli.address, cli.values, cli.xpbm);
    }
}
