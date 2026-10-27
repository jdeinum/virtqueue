mod error;

use crate::error::{Error, ErrorRepr, Result};
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
    free_head_index: Option<usize>,
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
        if let Some(x) = entries.iter_mut().last() {
            x.next = None
        }
        Self {
            entries,
            free_head_index: Some(0),
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
    /// TODO: I think getting a set of descriptors should return a mutable reference to those descriptors?
    pub fn get_chain(&mut self, num_descriptors: usize) -> Result<Vec<u64>> {
        // if the number of needed descriptors is larger than the table size, we'll never be able to satisfy it
        if num_descriptors > DESC_TABLE_SIZE {
            return Err(Error::from(ErrorRepr::RequestTooLarge(
                num_descriptors as u64,
            )));
        }

        // if the number of available descriptors is too small, we should probably block on some
        // sort of condvar? For now, we just panic.
        // FIXME: Don't panic :)
        if num_descriptors > self.total_available {
            unimplemented!("Cannot handle requests that are larger than the system just yet");
        }
        // if we reach this point, we should be able to satisfy the request
        else {
            // get the chain by walking the free list until we have enough descriptors
            let mut current = self.free_head_index;
            let mut chain: Vec<u64> = Vec::new();
            for _ in 0..num_descriptors {
                chain.push(current.unwrap() as u64);
                current = self.entries[current.unwrap()].next;
            }

            // update the new current head position
            self.free_head_index = current;

            // update the total available
            self.total_available -= num_descriptors;

            Ok(chain)
        }
    }
}

pub type AvailableRing = [usize; RING_SIZE];

pub type SharedAvailableRing = Arc<Mutex<AvailableRing>>;

pub type UsedRing = [usize; RING_SIZE];

pub type SharedUsedRing = Arc<Mutex<UsedRing>>;

pub struct Device {}

impl Device {
    pub fn run(
        self,
        _available_ring: SharedAvailableRing,
        _used_ring: SharedUsedRing,
    ) -> Result<()> {
        todo!()
    }
}

pub fn run() -> Result<()> {
    // define our structures
    // the table is local state to the driver, while the rings are shared between the driver and the
    // device. Therefore, we'll add standard synchronization on the rings for now.
    let _table: DescriptorTable = DescriptorTable::default();
    let available_ring = Arc::new(Mutex::new([0; RING_SIZE]));
    let used_ring = Arc::new(Mutex::new([0; RING_SIZE]));

    // the indexes are used for us to keep track of the next slot in the available ring to
    let _available_index = 0;
    let _used_index = 0;

    // we also define our tracking for our free list, because the descriptor table already encodes
    // the chain structure through the `next` field, we just need a head

    // now we can spawn our mock device that grabs from the available_ring
    let _handle = std::thread::spawn(move || {
        let device = Device {};
        let a = available_ring.clone();
        let u = used_ring.clone();
        device.run(a, u)
    });

    // now we can handle our IO requests and such
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::debug;

    pub(crate) fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn requesting_all_leaves_no_slots() {
        init_tracing();

        let mut desc_table = DescriptorTable::default();

        debug!("{desc_table:?}");

        // quick sanity check
        assert_eq!(desc_table.available(), DESC_TABLE_SIZE);

        // request all the descriptors
        let x = desc_table.get_chain(DESC_TABLE_SIZE).unwrap();

        debug!("chain: {x:?}");

        // available should be 0 now
        assert_eq!(desc_table.available(), 0);
    }

    #[test]
    fn requesting_two_valid() {
        init_tracing();

        let mut desc_table = DescriptorTable::default();

        // request 1 descriptor
        // NOTE: This assumes the table has at least 1 slot, and we don't assert this anywhere. The
        // spec has defined max elements to be 2^16 if I believe, so this should be safe.
        let x = desc_table.get_chain(1).unwrap();

        debug!("chain of 1: {x:?}");

        // request all remaining
        let rest = desc_table.get_chain(desc_table.available());

        debug!("rest: {rest:?}");

        // should be nothing left
        assert_eq!(desc_table.available(), 0);
    }
}
