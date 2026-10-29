use crate::{
    descriptor_table::{DESC_TABLE_SIZE, DescriptorTable},
    error::Result,
    rings::RING_SIZE,
    virtqueue::{SharedVirtQueue, VirtQueue},
};
use rand::{RngExt, SeedableRng, rngs::ChaCha8Rng};
use std::sync::{
    MutexGuard,
    mpsc::{Receiver, Sender},
};
use tracing::debug;

/// Any state private to the driver
#[derive(Debug)]
pub struct DriverState {
    /// The transmit virtqueue (driver -> device)
    transmit_virtqueue: SharedVirtQueue,

    /// The receive virtqueue (device -> driver)
    receive_virtqueue: SharedVirtQueue,

    /// How far the driver has read into the used ring
    last_used_index: u16,

    /// The head of the free chain in the descriptor table
    free_head_index: usize,

    /// Purely an optimization to know how many descriptors are free
    total_available: usize,

    /// Channel to signal to the device that something is ready
    device_notify: Sender<()>,

    /// The channel that the device will use to signal me
    driver_notify: Receiver<()>,

    /// Our source of RNG for deciding if there is a message to send to the device
    rng: ChaCha8Rng,
}

impl DriverState {
    pub fn new(
        transmit_virtqueue: SharedVirtQueue,
        receive_virtqueue: SharedVirtQueue,
        device_notify: Sender<()>,
        driver_notify: Receiver<()>,
    ) -> Self {
        Self {
            transmit_virtqueue,
            receive_virtqueue,
            last_used_index: 0,
            free_head_index: 0,
            total_available: DESC_TABLE_SIZE,
            device_notify,
            driver_notify,
            rng: ChaCha8Rng::from_seed([0; 32]), // Consistent random events
        }
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
    /// 1. Handle any chains the device has posted to the used ring, and refill the descriptors it
    ///    can use.
    /// 2. Take work that is handed to us and create a chain for it, posting to the available ring
    pub fn run(mut self) -> Result<()> {
        loop {
            // first we'll send any messages we have for the driver by allocating a new chain for
            // it, and putting the index of the head in available.
            if self.state.rng.random::<u16>() % 10 == 0 {
                debug!("Work to send");
            }

            // next we check the receive virtqueue to see if there are any messages for us
            // this is done by checking if used index is greater than the last used index

            // next we check the transmit virtqueue to see if any buffers need to be replaced
            // this is done by checking the used index and comparing it to the last used index
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

fn free_chain(descriptor_table: &mut DescriptorTable, starting_index: usize) {
    todo!()
}

/// Take a chain that was used by the device, and return it to the available ring
fn move_chain_back_to_available(virt_queue: &mut MutexGuard<VirtQueue>, chain_head_index: usize) {
    let slot = usize::from(virt_queue.available_index) % RING_SIZE;
    virt_queue.available_ring[slot] = chain_head_index;
    virt_queue.available_index = virt_queue.available_index.wrapping_add(1);
}
