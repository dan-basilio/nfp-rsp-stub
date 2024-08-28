#![allow(dead_code)]

use crate::libs::expansion_bar::ExpansionBar;
use crate::libs::performance_analyzer::{
    CaptureMethod, CaptureMode, CaptureStart, EventMethod, HistogramSource, PerfCounterAction,
    PerformanceAnalyzer, TcamCaptureSource, TcamCaptureType,
};
use crate::libs::rfpc::Rfpc;
use crate::libs::xpb_bus::xpb_write;
use bitfield::bitfield;

macro_rules! rfpc_pa_control {
    ($cluster:expr, $group:expr) => {
        0x280000 + (($cluster as u32) * 0xE) + (($group as u32) * 0x100) + 0x0020
    };
}

macro_rules! rfpc_perf_mux_config {
    ($cluster:expr, $group:expr) => {
        0x280000 + (($cluster as u32) * 0xE) + (($group as u32) * 0x100) + 0x0024
    };
}

// PAControl bitfields (see High Speed Performance Analyzer
// Peripheral EAS v0.3, section 2.3)
bitfield! {
    pub struct PAControl(u32);
    impl Debug;
    u32;
    pub enable, set_enable: 0;
    pub select, set_select: 3, 1;
    pub compress, set_compress: 4;
    pub capture_64, set_capture_64: 5;
    pub reserved1, set_reserved1: 7, 6;
    pub trace_en, set_trace_en: 8;
    pub trace_ctl, set_trace_ctl: 9;
    pub trace_pc, set_trace_pc: 10;
    pub trace_rfw, set_trace_rfw: 11;
    pub trace_bkpt, set_trace_bkpt: 12;
    pub reserved2, set_reserved2: 31, 13;
}

// PerfMuxConfig bitfields (see High Speed Performance Analyzer
// Peripheral EAS v0.3, section 2.3)
bitfield! {
    pub struct PerfMuxConfig(u32);
    impl Debug;
    u32;
    pub lane_select_lo, set_lane_select_lo: 1, 0;
    pub lane_select_mid, set_lane_select_mid: 3, 2;
    pub lane_select_hi, set_lane_select_hi: 5, 4;
    pub low_mux_select, set_low_mux_select: 9, 6;
    pub mid_mux_select, set_mid_mux_select: 13, 10;
    pub hi_mux_select, set_hi_mux_select: 17, 14;
    pub aux_select, set_aux_select: 20, 18;
    pub reserved, set_reserved: 31, 21;
}

/// Configures the Performance Analyzer for tracing based on specified parameters.
///
/// # Parameters
/// - `exp_bar`: Reference to the expansion bar for configuration.
/// - `rfpc`: Reference to the RFPC structure holding core parameters.
/// - `trace_pc`: Flag indicating whether to trace program counter.
/// - `trace_seq`: Flag indicating whether to trace sequential instructions.
/// - `trace_bp`: Flag indicating whether to trace breakpoints.
/// - `trace_reg`: Flag indicating whether to trace registers.
/// - `bus_words`: Number of bus words to sample.
/// - `word_index`: Performance analyzer bus word that should be sampled first.
/// - `timestamp`: Flag indicating whether to include timestamps.
///
/// # Returns
/// A configured `PerformanceAnalyzer`.
pub fn pa_trigger_on_uncomp_trace<'a>(
    exp_bar: &'a mut ExpansionBar,
    rfpc: &'a Rfpc,
    trace_pc: bool,
    trace_seq: bool,
    trace_bp: bool,
    trace_reg: bool,
    bus_words: u32,
    word_index: u32,
    timestamp: bool,
) -> PerformanceAnalyzer<'a> {
    // Determine the capture method based on bus_words and timestamp
    let capture_method = match bus_words {
        1 => {
            if timestamp {
                CaptureMethod::PerfBus32andTs
            } else {
                CaptureMethod::PerfBus32orTs
            }
        }
        2 => CaptureMethod::PerfBus64,
        _ => CaptureMethod::PerfBus96andTs,
    };

    let capture_start = match word_index {
        0 => CaptureStart::LowBusInFifoFirst,
        1 => CaptureStart::MidBusInFifoFirst,
        2 => CaptureStart::HighBusInFifoFirst,
        _ => panic!("Invalid word index!"),
    };

    // Build up the Performance Analyzer configuration and start it up.
    let pa = PerformanceAnalyzer::new(exp_bar, rfpc.island)
        .set_pa_global_config(
            false,
            false,
            false,
            false,
            HistogramSource::LowCaptureSource,
            CaptureMode::StoreInFifo,
            false,
            PerfCounterAction::DoNothing,
            capture_start,
            capture_method,
            0,
            EventMethod::NoEvents,
            false,
            0,
            false,
            false,
            false,
            true,
        )
        .set_mask_compare(0, 0, 0x08, 0x08, false)
        .set_mask_compare(0, 1, 0x80, 0x80, false)
        .set_mask_compare(1, 2, 0x80, 0x80, false)
        .set_mask_compare(2, 3, 0x01, 0x01, false)
        .set_mask_compare_detect(0, 0x0000, 0x000F)
        .set_capture_tcam(
            0,
            TcamCaptureType::CaptureData,
            TcamCaptureSource::MaskCompareDetectors,
            0x01,
            0x01,
            false,
        )
        .start_pa();

    // Set up and enable trace output for specified RFPC core.
    let mut pa_mux = PerfMuxConfig(0);
    pa_mux.set_lane_select_lo(1);
    pa_mux.set_lane_select_mid(2);
    pa_mux.set_lane_select_hi(3);

    xpb_write(
        pa.exp_bar,
        &pa.cpp_island,
        rfpc_perf_mux_config!(rfpc.cluster, rfpc.group),
        vec![pa_mux.0],
        false,
    );

    let mut pa_control = PAControl(0);
    pa_control.set_enable(true);
    pa_control.set_select(rfpc.core as u32);
    pa_control.set_trace_en(true);
    pa_control.set_trace_ctl(trace_seq);
    pa_control.set_trace_pc(trace_pc);
    pa_control.set_trace_rfw(trace_reg);
    pa_control.set_trace_bkpt(trace_bp);

    xpb_write(
        pa.exp_bar,
        &pa.cpp_island,
        rfpc_pa_control!(rfpc.cluster, rfpc.group),
        vec![pa_control.0],
        false,
    );

    pa
}

