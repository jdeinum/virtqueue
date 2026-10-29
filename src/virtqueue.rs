use crate::{
    descriptor_table::DescriptorTable,
    rings::{AvailableRing, UsedRing},
};
use std::sync::{Arc, Mutex};

/// A single virtqueue containing the shared state both sides needs access to
#[derive(Debug)]
pub struct VirtQueue {
    /// The descriptor table for this virtqueue
    pub descriptor_table: DescriptorTable,

    /// The available ring the driver posts to
    pub available_ring: AvailableRing,

    /// The used ring the device posts to
    pub used_ring: UsedRing,

    /// The current position of the available ring
    /// Incremented by the driver, which the device sees and interprets as work to be done
    pub available_index: usize,

    /// The current position of the used ring
    /// Incremented by the device, which the driver sees and interprets as chains to free
    pub used_index: usize,
}

pub type SharedVirtQueue = Arc<Mutex<VirtQueue>>;
