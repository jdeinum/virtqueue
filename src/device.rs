use crate::virtqueue::SharedVirtQueue;
use crate::{descriptor_table::DescriptorTable, error::Result};
use rand::rngs::ChaCha8Rng;
use rand::{RngExt, SeedableRng};
use std::sync::mpsc::{Receiver, Sender};
use tracing::debug;

/// Any state carried by the device
#[derive(Debug)]
pub struct DeviceState {
    /// The transmit virtqueue (device -> driver)
    transmit_virtqueue: SharedVirtQueue,

    /// The receive virtqueue (driver -> device)
    receive_virtqueue: SharedVirtQueue,

    /// How far we've read in the available ring
    next_available: u16,

    /// Where we'll write our next completion
    next_used: u16,

    /// Our pretend signaling mechanism to the driver for now
    /// Later on we could replace this with real interrupts.
    driver_notify: Sender<()>,

    /// How the driver signals us
    device_notify: Receiver<()>,

    /// Our source of RNG for deciding if there is a message to send to the driver
    rng: ChaCha8Rng,
}

impl DeviceState {
    pub fn new(
        transmit_virtqueue: SharedVirtQueue,
        receive_virtqueue: SharedVirtQueue,
        next_available: u16,
        next_used: u16,
        driver_notify: Sender<()>,
        device_notify: Receiver<()>,
    ) -> Self {
        Self {
            transmit_virtqueue,
            receive_virtqueue,
            next_available,
            next_used,
            driver_notify,
            device_notify,
            rng: ChaCha8Rng::from_seed([1; 32]),
        }
    }
}

pub struct Device {
    state: DeviceState,
}

impl Device {
    /// Run the device side of the loop, in this case, we have a few responsibilities:
    /// 1. Check the receive virtqueue to see if there are any messages for us
    /// 2. If we have messages to send, we put them in the transmit virtqueue
    pub fn run(mut self) -> Result<()> {
        loop {
            // we'll start by checking if we have data to send to the driver
            // normally this would be coming externally, but in this case, we'll generate it through
            // a random source.
            if self.state.rng.random::<u16>() % 10 == 0 {
                debug!("Got message for driver");
            }

            // next we'll check the receive queue to see if there are any messages for us, this is
            // done by checking the available index compared to our next available index.
            let receive_virtqueue = self.state.receive_virtqueue.lock().unwrap();
            for head in receive_virtqueue.available_chains(self.state.next_available) {
                handle_chain(&receive_virtqueue.descriptor_table, head);
            }
            self.state.next_available = receive_virtqueue.available_index;
        }
    }

    pub fn new(
        transmit_virtqueue: SharedVirtQueue,
        receive_virtqueue: SharedVirtQueue,
        device_notify: Receiver<()>,
        driver_notify: Sender<()>,
    ) -> Self {
        Self {
            state: DeviceState::new(
                transmit_virtqueue,
                receive_virtqueue,
                0,
                0,
                driver_notify,
                device_notify,
            ),
        }
    }
}

/// Handle a chain that
fn handle_chain(descriptor_table: &DescriptorTable, starting_index: usize) {
    let mut current = Some(starting_index);
    while let Some(cur) = current {
        let descriptor = descriptor_table[cur];
        debug!("Handling descriptor {descriptor:?}");
        current = descriptor.next;
    }
}
