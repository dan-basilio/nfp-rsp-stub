#![allow(dead_code)]

use crate::libs::cpp_bus::CppIsland;
use bitfield::bitfield;
use bitfield::fmt::Debug;

use crate::libs::expansion_bar::ExpansionBar;
use crate::libs::xpb_bus::{xpb_read, xpb_write};

/// Performance Analyzer XPB register MAP offsets.
const PA_CONFIG: u32 = 0x0000;
const PA_STATUS: u32 = 0x0004;
const PA_TIMER: u32 = 0x0008;
const PA_FIFO_CONTROL: u32 = 0x0010;
const PA_FIFO_DATA: u32 = 0x0014;
const PA_TRIGGER_STATUS: u32 = 0x0018;
const PA_TRIGGER_CONTROL: u32 = 0x001C;
const PA_TRIGGER_COUNTER_RESTART: [u32; 2] = [0x0020, 0x0024];
const PA_TRIGGER_COUNTER: [u32; 2] = [0x0028, 0x002C];
const PA_MASK_COMPARE: u32 = 0x0040;
const PA_MASK_COMPARE_DETECT: [u32; 8] = [
    0x0060, 0x0064, 0x0068, 0x006C, 0x0070, 0x0074, 0x0078, 0x007C,
];
const PA_TRIGGER_TRANSITION_CONFIG: [[u32; 2]; 8] = [
    [0x0080, 0x0084], // Transition 0
    [0x0088, 0x008C], // Transition 1
    [0x0090, 0x0094], // Transition 2
    [0x0098, 0x009C], // Transition 3
    [0x00A0, 0x00A4], // Transition 4
    [0x00A8, 0x00AC], // Transition 5
    [0x00B0, 0x00B4], // Transition 6
    [0x00B8, 0x00BC], // Transition 7
];
const PA_CAPTURE_TCAM: [u32; 8] = [
    0x00C0, 0x00C4, 0x00C8, 0x00CC, 0x00D0, 0x00D4, 0x00D8, 0x00DC,
];
const PA_PERFORMANCE_COUNTER: [u32; 4] = [0x00E0, 0x00E4, 0x00E8, 0x00EC];

// PAConfig bitfields (see High Speed Performance Analyzer Peripheral EAS v0.3,
// section 2.3)
bitfield! {
    pub struct PAConfig(u32);
    impl Debug;
    u32;
    pub active, set_active: 0;
    pub enable_as_valid, set_enable_as_valid: 1;
    pub halt_on_inactive, set_halt_on_inactive: 2;
    pub reserved1, set_reserved1: 3;
    pub journalling, set_journalling: 4;
    pub histogram_shift, set_histogram_shift: 7, 5;
    pub histogram_128, set_histogram_128: 8;
    pub event_method, set_event_method: 10, 9;
    pub reserved2, set_reserved2: 12, 11;
    pub capture_trigger, set_capture_trigger: 15, 13;
    pub capture_method, set_capture_method: 17, 16;
    pub capture_start, set_capture_start: 19, 18;
    pub pc_action, set_pc_action: 22, 20;
    pub pc_stats, set_pc_stats: 23;
    pub capture_mode, set_capture_mode: 25, 24;
    pub histogram_source, set_histogram_source: 27, 26;
    pub rv_decompress, set_rv_decompress: 28;
    pub rv_trace_64, set_rv_trace_64: 29;
    pub rv_trigger_decomp, set_rv_trigger_decomp: 30;
    pub rv_capture_decomp, set_rv_capture_decomp: 31;
}

// PAStatus bitfields (see High Speed Performance Analyzer Peripheral EAS v0.3,
// section 2.3)
bitfield! {
    pub struct PAStatus(u32);
    impl Debug;
    u32;
    pub active, set_active: 0;
    pub async_, set_async_: 1;
    pub reserved1, set_reserved1: 3, 2;
    pub journalling, set_journalling: 4;
    pub reserved2, set_reserved2: 8, 5;
    pub event_method, set_event_method: 10, 9;
    pub valid, set_valid: 11;
    pub reserved3, set_reserved3: 12;
    pub capture_trigger, set_capture_trigger: 15, 13;
    pub capture_method, set_capture_method: 17, 16;
    pub capture_start, set_capture_start: 19, 18;
    pub pc_action, set_pc_action: 22, 20;
    pub pc_stats, set_pc_stats: 23;
    pub capture_mode, set_capture_mode: 25, 24;
    pub histogram_source, set_histogram_source: 27, 26;
    pub rv_decompress, set_rv_decompress: 28;
    pub rv_trace_64, set_rv_trace_64: 29;
    pub rv_trigger_decomp, set_rv_trigger_decomp: 30;
    pub rv_capture_decomp, set_rv_capture_decomp: 31;
}

// PAFifoControl bitfields (see High Speed Performance Analyzer Peripheral EAS
// v0.3, section 2.3)
bitfield! {
    pub struct PAFifoControl(u32);
    impl Debug;
    u32;
    pub read_ptr, set_read_ptr: 14, 0;
    pub write_ptr, set_write_ptr: 29, 15;
    pub empty, set_empty: 30;
    pub overflow, set_overflow: 31;
}