/// Applies the Performance Analyzer settings, initiates the Performance Analyzer trigger,
/// and reads the specified number of samples from the Performance Analyzer FIFO. After
/// collecting the samples, it stops the trigger.
///
/// # Parameters
///
/// - `pa`: The `PerformanceAnalyzer` instance to read from.
/// - `num_words`: The number of 32-bit words to read from the FIFO.
///
/// # Returns
///
/// A `Vec<u32>` containing the 32-bit samples read from the Performance Analyzer's FIFO.
pub fn read_trace(pa: &mut PerformanceAnalyzer, num_words: u32) -> Vec<u32> {
    // Set the trigger to an idle state to ensure it's ready for sampling.
    pa.trigger_idle();
    // Start the trigger without any active states.
    pa.trigger_start(0, 0);

    // Initialize a vector to store the FIFO samples.
    let mut fifo_samples: Vec<u32> = Vec::new();

    // Continue reading samples until we have the requested number of words.
    while fifo_samples.len() < num_words as usize {
        let remaining_words: u32 = num_words - fifo_samples.len() as u32;
        // Read FIFO samples from the Performance Analyzer.
        let samples = pa.read_fifo(remaining_words);
        fifo_samples.extend(samples);
    }

    // Halt the trigger after collecting samples.
    pa.trigger_halt();

    // Return only the requested number of samples.
    fifo_samples.truncate(num_words as usize);
    fifo_samples
}

/// Formats uncompressed RFPC trace samples for display.
///
/// This function takes a vector of 32-bit sample values and formats them into a
/// human-readable string representation, organizing the output into a table-like
/// format. The header of the table includes the word indices, and if applicable,
/// a timestamp column. The samples are grouped based on the specified number of
/// words per sample.
///
/// # Parameters
///
/// * `samples`: A `Vec<u32>` containing the 32-bit samples to be formatted.
/// * `bus_words`: The number of words per sample from the performance bus.
/// * `word_index`: The index of the first word to be read.
/// * `timestamp`: A boolean indicating whether to include a timestamp column.
/// * `words_per_sample`: The number of words that constitute a single sample.
///
/// # Returns
///
/// Returns a `Vec<String>` where each string represents a formatted line of the
/// table, including a header and the individual sample values in hexadecimal format.
///
/// # Example
///
/// ```
/// let samples = vec![0xDEADBEEF, 0xCAFEBABE, 0xB16B00B5];
/// let formatted_lines = format_uncomp_trace(samples, 3, 0, true, 3);
/// for line in formatted_lines {
///     println!("{}", line);
/// }
/// ```
/// This would print a formatted table with a timestamp and the corresponding sample words.
pub fn format_uncomp_trace(
    samples: Vec<u32>,
    bus_words: u32,
    word_index: u32,
    timestamp: bool,
    words_per_sample: usize,
) -> Vec<String> {
    let mut formatted_lines = Vec::new();

    // Initialize header_line and determine modulus
    let mut header_line = Vec::new();
    let modulus = bus_words + 1 + timestamp as u32;

    // Adjust modulus if it equals word_index
    let mut adjusted_modulus = modulus;
    if word_index == adjusted_modulus {
        adjusted_modulus += 1;
    }

    // Generate header words
    for i in 0..bus_words {
        let header_word = if bus_words != 3 {
            (word_index + i) % adjusted_modulus
        } else {
            i
        };
        header_line.push(format!("WORD {}", header_word));
    }

    // Insert timestamp header if applicable
    if timestamp {
        header_line.insert(0, "TIMESTAMP".to_string());
    }

    // Format and add the header line
    formatted_lines.push(format!("| {} |", header_line.join(" | ")));

    // Determine timestamp index if applicable
    let ts_index: Option<usize> = if bus_words == 1 { Some(0) } else { None };

    // Process each sample
    for chunk in samples.chunks(words_per_sample) {
        let mut sample_line = Vec::new();

        // If we have a timestamp index, extract the timestamp
        if let Some(index) = ts_index {
            if chunk.len() > index {
                let ts = chunk[index];
                sample_line.push(format!("{:>10}", ts));
                // Create a new vector excluding the timestamp
                let sample_without_ts: Vec<u32> = chunk
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != index) // Exclude the timestamp index
                    .map(|(_, &v)| v)
                    .collect();
                sample_line.extend(sample_without_ts.iter().map(|&v| format!("{:#010x}", v)));
            }
        } else {
            // No timestamp, just format the sample as hex
            sample_line.extend(chunk.iter().map(|&v| format!("{:#010x}", v)));
        }

        // Add formatted line
        formatted_lines.push(format!("| {} |", sample_line.join(" | ")));
    }

    formatted_lines
}
