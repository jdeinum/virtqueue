use crate::error::Result;
use crate::{device::Device, driver::Driver, virtqueue::VirtQueue};
use anyhow::{Context, anyhow};
use std::sync::{Arc, Mutex};

/// Spins up a driver and a device connected by a transmit and receive virtqueue, and runs both
/// until they stop.
///
/// # Errors
///
/// Returns an error if the driver or device fails, or if the driver thread panics.
pub fn run() -> Result<()> {
    // create our virtqueues and signaling mechanisms
    // NOTE: TX/RX are from the perspective of the driver
    let transmit_virtqueue = Arc::new(Mutex::new(VirtQueue::default()));
    let receive_virtqueue = Arc::new(Mutex::new(VirtQueue::default()));

    let (driver_tx, driver_rx) = std::sync::mpsc::channel();
    let (device_tx, device_rx) = std::sync::mpsc::channel();

    // spawn our driver
    let tx = transmit_virtqueue.clone();
    let rx = receive_virtqueue.clone();
    let driver_handle = std::thread::spawn(move || {
        let driver: Driver = Driver::new(tx, rx, device_tx, driver_rx);
        driver.run().context("run driver")
    });

    // now we can create our device
    let device: Device = Device::new(receive_virtqueue, transmit_virtqueue, device_rx, driver_tx);
    device.run().context("run device")?;
    driver_handle
        .join()
        .map_err(|_| anyhow!("driver thread panicked"))??;
    Ok(())
}
