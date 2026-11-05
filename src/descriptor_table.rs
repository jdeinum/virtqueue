use crate::descriptor::Descriptor;
use std::ops::{Deref, DerefMut};

pub const DESC_TABLE_SIZE: usize = crate::rings::RING_SIZE;

#[derive(Default, Debug)]
pub struct DescriptorTable([Descriptor; DESC_TABLE_SIZE]);

impl Deref for DescriptorTable {
    type Target = [Descriptor];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DescriptorTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DescriptorTable {
    /// Links every descriptor into one free list, headed by descriptor 0
    pub fn link_free_list(&mut self) {
        for (index, descriptor) in self.iter_mut().enumerate() {
            descriptor.next = (index + 1 < DESC_TABLE_SIZE).then_some(index + 1);
        }
    }

    /// Detaches a chain of `len` descriptors from the front of the free list, returning the head
    /// of the chain and the new head of the free list. Returns `None`, leaving the list untouched,
    /// if there are fewer than `len` free descriptors.
    pub fn take_from_free_list(
        &mut self,
        free_head: Option<usize>,
        len: usize,
    ) -> Option<(usize, Option<usize>)> {
        let head = free_head?;

        let mut tail = head;
        for _ in 1..len {
            tail = self[tail].next?;
        }

        // cutting the link terminates the chain and leaves the rest as the free list
        let new_free_head = self[tail].next.take();
        Some((head, new_free_head))
    }

    pub fn add_to_free_list(&mut self, current_head: Option<usize>, chain_head: usize) -> usize {
        let Some(head) = current_head else {
            // empty free list: the chain becomes the free list
            return chain_head;
        };

        // walk to the tail by index
        let mut tail = head;
        while let Some(next) = self[tail].next {
            tail = next;
        }

        self[tail].next = Some(chain_head);
        head
    }
}
