use clap::{ArgAction, Parser};
use clap_num::maybe_hex;

use nfp_debug_tools::libs::common::{hex_parser, validate_nfp_bdf};
use nfp_debug_tools::libs::cpp_bus::CppIsland;
use nfp_debug_tools::libs::expansion_bar::{init_device_bars, ExpansionBar};
use nfp_debug_tools::libs::mem_access::{mem_read, mem_write, MemoryType, MuMemoryEngine};

/// Struct representing the CLI arguments
#[derive(Parser, Debug)]
#[command(
    about = "Read and write NFP memory over the CPP bus.",
    long_about = None,
    after_help = "Example usage: mem -Z 0000:65:00.0 -i rfpc0 -m Ctm -e Bulk32 -a 0x0 -l 4"
)]
struct Cli {
    #[arg(short = 'Z', long = "pci-bdf", required = true, value_parser = validate_nfp_bdf)]
    pci_bdf: String,

    #[arg(short = 'i', long = "island", required = true)]
    island: CppIsland,

    #[arg(short = 'm', long = "mem-type", required = true)]
    mem_type: MemoryType,

    #[arg(short = 'e', long = "mem-engine", default_value_t = MuMemoryEngine::Bulk32)]
    mem_engine: MuMemoryEngine,

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

    if cli.values.is_empty() {
        let read_words = mem_read(
            &mut exp_bar,
            cli.island,
            cli.mem_type,
            cli.mem_engine,
            cli.address,
            cli.length,
        );
        for (index, value) in read_words.iter().enumerate() {
            println!(
                "address 0x{:08x}: 0x{:08x}",
                cli.address + (index * 4) as u64,
                value
            );
        }
    } else {
        let mut values_to_write: Vec<u32> = Vec::new();
        values_to_write.extend(cli.values.iter().cycle().take(cli.length as usize));
        mem_write(
            &mut exp_bar,
            cli.island,
            cli.mem_type,
            cli.mem_engine,
            cli.address,
            values_to_write,
        );
    }
}
