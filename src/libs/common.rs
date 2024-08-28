use std::fs;
use std::num::ParseIntError;
use std::path::Path;
use std::path::PathBuf;

/// Validates a PCIe Bus/Device/Function (BDF) identifier for a Merlin NFP device.
///
/// This function checks if the provided BDF is formatted correctly and corresponds
/// to a valid PCIe device in the system. If the BDF is missing the domain part,
/// it automatically adds "0000:" as a prefix. The function also reads the vendor
/// and device IDs to confirm that the device is a Merlin NFP.
///
/// # Parameters
///
/// * `pci_bdf`: A string slice representing the PCIe BDF identifier to validate.
///
/// # Returns
///
/// Returns `Ok(String)` containing the formatted BDF if it is valid,
/// or `Err(String)` with an error message if the BDF is invalid or does not correspond
/// to a Merlin NFP device.
///
/// # Errors
///
/// The function can return errors for the following reasons:
/// - The specified PCIe device does not exist.
/// - Failed to read the vendor or device ID.
/// - The vendor or device ID does not match that of a Merlin NFP
pub fn validate_nfp_bdf(pci_bdf: &str) -> Result<String, String> {
    // If the BDF is missing the domain part, add "0000:" as a prefix
    let pci_bdf = if pci_bdf.split(':').count() < 3 {
        format!("0000:{}", pci_bdf)
    } else {
        pci_bdf.to_string()
    };

    // Construct the path to the PCIe device in the sysfs
    let str_path = format!("/sys/bus/pci/devices/{}", pci_bdf);
    let base_path = Path::new(&str_path);
    if !base_path.exists() {
        return Err(format!("No such PCIe device: {}", pci_bdf));
    }

    // Read the vendor ID
    let mut vendor_path = PathBuf::from(base_path);
    vendor_path.push("vendor");
    let vendor_id = match fs::read_to_string(&vendor_path) {
        Ok(contents) => contents,
        Err(_) => return Err(format!("Failed to read vendor ID for device: {}", pci_bdf)),
    };

    // Read the device ID
    let mut device_path = PathBuf::from(base_path);
    device_path.push("device");
    let device_id = match fs::read_to_string(&device_path) {
        Ok(contents) => contents,
        Err(_) => return Err(format!("Failed to read device ID for device: {}", pci_bdf)),
    };

    // Validate the vendor and device IDs for a Merlin NFP device
    if vendor_id.trim() != "0x1da8" || device_id.trim() != "0x7000" {
        return Err(format!(
            "PCIe BDF {} does not belong to a Merlin NFP.",
            pci_bdf
        ));
    }

    // If everything is valid, return the formatted PCI BDF
    Ok(pci_bdf)
}

/// Splits a 48-bit address into a base address and an offset.
///
/// This function takes a 48-bit address and an aperture value, which specifies the
/// alignment in terms of powers of two. It calculates the base address by masking
/// the original address and computes the offset from the base address.
///
/// # Parameters
///
/// * `address`: The 48-bit address to be split.
/// * `aperture`: The aperture value, specified as a power of 2.
///
/// # Returns
///
/// Returns a tuple containing:
/// - The base address aligned to the specified aperture.
/// - The offset from the base address
pub fn split_addr48(address: u64, aperture: u64) -> (u64, u64) {
    // Ensure aperture is a power of 2
    let aperture = 1 << (64 - aperture.leading_zeros() - 1);
    // Mask to get the base address
    let base_address = address & (0xFFFFFFFFFFFFu64 - (aperture - 1));
    // Compute the offset
    let offset = address - base_address;
    (base_address, offset)
}

/// Parses a string representation of a hexadecimal or decimal number.
///
/// This function attempts to parse the input string as a hexadecimal number if it
/// starts with "0x" or "0X". If it does not, it tries to parse it as a decimal
/// integer.
///
/// # Parameters
///
/// * `s`: A string slice containing the number to be parsed.
///
/// # Returns
///
/// Returns `Ok(u32)` if the parsing is successful, or an error of type `ParseIntError`
/// if the string cannot be parsed as a valid integer.
pub fn hex_parser(s: &str) -> Result<u32, ParseIntError> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16)
    } else {
        s.parse::<u32>()
    }
}
