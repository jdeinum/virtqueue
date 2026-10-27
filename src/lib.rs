mod error;

use anyhow::Result;
use std::sync::{Arc, Mutex};

pub const DESC_TABLE_SIZE: usize = 10;
pub const RING_SIZE: usize = 10;

#[derive(Default, Debug, Copy, Clone)]
pub struct Descriptor {
    pub addr: u64,
    pub len: usize,
    pub flags: u16,

    /// The next descriptor in the chain
    /// next is an index into the descriptor table
    pub next: Option<usize>,
}

#[derive(Debug)]
pub struct DescriptorTable {
    entries: [Descriptor; DESC_TABLE_SIZE],
    free_head_index: usize,
    total_available: usize,
}

impl Default for DescriptorTable {
    fn default() -> Self {
        let mut entries: [Descriptor; DESC_TABLE_SIZE] = (0..DESC_TABLE_SIZE)
            .map(|x| Descriptor {
                next: Some(x + 1),
                ..Descriptor::default()
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        // the last entry should have its next field cleared
        entries.iter_mut().last().map(|x| x.next = None);

        Self {
            entries,
            free_head_index: 0,
            total_available: DESC_TABLE_SIZE,
        }
    }
}

impl DescriptorTable {
    // get the number of available entries
    pub fn available(&self) -> usize {
        self.total_available
    }

    /// get a chain of descriptors for some IO request
    pub fn get_chain(&mut self, num_descriptors: usize) -> Result<()> {

        // if the number of
    }
}

pub type AvailableRing = [usize; RING_SIZE];

pub type SharedAvailableRing = Arc<Mutex<AvailableRing>>;

pub type UsedRing = [usize; RING_SIZE];

pub type SharedUsedRing = Arc<Mutex<UsedRing>>;

pub struct Device {}

impl Device {
    pub fn run(
        mut self,
        available_ring: SharedAvailableRing,
        used_ring: SharedUsedRing,
    ) -> Result<()> {
        todo!()
    }
}

pub fn run() -> Result<()> {
    // define our structures
    // the table is local state to the driver, while the rings are shared between the driver and the
    // device. Therefore, we'll add standard synchronization on the rings for now.
    let mut table: DescriptorTable = DescriptorTable::default();
    let available_ring = Arc::new(Mutex::new([0; RING_SIZE]));
    let used_ring = Arc::new(Mutex::new([0; RING_SIZE]));

    // the indexes are used for us to keep track of the next slot in the available ring to
    let mut available_index = 0;
    let mut used_index = 0;

    // we also define our tracking for our free list, because the descriptor table already encodes
    // the chain structure through the `next` field, we just need a head

    // now we can spawn our mock device that grabs from the available_ring
    let handle = std::thread::spawn(move || {
        let device = Device {};
        let a = available_ring.clone();
        let u = used_ring.clone();
        device.run(a, u)
    });

    // now we can handle our IO requests and such
    loop {

        //
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
