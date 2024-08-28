use clap::{ArgAction, Parser};
use clap_num::maybe_hex;

use nfp_debug_tools::libs::common::{hex_parser, validate_nfp_bdf};
use nfp_debug_tools::libs::cpp_bus::{CppBus, CppIsland, CppLength, CppTarget};
use nfp_debug_tools::libs::expansion_bar::{init_device_bars, ExpansionBar};

/// Struct representing the CLI arguments
#[derive(Parser, Debug)]
#[command(
    about = "Read and write data on the CPP bus.",
    long_about = None,
    after_help = "Example usage: cpp -Z 0000:65:00.0 -i rfpc0 -t mem -c 34 -o 0 -p len32 -a 0x0 -l 4"
)]
struct Cli {
    #[arg(short = 'Z', long = "pci-bdf", required = true, value_parser = validate_nfp_bdf)]
    pci_bdf: String,

    #[arg(short = 'i', long = "island", required = true)]
    island: CppIsland,

    #[arg(short = 't', long = "target", required = true)]
    target: CppTarget,

    #[arg(short = 'c', long = "action", required = true)]
    action: u8,

    #[arg(short = 'o', long = "token", required = true)]
    token: u8,

    #[arg(short = 'p', long = "cpp-len", default_value_t = CppLength::Len32)]
    cpp_len: CppLength,

    #[arg(short = 'a', long = "address", required = true, value_parser = maybe_hex::<u64>)]
    address: u64,

    #[arg(short = 'l', long = "length", default_value_t = 1, value_parser = maybe_hex::<u64>)]
    length: u64,

    #[arg(short = 'v', long = "value", action = ArgAction::Append, num_args = 1.., value_parser = hex_parser)]
    values: Vec<u32>,
}

fn main() {
    let cli = Cli::parse();

    // Initialize the PCIe BARs in the PCIe config. space.
    init_device_bars(&cli.pci_bdf);

    // Allocate a new expansion BAR for the PCIe device.
    let mut exp_bar = ExpansionBar::new(&cli.pci_bdf, None);

    // Instantiate Cpp bus with allocated expansion BAR.
    let mut cpp_bus = CppBus::new(&mut exp_bar);

    if cli.values.is_empty() {
        // Read over CPP bus.
        let read_words = cpp_bus.read(
            cli.island,
            cli.target,
            cli.action,
            cli.token,
            cli.cpp_len,
            cli.address,
            cli.length,
        );
        for value in read_words {
            println!("0x{:08x}", value);
        }
    } else {
        // Write over CPP bus.
        cpp_bus.write(
            cli.island,
            cli.target,
            cli.action,
            cli.token,
            cli.cpp_len,
            cli.address,
            cli.values,
        );
    }
}
