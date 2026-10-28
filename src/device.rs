use crate::error::Result;
use crate::shared::{CommonState, SharedCommonState};
use crate::{
    descriptor_table::SharedDescriptorTable,
    rings::{SharedAvailableRing, SharedUsedRing},
};
use std::sync::{Arc, Condvar};

/// Any state carried by the device
#[derive(Debug)]
pub struct DeviceState {
    /// The device should have the address of the shared descriptor table
    descriptor_table: SharedDescriptorTable,

    /// The device should have the address of the shared available ring
    available: SharedAvailableRing,

    /// The device should have the address of the shared used ring
    used: SharedUsedRing,

    /// How far we've read in the available ring
    next_available: usize,

    /// Where we'll write our next completion
    next_used: usize,

    /// Our pretend signaling mechanism to the driver for now
    /// Later on we could replace this with real interrupts.
    driver_condvar: Arc<Condvar>,
}

impl DeviceState {
    pub fn new(
        descriptor_table: SharedDescriptorTable,
        available: SharedAvailableRing,
        used: SharedUsedRing,
        next_available: usize,
        next_used: usize,
        driver_condvar: Arc<Condvar>,
    ) -> Self {
        Self {
            descriptor_table,
            available,
            used,
            next_available,
            next_used,
            driver_condvar,
        }
    }
}

pub struct Device {
    state: DeviceState,
    common_state: SharedCommonState,
}

impl Device {
    pub fn run(mut self) -> Result<()> {
        todo!()
    }

    pub fn new(
        descriptor_table: SharedDescriptorTable,
        available: SharedAvailableRing,
        used: SharedUsedRing,
        next_available: usize,
        next_used: usize,
        driver_condvar: Arc<Condvar>,
        common_state: SharedCommonState,
    ) -> Self {
        Self {
            state: DeviceState::new(
                descriptor_table,
                available,
                used,
                next_available,
                next_used,
                driver_condvar,
            ),
            common_state,
        }
    }
}