// PATriggerStatus bitfields (see High Speed Performance Analyzer Peripheral
// EAS v0.3, section 2.3)
bitfield! {
    pub struct PATriggerStatus(u32);
    impl Debug;
    u32;
    pub fsm, set_fsm: 1, 0;
    pub trigger_states, set_trigger_states: 9, 2;
    pub ext_pending_in, set_ext_pending_in: 10;
    pub trigger_out, set_trigger_out: 11;
    pub timeout, set_timeout: 31, 12;
}

// PATriggerControl bitfields (see High Speed Performance Analyzer Peripheral
// EAS v0.3, section 2.3)
bitfield! {
    pub struct PATriggerControl(u32);
    impl Debug;
    u32;
    pub trigger_command, set_trigger_command: 1, 0;
    pub active_states, set_active_states: 9, 2;
    pub reserved, set_reserved: 11, 10;
    pub timeout, set_timeout: 31, 12;
}

// PATriggerCounterRestart bitfields (see High Speed Performance Analyzer
// Peripheral EAS v0.3, section 2.3)
bitfield! {
    pub struct PATriggerCounterRestart(u32);
    impl Debug;
    u32;
    pub restart_counter, set_restart_counter: 31, 0;
}

// PAMaskCompare bitfields (see High Speed Performance Analyzer
// Peripheral EAS v0.3, section 2.3)
bitfield! {
    pub struct PAMaskCompare(u32);
    impl Debug;
    u32;
    pub value, set_value: 7, 0;
    pub mask, set_mask: 15, 8;
    pub mask_compare, set_mask_compare: 19, 16;
    pub reserved1, set_reserved1: 23, 20;
    pub invert, set_invert: 24;
    pub reserved2, set_reserved2: 27, 25;
    pub select, set_select: 31, 28;
}

impl Clone for PAMaskCompare {
    fn clone(&self) -> Self {
        PAMaskCompare(self.0)
    }
}

// PAMaskCompareDetect bitfields (see High Speed Performance Analyzer
// Peripheral EAS v0.3, section 2.3)
bitfield! {
    pub struct PAMaskCompareDetect(u32);
    impl Debug;
    u32;
    pub value, set_value: 15, 0;
    pub mask, set_mask: 31, 16;
}

impl Clone for PAMaskCompareDetect {
    fn clone(&self) -> Self {
        PAMaskCompareDetect(self.0)
    }
}

// PACaptureTCAM bitfields (see High Speed Performance Analyzer
// Peripheral EAS v0.3, section 2.3)
bitfield! {
    pub struct PACaptureTCAM(u32);
    impl Debug;
    u32;
    pub mask, set_mask: 7, 0;
    pub value, set_value: 15, 8;
    pub source, set_source: 17, 16;
    pub invert, set_invert: 18;
    pub reserved1, set_reserved1: 23, 19;
    pub capture_type, set_capture_type: 26, 24;
    pub reserved2, set_reserved2: 31, 27;
}

impl Clone for PACaptureTCAM {
    fn clone(&self) -> Self {
        PACaptureTCAM(self.0)
    }
}

// PATriggerTransitionConfig0 bitfields (see High Speed Performance Analyzer
// Peripheral EAS v0.3, section 2.3)
bitfield! {
    pub struct PATriggerTransitionConfig0(u32);
    impl Debug;
    u32;
    pub state_mask, set_state_mask: 7, 0;
    pub mcd_mask, set_mcd_mask: 15, 8;
    pub mcd_value, set_mcd_value: 23, 16;
    pub counters_zero_mask, set_counters_zero_mask: 25, 24;
    pub counters_nonzero_mask, set_counters_nonzero_mask: 27, 26;
    pub ext_mask, set_ext_mask: 28;
    pub invert, set_invert: 29;
    pub reserved, set_reserved: 31, 30;
}

impl Clone for PATriggerTransitionConfig0 {
    fn clone(&self) -> Self {
        PATriggerTransitionConfig0(self.0)
    }
}

// PATriggerTransitionConfig1 bitfields (see High Speed Performance Analyzer
// Peripheral EAS v0.3, section 2.3)
bitfield! {
    pub struct PATriggerTransitionConfig1(u32);
    impl Debug;
    u32;
    pub destination_mask, set_destination_mask: 7, 0;
    pub reserved1, set_reserved1: 15, 8;
    pub counter_restart, set_counter_restart: 17, 16;
    pub counter_inc, set_counter_inc: 19, 18;
    pub counter_dec, set_counter_dec: 21, 20;
    pub reserved2, set_reserved2: 31, 22;
}

