use crate::virtqueue::SharedVirtQueue;
use crate::{descriptor_table::DescriptorTable, error::Result};
use anyhow::Context;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tracing::debug;

/// How many bytes we pretend to write for each message to the driver. Descriptors don't point at
/// real memory yet, so this is a placeholder.
const MESSAGE_LEN: u32 = 32;

/// The longest we wait for the driver to signal us before checking for work to send
const NOTIFY_TIMEOUT: Duration = Duration::from_millis(1);

/// Any state carried by the device
#[derive(Debug)]
pub struct DeviceState {
    /// The transmit virtqueue (device -> driver)
    transmit_virtqueue: SharedVirtQueue,

    /// The receive virtqueue (driver -> device)
    receive_virtqueue: SharedVirtQueue,

    /// How far we've read in the available ring of the transmit virtqueue
    transmit_next_available: u16,

    /// How far we've read in the available ring of the receive virtqueue
    receive_next_available: u16,

    /// Our pretend signaling mechanism to the driver for now
    /// Later on we could replace this with real interrupts.
    driver_notify: Sender<()>,

    /// How the driver signals us
    device_notify: Receiver<()>,

    /// Our source of RNG for deciding if there is a message to send to the driver
    /// Does not need to be secure, just fast
    rng: SmallRng,
}

impl DeviceState {
    pub fn new(
        transmit_virtqueue: SharedVirtQueue,
        receive_virtqueue: SharedVirtQueue,
        driver_notify: Sender<()>,
        device_notify: Receiver<()>,
    ) -> Self {
        Self {
            transmit_virtqueue,
            receive_virtqueue,
            transmit_next_available: 0,
            receive_next_available: 0,
            driver_notify,
            device_notify,
            rng: SmallRng::seed_from_u64(0),
        }
    }

    /// Writes a message into the next buffer the driver has offered and hands it back through the
    /// used ring, dropping the message if no buffer is available.
    fn send_message(&mut self) -> Result<()> {
        let mut transmit_guard = self.transmit_virtqueue.lock().unwrap();
        let next_chain = transmit_guard
            .available_chains(self.transmit_next_available)
            .next();
        let Some(head) = next_chain else {
            debug!("Dropping message, no buffers available");
            return Ok(());
        };

        handle_chain(&transmit_guard.descriptor_table, head);
        transmit_guard.push_used(head, MESSAGE_LEN);
        self.transmit_next_available = self.transmit_next_available.wrapping_add(1);
        drop(transmit_guard);

        debug!("Sent chain {head} to driver");
        self.driver_notify.send(()).context("notify driver")?;
        Ok(())
    }

    /// Reads every message the driver has posted, and hands each chain back through the used ring.
    /// We only read these chains, so we report 0 bytes written.
    fn receive_messages(&mut self) -> Result<()> {
        let mut receive_guard = self.receive_virtqueue.lock().unwrap();
        let heads: Vec<_> = receive_guard
            .available_chains(self.receive_next_available)
            .collect();

        for &head in &heads {
            handle_chain(&receive_guard.descriptor_table, head);
            receive_guard.push_used(head, 0);
        }

        self.receive_next_available = receive_guard.available_index;
        drop(receive_guard);

        if !heads.is_empty() {
            self.driver_notify.send(()).context("notify driver")?;
        }
        Ok(())
    }

    /// Waits briefly for a signal from the driver, then clears any that have built up. The rings
    /// are the source of truth, so a signal only means it's worth checking them sooner. This must
    /// not block forever, because both sides could end up waiting on each other.
    fn wait_for_notification(&self) {
        let _ = self.device_notify.recv_timeout(NOTIFY_TIMEOUT);
        while self.device_notify.try_recv().is_ok() {}
    }
}

pub struct Device {
    state: DeviceState,
}

impl Device {
    /// Run the device side of the loop, in this case, we have a few responsibilities:
    /// 1. If we have messages to send, we put them in the transmit virtqueue
    /// 2. Check the receive virtqueue to see if there are any messages for us
    pub fn run(mut self) -> Result<()> {
        loop {
            self.step().context("device step")?;
            self.state.wait_for_notification();
        }
    }

    /// Runs a single pass of the device loop without blocking, see [`Device::run`] for the sequence
    pub fn step(&mut self) -> Result<()> {
        // we'll start by checking if we have data to send to the driver
        // normally this would be coming externally, but in this case, we'll generate it through
        // a random source.
        if self.state.rng.random::<u16>() % 10 == 0 {
            debug!("Got message for driver");
            self.state.send_message()?;
        }

        // next we'll check the receive queue to see if there are any messages for us, this is
        // done by checking the available index compared to our next available index.
        self.state.receive_messages()
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
                driver_notify,
                device_notify,
            ),
        }
    }
}

/// Walks a chain that is ready to be handled
fn handle_chain(descriptor_table: &DescriptorTable, starting_index: usize) {
    let mut current = Some(starting_index);
    while let Some(cur) = current {
        let descriptor = descriptor_table[cur];
        debug!("Handling descriptor {descriptor:?}");
        current = descriptor.next;
    }
}
