use std::sync::{Arc, Mutex};

/// Any state that is shared between the device and the driver
///
/// FIXME: Probably should put the shared table and rings in here too? Only issue might be that we
/// are locking all of these things with one lock, not ideal.
#[derive(Default, Debug)]
pub struct CommonState {
    /// The total number of chains posted by the driver
    pub available_index: usize,

    /// The total number of chains completed by the device
    pub used_index: usize,
}

pub type SharedCommonState = Arc<Mutex<CommonState>>;