impl Clone for PATriggerTransitionConfig1 {
    fn clone(&self) -> Self {
        PATriggerTransitionConfig1(self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)] // Ensure the enum uses an 8-bit unsigned integer
pub enum HistogramSource {
    LowCaptureSource = 0,  // Low of the capture source.
    MidCaptureSource = 1,  // Mid of the capture source.
    HighCaptureSource = 2, // High of the capture source.
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum CaptureMode {
    StoreInFifo = 0,              // Store data in the FIFO.
    ChangePerfCounters = 1,       // Do performance counters.
    HistogramAndPerfCounters = 2, // Do histogram with performance counters.
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum PerfCounterAction {
    DoNothing = 0,                                // Do nothing (hold).
    IncPerfCounter = 1,                           // Increment the performance counter.
    AddTriggerCounter0ToPerfCounter = 2, // Add trigger counter value 0 to performance counter.
    SetPerfCounterToZero = 3,            // Set the performance counter to zero.
    SetPerfCounterValueToTriggerCounterValue = 4, // Set the perf counter = trigger counter.
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum CaptureStart {
    LowBusInFifoFirst = 0, // First 32 bits of FIFO data comes from low 32 bits of bus.
    MidBusInFifoFirst = 1, // First 32 bits of FIFO data comes from mid 32 bits of bus.
    HighBusInFifoFirst = 2, // First 32 bits of FIFO data comes from high 32 bits of bus.
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum CaptureMethod {
    PerfBus32orTs = 0,  // 32 data bits of performance bus or timestamp.
    PerfBus32andTs = 1, // 32 data bits of performance bus and timestamp.
    PerfBus64 = 2,      // 64 data bits of performance bus (mid/low, high/mid or low/high).
    PerfBus96andTs = 3, // 96 data bits of performance bus and timestamp.
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum EventMethod {
    NoEvents = 0,          // Don't generate events.
    EventOnFifoEmpty = 1,  // Generate event on event bus when FIFO becomes empty.
    EventOnFifoFull = 2,   // Generate event on event bus when FIFO becomes full.
    EventOnExtTrigger = 3, // Generate event on event bus when external trigger out toggles.
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TriggerControlStates {
    StartTrigger = 1, // Start trigger if idle.
    HaltTrigger = 2,  // Halt trigger if running.
    IdleTrigger = 3,  // Puts trigger into idle state from any other state.
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TcamCaptureType {
    IgnoreTcam = 0,           // Ignore TCAM matching.
    CaptureData = 2,          // Capture data in FIFO when TCAM matches.
    CaptureDataIfChanged = 3, // Capture data in FIFO if different from previous captured data.
    PerfCounting = 4,         // Update Performance Counters based on TCAM match.
    ToggleTrigger = 7,        // Toggle Trigger output if TCAM matches to trigger another PA.
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum TcamCaptureSource {
    MaskCompareDetectors = 0,    // Use Mask Compare Detectors for TCAM match.
    TriggerStateTransitions = 1, // Use trigger state transitions for TCAM match.
}

/// A struct representing the High Speed Performance Analyzer Peripheral.
///
/// This struct initializes the High Speed Performance Analyzer Peripheral. Each
/// island on the NFP contains one such peripheral. On RFPC islands, it is located
/// in the Cluster Local Scratch (CLS). The peripheral functions similarly to a
/// logic analyzer, allowing for various signals on each island to be sampled or
/// for counters to increment based on complex trigger conditions.
///
/// The Performance Analyzer's bus width for sampling in each island is 96 bits.
/// While there are thousands of possible signals to place on the bus in each
/// island, only 96 can be sampled at any given time. These signals are all
/// wire-ORed to 96 bits before entering the peripheral. There are multiple
/// methods to multiplex the desired signals onto the 96-bit bus that feeds
/// into the peripheral. The specifics of this process are detailed in the EAS
/// for the relevant island where the Performance Analyzer is located. Silicon
/// design engineers for each island decide which signals can be analyzed on the
/// bus.
///
/// This struct primarily deals with configuring the Performance Analyzer
/// Peripheral in each island to either increment counters, sample the bus in
/// the FIFO, or set up events based on various user-defined trigger conditions.
/// The selection or multiplexing of signals on the bus is not covered, as this
/// configuration is generally done by a firmware application based on its
/// specific needs and signals of interest. For more detailed information on the
/// peripheral's inner workings, please refer to the High Speed Performance
/// Analyzer Peripheral EAS v0.3.
///
/// # Fields
///
/// * `exp_bar`: A mutable reference to a PCIe expansion BAR.
/// * `cpp_island`: CppIsland in which the performance analyzer resides.
pub struct PerformanceAnalyzer<'a> {
    pub exp_bar: &'a mut ExpansionBar,
    pub cpp_island: CppIsland,
    pub pa_base_addr: u32,
    pa_configuration: PAConfig,
    mask_compare_units: Vec<PAMaskCompare>,
    mask_compare_detect_units: Vec<PAMaskCompareDetect>,
    tcam_capture_units: Vec<PACaptureTCAM>,
    state_transitions: Vec<(PATriggerTransitionConfig0, PATriggerTransitionConfig1)>,
}

impl<'a> PerformanceAnalyzer<'a> {
    pub fn new(exp_bar: &'a mut ExpansionBar, cpp_island: CppIsland) -> Self {
        let pa_configuration = PAConfig(0);
        let mask_compare_units = vec![PAMaskCompare(0); 16];
        let mask_compare_detect_units = vec![PAMaskCompareDetect(0); 8];
        let tcam_capture_units = vec![PACaptureTCAM(0); 8];
        let mut state_transitions: Vec<(PATriggerTransitionConfig0, PATriggerTransitionConfig1)> =
            Vec::new();

        for _ in 0..7 {
            let config0 = PATriggerTransitionConfig0(0);
            let config1 = PATriggerTransitionConfig1(0);
            state_transitions.push((config0, config1));
        }

        let pa_base_addr: u32 = match cpp_island {
            CppIsland::Rfpc0 => 0x000F0000,
            _ => panic!("Island not supported yet"),
        };

        PerformanceAnalyzer {
            exp_bar,
            cpp_island,
            pa_base_addr,
            pa_configuration,
            mask_compare_units,
            mask_compare_detect_units,
            tcam_capture_units,
            state_transitions,
        }
    }

    /// This method applies the Performance Analyzer configuration
    /// and starts it.
    pub fn start_pa(mut self) -> Self {
        self.apply_configuration();
        self
    }

    /// Configures the Performance Analyzer for a specific mode of operation.
    ///
    /// **Note!** This method does not configure the `PAConfig` register
    /// on the Performance Analyzer Peripheral itself. It only updates the local
    /// configuration. The registers are only updated when the
    /// `_apply_configuration()` method is called when starting the Performance
    /// Analyzer. Local configuration is useful to keep track of state because
    /// most of the Performance Analyzer configuration registers are write-only.
    ///
    /// # Parameters
    ///
    /// - `capture_decomp`: `bool` - Capture decompressed output when decompression is enabled.
    /// - `trigger_decomp`: `bool` - Trigger on decompressed output when decompression is enabled.
    /// - `trace64_decomp`: `bool` - When decompressing and `true`, then trace is 64-bits, else 96-bits.
    /// - `decompress`: `bool` - Decompress the trace on the PA bus.
    /// - `hist_source`: `HistogramSource` - Low, High, or Mid of capture source used for Histogram.
    /// - `capture_mode`: `CaptureMode` - What mode to use to capture data.
    /// - `pc_stats`: `bool` - Do stats for performance counters.
    /// - `pc_action`: `PerfCounterAction` - Action to perform on performance counters.
    /// - `capture_start`: `CaptureStart` - Which section of the performance bus to capture in FIFO first (if
    ///   capturing more than one section).
    /// - `capture_method`: `CaptureMethod` - Method of capturing data.
    /// - `capture_trigger`: `u8` - Instead of capturing the perf low, mid, and high, replace with
    ///   trigger for low, counter0 for mid, and counter1 for high, depending
    ///   on what bits are set (3 bits).
    /// - `event_method`: `EventMethod` - Specifies when to place an event on the event bus.
    /// - `histogram128`: `bool` - If set for histograms, use 128 bits of SRAM per index and 4 PCs; if
    ///   clear, use 32 bits and only read/write PC0.
    /// - `histogram_shift`: `u8` - Histogram shift for memory address shifting.
    /// - `journalling`: `bool` - If set, then journal the FIFO, else overflow on full.
    /// - `halt_on_inactive`: `bool` - Set if trigger should stop when no states are active.
    /// - `enable_as_valid`: `bool` - Only track valid data or enable on every cycle.
    ///
    /// # Panics
    ///
    /// Panics if input parameters are invalid or not in range.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to `self`.
    pub fn set_pa_global_config(
        mut self,
        capture_decomp: bool,
        trigger_decomp: bool,
        trace64_decomp: bool,
        decompress: bool,
        hist_source: HistogramSource,
        capture_mode: CaptureMode,
        pc_stats: bool,
        pc_action: PerfCounterAction,
        capture_start: CaptureStart,
        capture_method: CaptureMethod,
        capture_trigger: u8,
        event_method: EventMethod,
        histogram128: bool,
        histogram_shift: u8,
        journalling: bool,
        halt_on_inactive: bool,
        enable_as_valid: bool,
        valid: bool,
    ) -> Self {
        if capture_trigger >= (1 << 3) {
            panic!("capture_trigger bitmask can only be 3 bits maximum.");
        }
        if histogram_shift >= (1 << 3) {
            panic!("histogram_shift value can only be 3 bits maximum.");
        }

        self.pa_configuration.set_active(valid);
        self.pa_configuration.set_enable_as_valid(enable_as_valid);
        self.pa_configuration.set_halt_on_inactive(halt_on_inactive);
        self.pa_configuration.set_journalling(journalling);
        self.pa_configuration
            .set_histogram_shift(histogram_shift as u32);
        self.pa_configuration.set_histogram_128(histogram128);
        self.pa_configuration.set_event_method(event_method as u32);
        self.pa_configuration
            .set_capture_trigger(capture_trigger as u32);
        self.pa_configuration
            .set_capture_method(capture_method as u32);
        self.pa_configuration
            .set_capture_start(capture_start as u32);
        self.pa_configuration.set_pc_action(pc_action as u32);
        self.pa_configuration.set_pc_stats(pc_stats);
        self.pa_configuration.set_capture_mode(capture_mode as u32);
        self.pa_configuration
            .set_histogram_source(hist_source as u32);
        self.pa_configuration.set_rv_decompress(decompress);
        self.pa_configuration.set_rv_trace_64(trace64_decomp);
        self.pa_configuration.set_rv_trigger_decomp(trigger_decomp);
        self.pa_configuration.set_rv_capture_decomp(capture_decomp);

        self
    }

    /// Configures a mask/compare unit for one of the 12 bytes on
    /// the Performance Analyzer's 96-bit bus.
    ///
    /// There are a total of 16 mask/compare units in the Performance Analyzer Peripheral.
    /// With four more mask/compare units than bytes on the performance bus, multiple
    /// units can be assigned to the same byte, providing flexibility in
    /// creating trigger conditions.
    ///
    /// The mask/compare match is always implemented with 8-bit mask/compare
    /// fields for a specific performance bus byte. Each of the 8 bits in the
    /// mask/compare unit can be encoded in one of four 2-bit combinations:
    /// 2b00, 2b01, 2b10, and 2b11.
    ///
    /// - `2b00`: (mask bit 0, compare bit 0) - The selected PA bus byte's bit is
    /// ignored for the match result to be 'True'.
    ///
    /// - `2b10`: (mask bit 1, compare bit 0) - The selected PA bus byte's bit must
    /// be zero for the match result to be 'True'.
    ///
    /// - `2b11`: (mask bit 1, compare bit 1) - The selected PA bus byte's bit must
    /// be one for the match result to be 'True'.
    ///
    /// - `2b01`: (mask bit 0, compare bit 1) - Marks a bit as part of a set (up to
    /// 8 bits) that must be set for the match result to be 'True'. In this
    /// case, all bits marked with 2b01 are extracted and compared to 0. If all
    /// extracted bits are zero, the match is 'False'; if any bit is one, the
    /// match is 'True'. If no bits are marked with 2b01, this phase is skipped.
    /// For example, a mask/compare of 8b00000000/8b10100000 will match if the
    /// selected PA bus byte has either bit 7 or bit 5 set. To check for a
    /// non-zero value, use a mask of 8b00000000 and a compare of 8b11111111,
    /// indicating a match if any bit in the selected byte is set.
    ///
    /// If the final match result is 'True', the output of the mask/compare unit
    /// is 1. If 'False', the output is 0. The output can also be inverted after
    /// the match for additional flexibility.
    ///
    /// **Note!** This method does not configure the `PAMaskCompare` registers
    /// on the Performance Analyzer Peripheral itself. It only updates the local
    /// configuration. The registers are only updated when the
    /// `_apply_configuration()` method is called when starting the Performance
    /// Analyzer. Local configuration is useful to keep track of state because
    /// most of the Performance Analyzer configuration registers are write-only.
    ///
    /// # Parameters
    ///
    /// - `byte_num`: `u8` - Byte number on Performance bus to mask/compare (0-11).
    ///   Byte 0 refers to the least significant byte.
    /// - `mask_compare_unit_num`: `u8` - Mask/compare unit number to mask/compare the byte (0-15).
    /// - `mask`: `u8` - Mask value to use for mask/compare (must be 8 bits).
    /// - `compare`: `u8` - Compare value to use for mask/compare (must be 8 bits).
    /// - `invert_output`: `bool` - Invert the output of the mask/compare unit.
    ///
    /// # Panics
    ///
    /// Panics if input parameters are not in range.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to `self`.
    pub fn set_mask_compare(
        mut self,
        byte_num: u8,
        mask_compare_unit_num: u8,
        mask: u8,
        compare: u8,
        invert_output: bool,
    ) -> Self {
        if byte_num >= (1 << 4) {
            panic!("byte_num can only be 4 bits maximum.");
        }
        if mask_compare_unit_num >= (1 << 4) {
            panic!("mask_compare_unit_num can only be 4 bits maximum.");
        }

        self.mask_compare_units[mask_compare_unit_num as usize].set_value(compare as u32);
        self.mask_compare_units[mask_compare_unit_num as usize].set_mask(mask as u32);
        self.mask_compare_units[mask_compare_unit_num as usize]
            .set_mask_compare(mask_compare_unit_num as u32);
        self.mask_compare_units[mask_compare_unit_num as usize].set_invert(invert_output);
        self.mask_compare_units[mask_compare_unit_num as usize].set_select(byte_num as u32);

        self
    }

    /// Configures one of the 8 Mask Compare Detect units.
    ///
    /// The outputs from the 16 mask/compare units (which match on performance bus
    /// bytes) are fed into each of the 8 Mask Compare Detect units, providing
    /// an additional layer of flexibility for triggering certain conditions.
    ///
    /// The mask/compare rules for the Mask Compare Detect units are the same as
    /// those for the mask/compare units, with the difference being that the
    /// mask/compare operation is performed on a 16-bit value instead of an
    /// 8-bit value.
    ///
    /// The outputs of the Mask Compare Detect units will be 1 when the match is
    /// 'True' and 0 when 'False'. These outputs can be used to drive trigger
    /// state transitions or be directly utilized by the TCAM Capture units to
    /// capture data in the FIFO or increment Performance Counters.
    ///
    /// **Note!** This method does not configure the `PAMaskCompareDetect` registers
    /// on the Performance Analyzer Peripheral itself. It only updates the local
    /// configuration. The registers are only updated when the
    /// `_apply_configuration()` method is called when starting the Performance
    /// Analyzer. Local configuration is useful to keep track of state because
    /// most of the Performance Analyzer configuration registers are write-only.
    ///
    /// # Parameters
    ///
    /// - `unit_num`: `u8` - MaskCompareDetect unit number (0-7).
    /// - `mask`: `u16` - Mask value to use for mask/compare (must be 16 bits).
    /// - `compare`: `u16` - Compare value to use for mask/compare (must be 16 bits).
    ///
    /// # Panics
    ///
    /// Panics if input parameters are not in range.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to `self`.
    pub fn set_mask_compare_detect(mut self, unit_num: u8, mask: u16, compare: u16) -> Self {
        if unit_num >= (1 << 3) {
            panic!("unit_num can only be 3 bits maximum.");
        }

        self.mask_compare_detect_units[unit_num as usize].set_value(compare as u32);
        self.mask_compare_detect_units[unit_num as usize].set_mask(mask as u32);

        self
    }

    /// Configures a TCAM capture unit for data capture based on input conditions.
    ///
    /// The trigger state machine transitions or the outputs of the Mask Compare
    /// Detectors can serve as inputs to the TCAM capture units. These inputs
    /// determine whether and what form of data capture to perform in a given
    /// cycle, such as updating performance counters, capturing data to the
    /// FIFO, or even capturing performance counters to the FIFO, depending on
    /// the configuration. There are 8 TCAM capture units, each of which can be
    /// independently configured to capture data based on different input
    /// conditions.
    ///
    /// **Note!** This method does not configure the `PACaptureTCAM` registers
    /// on the Performance Analyzer Peripheral itself. It only updates the local
    /// configuration. The registers are only updated when the
    /// `_apply_configuration()` method is called when starting the Performance
    /// Analyzer. Local configuration is useful to keep track of state because
    /// most of the Performance Analyzer configuration registers are write-only.
    ///
    /// # Parameters
    ///
    /// - `unit_num`: `u8` - TCAM Capture unit number to configure.
    /// - `capture_type`: `TcamCaptureType` - Type of data capture to do if TCAM matches.
    /// - `capture_source`: `TcamCaptureSource` - Source of input data to the TCAM match unit.
    /// - `mask`: `u8` - Mask value to use for mask/compare (must be 8 bits).
    /// - `compare`: `u8` - Compare value to use for mask/compare (must be 8 bits).
    /// - `invert_tcam_output`: `bool` - Invert TCAM unit match output.
    ///
    /// # Panics
    ///
    /// Panics if input parameters are not in range.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to `self`.
    pub fn set_capture_tcam(
        mut self,
        unit_num: u8,
        capture_type: TcamCaptureType,
        capture_source: TcamCaptureSource,
        mask: u8,
        compare: u8,
        invert_tcam_output: bool,
    ) -> Self {
        if unit_num >= (1 << 3) {
            panic!("unit_num can only be 3 bits maximum.");
        }

        self.tcam_capture_units[unit_num as usize].set_mask(mask as u32);
        self.tcam_capture_units[unit_num as usize].set_value(compare as u32);
        self.tcam_capture_units[unit_num as usize].set_source(capture_source as u32);
        self.tcam_capture_units[unit_num as usize].set_invert(invert_tcam_output);
        self.tcam_capture_units[unit_num as usize].set_capture_type(capture_type as u32);

        self
    }

    /// Configures a state transition from one set of states to another.
    ///
    /// The high-speed performance analyzer employs an NFA (nondeterministic finite
    /// automata) approach to triggering, in contrast to the traditional DFA
    /// (deterministic finite automata) approach. The key difference between a
    /// DFA and an NFA is that an NFA can be in multiple states (or no states)
    /// simultaneously. In the DFA approach, the analyzer only needs to check
    /// for transitions from its current state; with the NFA approach, the
    /// analyzer must consider all possible transitions at once.
    ///
    /// **Note!** This method does not configure the `PATriggerTransitionConfig`
    /// registers on the Performance Analyzer Peripheral itself. It only updates
    /// the local configuration. The registers are only updated when the
    /// `_apply_configuration()` method is called when starting the Performance
    /// Analyzer. Local configuration is useful to keep track of state because
    /// most of the Performance Analyzer configuration registers are write-only.
    ///
    /// # Parameters
    ///
    /// - `transition_num`: `u8` - State transition number to configure.
    /// - `state_mask`: `u32` - Mask of which combination of trigger states have this transition.
    /// - `mcd_mask`: `u32` - Mask for Mask Compare Detector output matching.
    /// - `mcd_compare`: `u32` - Compare for Mask Compare Detector output matching.
    /// - `counters_zero_mask`: `u32` - Mask of which trigger counters need to be zero for this transition to occur.
    /// - `counters_nonzero_mask`: `u32` - Mask of which trigger counters need to be non-zero for this transition to occur.
    /// - `ext_mask`: `bool` - If `true`, then external trigger must have fired for this transition to be valid.
    /// - `invert`: `bool` - If `true`, then invert transition detection result.
    /// - `counter_dec_mask`: `u32` - Mask of which trigger counters should be decremented if this transition fires.
    /// - `counter_inc_mask`: `u32` - Mask of which trigger counters should be incremented if this transition fires.
    /// - `counter_restart_mask`: `u32` - Mask of which trigger counters should be restarted if this transition fires.
    /// - `destination_mask`: `u32` - Mask of which states should be transitioned to if this transition fires.
    ///
    /// # Panics
    ///
    /// Panics if input parameters are not in range.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to `self`.
    pub fn set_state_transition(
        mut self,
        transition_num: u8,
        state_mask: u8,
        mcd_mask: u8,
        mcd_compare: u8,
        counters_zero_mask: u8,
        counters_nonzero_mask: u8,
        ext_mask: bool,
        invert: bool,
        counter_dec_mask: u8,
        counter_inc_mask: u8,
        counter_restart_mask: u8,
        destination_mask: u8,
    ) -> Self {
        if transition_num >= (1 << 3) {
            panic!("transition_num can only be 3 bits maximum.");
        }
        if counters_zero_mask >= (1 << 2) {
            panic!("counters_zero_mask can only be 2 bits maximum.");
        }
        if counter_dec_mask >= (1 << 2) {
            panic!("counter_dec_mask can only be 2 bits maximum.");
        }
        if counter_inc_mask >= (1 << 2) {
            panic!("counter_inc_mask can only be 2 bits maximum.");
        }
        if counter_restart_mask >= (1 << 2) {
            panic!("counter_restart_mask can only be 2 bits maximum.");
        }

        let (config0, config1) = &mut self.state_transitions[transition_num as usize];

        // For PATriggerTransitionConfig0
        config0.set_state_mask(state_mask as u32);
        config0.set_mcd_mask(mcd_mask as u32);
        config0.set_mcd_value(mcd_compare as u32);
        config0.set_counters_zero_mask(counters_zero_mask as u32);
        config0.set_counters_nonzero_mask(counters_nonzero_mask as u32);
        config0.set_ext_mask(ext_mask);
        config0.set_invert(invert);

        // For PATriggerTransitionConfig1
        config1.set_destination_mask(destination_mask as u32);
        config1.set_counter_restart(counter_restart_mask as u32);
        config1.set_counter_inc(counter_inc_mask as u32);
        config1.set_counter_dec(counter_dec_mask as u32);

        self
    }

    /// Applies the local configuration to the Performance Analyzer XPB registers.
    ///
    /// This method ensures that any local configuration changes are written to
    /// the corresponding registers in the Performance Analyzer Peripheral.
    fn apply_configuration(&mut self) {
        xpb_write(
            self.exp_bar,
            &self.cpp_island,
            self.pa_base_addr + PA_CONFIG,
            vec![self.pa_configuration.0],
            false,
        );

        for mc_val in &self.mask_compare_units {
            xpb_write(
                self.exp_bar,
                &self.cpp_island,
                self.pa_base_addr + PA_MASK_COMPARE,
                vec![mc_val.0],
                false,
            );
        }

        for (index, mcd_val) in self.mask_compare_detect_units.iter().enumerate() {
            xpb_write(
                self.exp_bar,
                &self.cpp_island,
                self.pa_base_addr + PA_MASK_COMPARE_DETECT[index],
                vec![mcd_val.0],
                false,
            );
        }

        for (index, (config0, config1)) in self.state_transitions.iter().enumerate() {
            xpb_write(
                self.exp_bar,
                &self.cpp_island,
                self.pa_base_addr + PA_TRIGGER_TRANSITION_CONFIG[index][0],
                vec![config0.0],
                false,
            );

            xpb_write(
                self.exp_bar,
                &self.cpp_island,
                self.pa_base_addr + PA_TRIGGER_TRANSITION_CONFIG[index][1],
                vec![config1.0],
                false,
            );
        }

        for (index, tcam_val) in self.tcam_capture_units.iter().enumerate() {
            xpb_write(
                self.exp_bar,
                &self.cpp_island,
                self.pa_base_addr + PA_CAPTURE_TCAM[index],
                vec![tcam_val.0],
                false,
            );
        }
    }

    /// Reads the configuration status of the Performance Analyzer.
    ///
    /// # Returns
    ///
    /// * `PAStatus` - The current status of the Performance Analyzer.
    pub fn read_pa_status(&mut self) -> PAStatus {
        let raw_val = xpb_read(
            self.exp_bar,
            &self.cpp_island,
            self.pa_base_addr + PA_STATUS,
            1,
            false,
        );
        PAStatus(raw_val[0])
    }

    /// Puts the trigger into an idle state from any other state.
    pub fn trigger_idle(&mut self) {
        let mut trigger = PATriggerControl(0);
        trigger.set_active_states(0);
        trigger.set_timeout(0);
        trigger.set_trigger_command(3);
        xpb_write(
            self.exp_bar,
            &self.cpp_island,
            self.pa_base_addr + PA_TRIGGER_CONTROL,
            vec![trigger.0],
            false,
        );
    }

    /// Halt trigger if running.
    pub fn trigger_halt(&mut self) {
        let mut trigger = PATriggerControl(0);
        trigger.set_active_states(0);
        trigger.set_timeout(0);
        trigger.set_trigger_command(2);
        xpb_write(
            self.exp_bar,
            &self.cpp_island,
            self.pa_base_addr + PA_TRIGGER_CONTROL,
            vec![trigger.0],
            false,
        );
    }

    /// Starts the trigger if the Performance Analyzer is idle.
    ///
    /// # Parameters
    ///
    /// * `active_states` - States that need to be currently active for the trigger to start
    ///                     (must be an 8-bit value where each bit location represents a state number).
    /// * `timeout` - Number of cycles to run the trigger before automatically halting.
    ///               If set to 0, the trigger will run indefinitely with no automatic halting.
    ///
    /// # Errors
    ///
    /// This function will `panic!` if the input parameters are out of range.
    ///
    /// # Returns
    ///
    /// A mutable reference to `self`.
    pub fn trigger_start(&mut self, active_states: u8, timeout: u8) {
        let mut trigger = PATriggerControl(0);
        trigger.set_active_states(active_states as u32);
        trigger.set_timeout(timeout as u32);
        trigger.set_trigger_command(1);
        xpb_write(
            self.exp_bar,
            &self.cpp_island,
            self.pa_base_addr + PA_TRIGGER_CONTROL,
            vec![trigger.0],
            false,
        );
    }

    /// Reads a number of 32-bit words from the Performance Analyzer FIFO.
    ///
    /// If the number of words is not specified or zero, all the words in the FIFO are read and returned.
    /// Every time the FifoData CSR is read, the ReadPtr increments until it reaches the value of the WritePtr.
    /// Reading beyond that will keep returning the last read value.
    ///
    /// # Parameters
    ///
    /// * `nfp_bdf` - The PCIe Bus/Device/Function identifier.
    /// * `island` - The island where the PA FIFO is located.
    /// * `num_words` - Number of 32-bit words to read from the Performance Analyzer FIFO. If set to zero,
    ///                 all words in the FIFO are read.
    ///
    /// # Returns
    ///
    /// A `Vec<u32>` containing the words read from the FIFO.
    ///
    /// # Errors
    ///
    /// This function will `panic!` in the following cases:
    /// * If the FIFO buffer is empty.
    /// * If `num_words` exceeds 4096.
    pub fn read_fifo(&mut self, num_words: u32) -> Vec<u32> {
        // Check if num_words exceeds the maximum FIFO size
        if num_words > 4096 {
            panic!("The maximum size of the FIFO is 4096 32-bit words.");
        }

        let mut fifo_words: Vec<u32> = Vec::new();

        // Read FIFO control data
        let fifo_control = PAFifoControl(
            xpb_read(
                self.exp_bar,
                &self.cpp_island,
                self.pa_base_addr + PA_FIFO_CONTROL,
                1,
                false,
            )[0],
        );

        // Determine number of entries in the FIFO
        let entries_in_fifo = if fifo_control.overflow() && !self.pa_configuration.journalling() {
            4096
        } else {
            fifo_control.write_ptr() - fifo_control.read_ptr()
        };

        // Check if the FIFO is empty
        if fifo_control.empty() {
            panic!("FIFO buffer is empty");
        }

        // Adjust the number of words to read
        let words_to_read = if num_words != 0 && num_words < entries_in_fifo {
            num_words
        } else {
            entries_in_fifo
        };

        // Read words from the FIFO
        for _ in 0..words_to_read {
            fifo_words.push(
                xpb_read(
                    self.exp_bar,
                    &self.cpp_island,
                    self.pa_base_addr + PA_FIFO_DATA,
                    1,
                    false,
                )[0],
            );
        }

        fifo_words
    }

    /// Reads the current status of the trigger.
    ///
    /// # Returns
    ///
    /// A `PATriggerStatus` instance that contains the current status of the trigger.
    pub fn read_trigger_status(&mut self) -> PATriggerStatus {
        let raw_val = xpb_read(
            self.exp_bar,
            &self.cpp_island,
            self.pa_base_addr + PA_TRIGGER_STATUS,
            1,
            false,
        );
        PATriggerStatus(raw_val[0])
    }

    /// Retrieves the current value of the Performance Analyzer Timer.
    /// The timer is a counter that increments every cycle.
    ///
    /// # Returns
    ///
    /// The current 32-bit timer value as a `u32`.
    pub fn read_pa_timer(&mut self) -> u32 {
        xpb_read(
            self.exp_bar,
            &self.cpp_island,
            self.pa_base_addr + PA_TIMER,
            1,
            false,
        )[0]
    }

    /// Retrieves the current value of one of the Performance Counters.
    ///
    /// # Parameters
    /// - `counter_num`: Performance counter to read (0-3).
    ///
    /// # Returns
    ///
    /// A mutable reference to self with the current 32-bit counter value.
    ///
    /// # Panics
    ///
    /// This function will panic if `counter_num` is not in the range 0-3.
    pub fn read_perf_counter(&mut self, counter_num: u8) -> u32 {
        if counter_num >= (1 << 2) {
            panic!("counter_num can only be 2 bits maximum.");
        }
        xpb_read(
            self.exp_bar,
            &self.cpp_island,
            self.pa_base_addr + PA_PERFORMANCE_COUNTER[counter_num as usize],
            1,
            false,
        )[0]
    }

    /// Retrieves the current value of one of the Trigger Counters.
    ///
    /// # Parameters
    /// - `counter_num`: Trigger Counter to read (0-1).
    ///
    /// # Returns
    ///
    /// The current 32-bit counter value as `u32`.
    ///
    /// # Panics
    ///
    /// This function will panic if `counter_num` is not in the range 0-1.
    pub fn read_trigger_counter(&mut self, counter_num: u8) -> u32 {
        if counter_num >= (1 << 1) {
            panic!("counter_num can only be 1 bit maximum.");
        }
        xpb_read(
            self.exp_bar,
            &self.cpp_island,
            self.pa_base_addr + PA_TRIGGER_COUNTER[counter_num as usize],
            1,
            false,
        )[0]
    }

    /// Sets the restart value for a Trigger counter.
    ///
    /// # Parameters
    /// - `counter_num`: The Trigger Counter number to set the restart value for (0-1).
    /// - `value`: The restart value for the Trigger Counter as `u32`.
    ///
    /// # Panics
    ///
    /// This function will panic if `counter_num` is not in the range 0-1.
    pub fn set_trigger_counter_restart(&mut self, counter_num: u8, value: u32) {
        if counter_num >= (1 << 1) {
            panic!("counter_num can only be 1 bit maximum.");
        }

        xpb_write(
            self.exp_bar,
            &self.cpp_island,
            self.pa_base_addr + PA_TRIGGER_COUNTER_RESTART[counter_num as usize],
            vec![value],
            false,
        );
    }
}
