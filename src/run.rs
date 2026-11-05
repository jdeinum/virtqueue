use crate::error::Result;
use crate::utils::System;
use anyhow::{Context, anyhow};

/// Spins up a driver and a device connected by a transmit and receive virtqueue, and runs both
/// until they stop.
///
/// # Errors
///
/// Returns an error if the driver or device fails, or if the driver thread panics.
pub fn run() -> Result<()> {
    // create our system
    let System { device, driver } = System::default();

    // spawn the driver
    let driver_handle = std::thread::spawn(move || driver.run().context("run driver"));

    // now we can create our device
    device.run().context("run device")?;
    driver_handle
        .join()
        .map_err(|_| anyhow!("driver thread panicked"))??;
    Ok(())
}
