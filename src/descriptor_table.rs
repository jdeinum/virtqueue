use crate::descriptor::Descriptor;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};

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

pub type SharedDescriptorTable = Arc<Mutex<DescriptorTable>>;
