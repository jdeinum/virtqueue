use crate::{
    descriptor_table::DescriptorTable,
    rings::{AvailableRing, RING_SIZE, UsedElem, UsedRing},
};
use std::sync::{Arc, Mutex};

/// A single virtqueue containing the shared state both sides needs access to
#[derive(Debug, Default)]
pub struct VirtQueue {
    /// The descriptor table for this virtqueue
    pub descriptor_table: DescriptorTable,

    /// The available ring the driver posts to
    pub available_ring: AvailableRing,

    /// The used ring the device posts to
    pub used_ring: UsedRing,

    /// The current position of the available ring
    /// Incremented by the driver, which the device sees and interprets as work to be done.
    /// Like the spec, this is a free-running u16 that wraps; the ring slot is `idx % RING_SIZE`.
    pub available_index: u16,

    /// The current position of the used ring
    /// Incremented by the device, which the driver sees and interprets as chains to free.
    /// Free-running and wrapping, same as `available_index`.
    pub used_index: u16,
}

impl VirtQueue {
    /// Returns the head descriptor index of every chain posted to the available ring since
    /// `available_next_index`
    pub fn available_chains(&self, available_next_index: u16) -> impl Iterator<Item = usize> {
        ring_entries(
            &self.available_ring,
            available_next_index,
            self.available_index,
        )
    }

    /// Returns every entry the device has posted to the used ring since `used_next_index`
    pub fn used_chains(&self, used_next_index: u16) -> impl Iterator<Item = UsedElem> {
        ring_entries(&self.used_ring, used_next_index, self.used_index)
    }

    /// Hands a chain back to the driver, recording how many bytes the device wrote into it
    pub fn push_used(&mut self, id: usize, len: u32) {
        let slot = usize::from(self.used_index) % RING_SIZE;
        self.used_ring[slot] = UsedElem { id, len };
        self.used_index = self.used_index.wrapping_add(1);
    }
}

/// Walks the ring entries between `from` and `to`. The difference is computed with wrapping
/// arithmetic so this keeps working after the u16 index overflows.
fn ring_entries<T: Copy>(ring: &[T], from: u16, to: u16) -> impl Iterator<Item = T> + '_ {
    (0..to.wrapping_sub(from)).map(move |i| ring[usize::from(from.wrapping_add(i)) % RING_SIZE])
}

pub type SharedVirtQueue = Arc<Mutex<VirtQueue>>;
