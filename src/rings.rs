use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

/// The spec requires a power of two for split rings, which is what lets the free-running u16
/// indices map onto ring slots with `% RING_SIZE` across a wrap.
pub const RING_SIZE: usize = 8;
const _: () = assert!(RING_SIZE.is_power_of_two());

/// The available ring is the mechanism which the driver can send data to the device. The driver places the
/// index of a descriptor slot into the available ring,
#[derive(Default, Debug)]
pub struct AvailableRing([usize; RING_SIZE]);

impl Deref for AvailableRing {
    type Target = [usize];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AvailableRing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A single entry in the used ring, matching `virtq_used_elem` from the spec
#[derive(Default, Debug, Copy, Clone)]
pub struct UsedElem {
    /// The head descriptor index of the chain the device has finished with
    pub id: usize,

    /// The number of bytes the device wrote into the device-writable part of the chain. Zero for
    /// chains the device only read from, e.g. on the transmit queue.
    pub len: u32,
}

/// The used ring is the mechanism which the device hands chains back to the driver once it is done
/// with them.
#[derive(Default, Debug)]
pub struct UsedRing([UsedElem; RING_SIZE]);

impl Deref for UsedRing {
    type Target = [UsedElem];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UsedRing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub type SharedAvailableRing = Arc<Mutex<AvailableRing>>;

pub type SharedUsedRing = Arc<Mutex<UsedRing>>;
