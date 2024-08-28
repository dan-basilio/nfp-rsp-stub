use std::convert::TryInto;

use clap::{ArgAction, Parser};

use nfp_debug_tools::libs::common::validate_nfp_bdf;
use nfp_debug_tools::libs::cpp_bus::CppIsland;
use nfp_debug_tools::libs::expansion_bar::{init_device_bars, ExpansionBar};
use nfp_debug_tools::libs::rfpc::Rfpc;
use nfp_debug_tools::libs::rfpc_trace::{
    format_uncomp_trace, pa_trigger_on_uncomp_trace, read_trace,
};

/// Struct representing the CLI arguments
#[derive(Parser, Debug)]
#[command(
    about = "Capture RFPC trace information.",
    long_about = None,
    after_help = "Example usage: reg -Z 0000:65:00.0 -i rfpc0 -u 0 -r 0 -c 0 -tp -ts -n 5 -b 1 -w 1 -t"
)]
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

    #[arg(long = "tp", action = ArgAction::SetFalse)]
    trace_pc: bool,

    #[arg(long = "ts", action = ArgAction::SetFalse)]
    trace_seq: bool,

    #[arg(long = "tb", action = ArgAction::SetFalse)]
    trace_bp: bool,

    #[arg(long = "tr", action = ArgAction::SetFalse)]
    trace_reg: bool,

    #[arg(short = 'n', long = "num-samples", required = true)]
    num_samples: u32,

    #[arg(short = 'b', long = "bus-words", required = true)]
    bus_words: u32,

    #[arg(short = 'w', long = "word-index", required = true)]
    word_index: u32,

    #[arg(long = "t", long = "timestamp", action = ArgAction::SetFalse)]
    timestamp: bool,
}

fn main() {
    let cli = Cli::parse();

    // Panic for illegal CLI arguments.
    if cli.timestamp && (cli.bus_words == 2) {
        panic!(
            "Cannot sample only 2 bus words and a timestamp. \
                A timestamp is only available when sampling either 1 or 3 words \
                on the Performance bus."
        );
    }

    if !cli.timestamp && (cli.bus_words == 3) {
        panic!(
            "Timestamp must be read when sampling all 3 words \
                on the performance bus."
        );
    }

    // Initialize the PCIe BARs in the PCIe config space.
    init_device_bars(&cli.pci_bdf);

    let mut words_per_sample = cli.bus_words;
    if cli.timestamp {
        words_per_sample += 1;
    }

    // Allocate a new expansion BAR for the PCIe device.
    let mut exp_bar = ExpansionBar::new(&cli.pci_bdf, None);

    // Create an RFPC from input parameters.
    let rfpc = Rfpc {
        island: cli.island,
        cluster: cli.cluster,
        group: cli.group,
        core: cli.core,
    };

    // Configure Performance Analyzer to trigger on an uncompressed trace.
    let mut pa = pa_trigger_on_uncomp_trace(
        &mut exp_bar,
        &rfpc,
        cli.trace_pc,
        cli.trace_seq,
        cli.trace_bp,
        cli.trace_reg,
        cli.bus_words,
        cli.word_index,
        cli.timestamp,
    );

    // Read the specified number of samples from the Performance Analyzer.
    let samples: Vec<u32> = read_trace(&mut pa, cli.num_samples * words_per_sample);
    // Format the samples
    let formatted_lines = format_uncomp_trace(
        samples,
        cli.bus_words,
        cli.word_index,
        cli.timestamp,
        words_per_sample.try_into().unwrap(),
    );
    for line in formatted_lines {
        println!("{}", line);
    }
}
