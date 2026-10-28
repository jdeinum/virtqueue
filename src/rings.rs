use std::sync::{Arc, Mutex};

pub const RING_SIZE: usize = 10;

/// The available ring is the mechanism which the driver can send data to the device. The driver places the
/// index of a descriptor slot into the available ring,
#[derive(Default, Debug)]
pub struct AvailableRing([usize; RING_SIZE]);

#[derive(Default, Debug)]
pub struct UsedRing([usize; RING_SIZE]);

pub type SharedAvailableRing = Arc<Mutex<AvailableRing>>;

pub type SharedUsedRing = Arc<Mutex<UsedRing>>;
