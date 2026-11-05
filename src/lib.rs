//! Manual implementation of a virtqueue.
//!
//! 2026-10-28
//!
//! My initial understand of a virtqueue was that there was a
//! single virtqueue for both transmitting (driver -> device) and receiving (device -> driver).
//! After reading the spec (5.1.2) I see that in multiqueue mode, each virtio device can have
//! up to X (receive, transmit) pairs. With multiqueue mode disabled, there is 1 receive and 1
//! transmit queue, and optionally a control queue if the `VIRTIO_NET_F_CTRL_VQ` is set.
//!
//! From what I understand, the virtqueues that are used are agreed up during the initial
//! negotiation between driver and device. I am currently kind of mocking that with shared memory at
//! startup, which I think is sufficient for this exercise.
//!
//! Another misconception I had was that the buffers are explicitly sized (i.e. each descriptor had
//! an underlying buffer of size N). The buffers themselves are not sized, and instead point into
//! the VMM virtual memory space with a specific size.
//!
//! 2026-11-05
//!
//! One thing I wasn't confident about was why we would ever split up a single payload among
//! multiple descriptors. If you already have the data in one contiguous memory block, isn't
//! splitting it almost always strictly worse? There are 2 things I didn't realize: First, the data
//! is _not_ continuous, a packet header and it's body may already live in different locations.
//! Either the driver needs to copy them into one buffer which is wasteful, or we need to support
//! splitting, which virtio obviously does. Second, no descriptor can be both readable and writable.
//! Thus, if we want to request something from the device, we send a message with the first
//! descriptor being a read while the second is writable and allows the device to send its response.
//!
//! Also, descriptors hold guest physical addresses, unless you are using a virtual IOMMU
//! (negotiated through VIRTIO_F_ACCESS_PLATFORM), in which case the descriptors contain virtual
//! addresses that need to be mapped through the IOMMU. For now, I will ignore IOMMU and just assume
//! the addresses are always physical.

mod descriptor;
mod descriptor_table;
mod device;
mod driver;
mod error;
mod rings;
mod run;
mod utils;
mod virtqueue;

pub use device::Device;
pub use driver::Driver;
pub use run::run;
pub use virtqueue::{SharedVirtQueue, VirtQueue};
