use crate::{
    descriptor::{DESC_F_WRITE, Descriptor},
    descriptor_table::{DESC_TABLE_SIZE, DescriptorTable},
    error::Result,
    rings::RING_SIZE,
    virtqueue::{SharedVirtQueue, VirtQueue},
};
use anyhow::Context;
use rand::{RngExt, SeedableRng, rngs::SmallRng};
use std::{
    sync::{
        MutexGuard,
        mpsc::{Receiver, Sender},
    },
    time::Duration,
};
use tracing::debug;

/// How many descriptors a message on the transmit virtqueue takes up
const TRANSMIT_CHAIN_LEN: usize = 2;

/// The length of each descriptor in a transmit chain. Descriptors don't point at real memory yet,
/// so this and `RECEIVE_BUFFER_LEN` are placeholders.
const TRANSMIT_DESCRIPTOR_LEN: usize = 32;

/// The length of each buffer we offer the device on the receive virtqueue
const RECEIVE_BUFFER_LEN: usize = 64;

/// The longest we wait for the device to signal us before checking for work to send
const NOTIFY_TIMEOUT: Duration = Duration::from_millis(1);

/// Any state private to the driver
#[derive(Debug)]
pub struct DriverState {
    /// The transmit virtqueue (driver -> device)
    transmit_virtqueue: SharedVirtQueue,

    /// The receive virtqueue (device -> driver)
    receive_virtqueue: SharedVirtQueue,

    /// How far the driver has read into the used ring of the transmit virtqueue
    transmit_last_used: u16,

    /// How far the driver has read into the used ring of the receive virtqueue
    receive_last_used: u16,

    /// The head of the free chain in the transmit descriptor table, `None` when the list is empty.
    /// The receive virtqueue has no free list because its chains are always reposted.
    free_head_index: Option<usize>,

    /// Purely an optimization to know how many descriptors are free
    total_available: usize,

    /// Channel to signal to the device that something is ready
    device_notify: Sender<()>,

    /// The channel that the device will use to signal me
    driver_notify: Receiver<()>,

    /// Our source of RNG for deciding if there is a message to send to the device
    /// Does not need to be secure, just fast
    rng: SmallRng,
}

impl DriverState {
    pub fn new(
        transmit_virtqueue: SharedVirtQueue,
        receive_virtqueue: SharedVirtQueue,
        device_notify: Sender<()>,
        driver_notify: Receiver<()>,
    ) -> Self {
        transmit_virtqueue
            .lock()
            .unwrap()
            .descriptor_table
            .link_free_list();

        // offer every receive buffer up front, otherwise the device has nowhere to write
        let mut receive_guard = receive_virtqueue.lock().unwrap();
        for head in 0..DESC_TABLE_SIZE {
            receive_guard.descriptor_table[head] = Descriptor {
                addr: 0,
                len: RECEIVE_BUFFER_LEN,
                flags: DESC_F_WRITE,
                next: None,
            };
            post_to_available(&mut receive_guard, head);
        }
        drop(receive_guard);

        Self {
            transmit_virtqueue,
            receive_virtqueue,
            transmit_last_used: 0,
            receive_last_used: 0,
            free_head_index: Some(0),
            total_available: DESC_TABLE_SIZE,
            device_notify,
            driver_notify,
            rng: SmallRng::seed_from_u64(1), // Consistent random events
        }
    }

    /// Allocates a chain from the free list and posts it to the transmit virtqueue, dropping the
    /// message if there aren't enough free descriptors.
    fn send_message(&mut self) -> Result<()> {
        if self.total_available < TRANSMIT_CHAIN_LEN {
            debug!(
                "Dropping message, {} descriptors free",
                self.total_available
            );
            return Ok(());
        }

        let mut transmit_guard = self.transmit_virtqueue.lock().unwrap();
        let Some((head, new_free_head)) = transmit_guard
            .descriptor_table
            .take_from_free_list(self.free_head_index, TRANSMIT_CHAIN_LEN)
        else {
            debug!("Dropping message, free list is too short");
            return Ok(());
        };
        self.free_head_index = new_free_head;
        self.total_available -= TRANSMIT_CHAIN_LEN;

        let mut current = Some(head);
        while let Some(index) = current {
            let descriptor = &mut transmit_guard.descriptor_table[index];
            descriptor.len = TRANSMIT_DESCRIPTOR_LEN;
            current = descriptor.next;
        }

        post_to_available(&mut transmit_guard, head);
        drop(transmit_guard);

        debug!("Sent chain {head} to device");
        self.device_notify.send(()).context("notify device")?;
        Ok(())
    }

    /// Returns every chain the device has finished reading to the free list
    fn reclaim_transmit_chains(&mut self) {
        let mut transmit_guard = self.transmit_virtqueue.lock().unwrap();
        let heads: Vec<_> = transmit_guard
            .used_chains(self.transmit_last_used)
            .map(|used| used.id)
            .collect();

        for head in heads {
            self.total_available += handle_chain(&transmit_guard.descriptor_table, head);
            self.free_head_index = Some(
                transmit_guard
                    .descriptor_table
                    .add_to_free_list(self.free_head_index, head),
            );
        }

        self.transmit_last_used = transmit_guard.used_index;
    }

