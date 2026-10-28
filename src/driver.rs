use crate::{
    descriptor::DescriptorType,
    descriptor_table::{DESC_TABLE_SIZE, SharedDescriptorTable},
    error::Result,
    rings::{SharedAvailableRing, SharedUsedRing},
    shared::SharedCommonState,
};
use std::sync::{Arc, Condvar};

/// Any state private to the driver
#[derive(Debug)]
pub struct DriverState {
    /// The driver should have the address of the shared descriptor table
    descriptor_table: SharedDescriptorTable,

    /// The driver should have the address of the shared available ring
    available: SharedAvailableRing,

    /// The driver should have the address of the shared used ring
    used: SharedUsedRing,

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
}

impl DriverState {
    pub fn new(
        descriptor_table: SharedDescriptorTable,
        available: SharedAvailableRing,
        used: SharedUsedRing,
        device_condvar: Arc<Condvar>,
        driver_condvar: Arc<Condvar>,
    ) -> Self {
        Self {
            descriptor_table,
            available,
            used,
            last_used_index: 0,
            free_head_index: 0,
            total_available: DESC_TABLE_SIZE,
            device_condvar,
            driver_condvar,
        }
    }
}

#[derive(Debug)]
pub struct Driver {
    state: DriverState,
    common_state: SharedCommonState,
}

impl Driver {
    pub fn new(
        descriptor_table: SharedDescriptorTable,
        available: SharedAvailableRing,
        used: SharedUsedRing,
        common_state: SharedCommonState,
        device_condvar: Arc<Condvar>,
        driver_condvar: Arc<Condvar>,
    ) -> Self {
        Self {
            state: DriverState::new(
                descriptor_table,
                available,
                used,
                device_condvar,
                driver_condvar,
            ),
            common_state,
        }
    }

    /// Adds a new chain to the available queue
    ///
    /// NOTE: I believe that thinking in descriptors is incorrect, as they may have different sizes?
    /// Worth checking the spec.
    ///
    /// NOTE: I believe the size of chains we post depend on the device, as if all the descriptors
    /// provide X bytes but the device needs 2X bytes, then it can't really do anything (ignoring
    /// merging).
    fn post_chain(&self, num_descriptors: usize, mode: DescriptorType) -> Result<()> {
        todo!()
    }

    /// Run the driver side of the loop, in this case, we have a few responsibilities:
    /// 1. Post information we want to send using descriptors in device read mode
    /// 2. Post some number of chains that are device writable so it can send data to us
    /// 3. Reclaim any buffers in the used chain and add them to the free list
    fn run(mut self) -> Result<()> {
        loop {
            // get the lock to the shared state
            let guard = self.common_state.lock().unwrap();

            // if the used index >= last_used_index, then there are chains that can be reclaimed
            if guard.used_index > self.state.last_used_index {
                todo!()
            }

            self.state.driver_condvar.wait(guard)
        }
    }
}
