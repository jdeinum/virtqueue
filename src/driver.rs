use crate::{
    descriptor::DescriptorType,
    descriptor_table::{DESC_TABLE_SIZE, DescriptorTable},
    error::Result,
    rings::{AvailableRing, RING_SIZE},
    virtqueue::{SharedVirtQueue, VirtQueue},
};
use rand::{RngExt, SeedableRng, rngs::ChaCha8Rng};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use tracing::debug;

/// Any state private to the driver
#[derive(Debug)]
pub struct DriverState {
    /// The transmit virtqueue (driver -> device)
    transmit_virtqueue: SharedVirtQueue,

    /// The receive virtqueue (device -> driver)
    receive_virtqueue: SharedVirtQueue,

    /// How far the driver has read into the used ring
    last_used_index: usize,

    /// The head of the free chain in the descriptor table
    free_head_index: usize,

    /// Purely an optimization to know how many descriptors are free
    total_available: usize,

    /// Condvar to signal to the device that something is ready
    device_condvar: Arc<Condvar>,

    /// The condvar that the driver will use to signal me
    driver_condvar: Arc<Condvar>,

    /// Our source of RNG for deciding if there is a message to send to the device
    rng: ChaCha8Rng,
}

impl DriverState {
    pub fn new(
        transmit_virtqueue: SharedVirtQueue,
        receive_virtqueue: SharedVirtQueue,
        device_condvar: Arc<Condvar>,
        driver_condvar: Arc<Condvar>,
    ) -> Self {
        Self {
            transmit_virtqueue,
            receive_virtqueue,
            last_used_index: 0,
            free_head_index: 0,
            total_available: DESC_TABLE_SIZE,
            device_condvar,
            driver_condvar,
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
        device_condvar: Arc<Condvar>,
        driver_condvar: Arc<Condvar>,
    ) -> Self {
        Self {
            state: DriverState::new(
                transmit_virtqueue,
                receive_virtqueue,
                device_condvar,
                driver_condvar,
            ),
        }
    }

    fn handle_chain(&self, descriptor_table: &DescriptorTable, starting_index: usize) {
        let mut current = Some(starting_index);
        while let Some(cur) = current {
            let descriptor = descriptor_table[cur];
            debug!("Handling descriptor {descriptor:?}");
            current = descriptor.next;
        }
    }

    fn free_chain(&self, descriptor_table: &mut DescriptorTable, starting_index: usize) {
        todo!()
    }

    /// Take a chain that was used by the device, and return it to the available ring
    fn move_chain_back_to_available(
        &self,
        available_ring: &mut AvailableRing,
        available_index: &mut usize,
        chain_head_index: usize,
    ) {
        todo!()
    }

    /// Run the driver side of the loop, in this case, we have a few responsibilities:
    /// 1. Handle any chains the device has posted to the used ring, and refill the descriptors it
    ///    can use.
    /// 2. Take work that is handed to us and create a chain for it, posting to the available ring

    fn run(mut self) -> Result<()> {
        loop {
            // get the lock to the receive virtqueue
            let mut receive_guard = self.state.receive_virtqueue.lock().unwrap();

            // handle any new chains from the device
            // for now, when freeing them, we just add them directly back to the available ring
            for x in self.state.last_used_index..receive_guard.used_index {
                let slot = x % RING_SIZE;
                self.handle_chain(&mut receive_guard.descriptor_table, slot);
                self.move_chain_back_to_available(
                    &mut receive_guard.available_ring,
                    &mut receive_guard.available_index,
                    slot,
                );
            }

            // next we check if we have some data to send to the device
            //
            // NOTE: We assume work is only created when there are new chains in the used ring. In a
            // real implementation, I expect that we need some sort of signaling mechanism for us
            // to send messages that come externally.
            //
            // TODO: Make data to send come from an external source
            if self.state.rng.random::<u16>() % 10 == 0 {
                debug!("Work to send");
            }

            // This would be the point at which we would post new chains to the available ring if we
            // didn't already move it directly back to the available ring.
        }
    }
}