    /// Processes every message the device has written, then reposts its buffer. The lock is held
    /// throughout, and processing comes first so that the device can't overwrite a message that
    /// we haven't read yet.
    fn receive_messages(&mut self) -> Result<()> {
        let mut receive_guard = self.receive_virtqueue.lock().unwrap();
        let used: Vec<_> = receive_guard.used_chains(self.receive_last_used).collect();

        for element in &used {
            debug!("Received {} bytes in chain {}", element.len, element.id);
            handle_chain(&receive_guard.descriptor_table, element.id);
            post_to_available(&mut receive_guard, element.id);
        }

        self.receive_last_used = receive_guard.used_index;
        drop(receive_guard);

        if !used.is_empty() {
            self.device_notify.send(()).context("notify device")?;
        }
        Ok(())
    }

    /// Waits briefly for a signal from the device, then clears any that have built up. The rings
    /// are the source of truth, so a signal only means it's worth checking them sooner. This must
    /// not block forever, because both sides could end up waiting on each other.
    fn wait_for_notification(&self) {
        let _ = self.driver_notify.recv_timeout(NOTIFY_TIMEOUT);
        while self.driver_notify.try_recv().is_ok() {}
    }
}

#[derive(Debug)]
pub struct Driver {
    state: DriverState,
}

impl Driver {
    pub fn new(
        transmit_virtqueue: SharedVirtQueue,
        receive_virtqueue: SharedVirtQueue,
        device_notify: Sender<()>,
        driver_notify: Receiver<()>,
    ) -> Self {
        Self {
            state: DriverState::new(
                transmit_virtqueue,
                receive_virtqueue,
                device_notify,
                driver_notify,
            ),
        }
    }

    /// Run the driver side of the loop, in this case, we have a few responsibilities:
    ///
    /// The transmit queue is for sending messages to the device. The driver needs to grab a set of
    /// descriptors needed for the message. If no descriptors are available, then we drop the
    /// message (for now). Once you have the descriptors, we fill in the data, and then we post it
    /// to the available chain.
    ///
    /// Next, once a device has read our message, it should post that chain to the `used_chain`. We
    /// should check if any chains have been posted so that we can reclaim the descriptors behind
    /// that chain.
    ///
    /// Finally, if the device has a message for us, it will post a chain to the receive virtqueue.
    /// Therefore, we should check the used ring on the receive virtqueue and replace those
    /// descriptors ASAP.
    ///
    /// Thus, we get the following sequence:
    ///
    /// 1. Check if we have messages to send
    /// 2. If (1) is yes, allocate 2 descriptors on the transmit for the message, and send. Drop if 2 aren't available
    /// 3. Check the used ring on the transmit virtqueue to see if the device has read any of our messages.
    /// 4. Check the used ring on the receive virtqueue if the device has any messages for us, if
    ///    so, replace the descriptors as soon as possible.
    ///
    /// # Errors
    ///
    /// Returns an error when sending a message to the device fails
    pub fn run(mut self) -> Result<()> {
        loop {
            self.step().context("driver step")?;
            self.state.wait_for_notification();
        }
    }

    /// Run a single step of our virtqueue
    /// For now, step will be our basic runtime unit we can build around. Tests can call step in
    /// whatever sequence they want, and if we use DST or fuzzing of some kind, we can use a PRNG to
    /// decide what gets to make progress.
    ///
    /// # Errors
    ///
    /// Returns an error when sending a message to the device fails
    pub fn step(&mut self) -> Result<()> {
        // first we'll send any messages we have for the driver by allocating a new chain for
        // it, and putting the index of the head in available.
        if self.state.rng.random::<u16>() % 10 == 0 {
            debug!("Work to send");
            self.state
                .send_message()
                .context("send message to device")?;
        }

        // next we check the transmit virtqueue to see if any buffers need to be replaced
        // this is done by checking the used index and comparing it to the last used index
        self.state.reclaim_transmit_chains();

        // next we check the receive virtqueue to see if there are any messages for us
        // this is done by checking if used index is different to the last used index
        self.state.receive_messages()
    }
}

/// Walks a chain that is ready to be handled, returning the number of descriptors in it
fn handle_chain(descriptor_table: &DescriptorTable, starting_index: usize) -> usize {
    let mut count = 0;
    let mut current = Some(starting_index);
    while let Some(cur) = current {
        let descriptor = descriptor_table[cur];
        debug!("Handling descriptor {descriptor:?}");
        count += 1;
        current = descriptor.next;
    }
    count
}

/// Offers a chain to the device by posting its head to the available ring
fn post_to_available(virt_queue: &mut MutexGuard<VirtQueue>, chain_head_index: usize) {
    let slot = usize::from(virt_queue.available_index) % RING_SIZE;
    virt_queue.available_ring[slot] = chain_head_index;
    virt_queue.available_index = virt_queue.available_index.wrapping_add(1);
}
