use crate::{Device, Driver, VirtQueue};
use std::sync::{Arc, Mutex};

pub struct System {
    pub device: Device,
    pub driver: Driver,
}

impl Default for System {
    fn default() -> Self {
        // create our virtqueues and signaling mechanisms
        // NOTE: TX/RX are from the perspective of the driver
        let transmit_virtqueue = Arc::new(Mutex::new(VirtQueue::default()));
        let receive_virtqueue = Arc::new(Mutex::new(VirtQueue::default()));

        let (driver_tx, driver_rx) = std::sync::mpsc::channel();
        let (device_tx, device_rx) = std::sync::mpsc::channel();

        // our driver
        let tx = transmit_virtqueue.clone();
        let rx = receive_virtqueue.clone();
        let driver = Driver::new(tx, rx, device_tx, driver_rx);

        // our device
        let device: Device =
            Device::new(receive_virtqueue, transmit_virtqueue, device_rx, driver_tx);

        Self { device, driver }
    }
}
