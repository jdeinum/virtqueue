//! Manual implementation of a virtqueue.
//!
//! 2026-10-28
//!
//! My initial understand of a virtqueue was that there was a
//! single virtqueue for both transmitting (driver -> device) and receiving (device -> driver).
//! After reading the spec (5.1.2) I see that in multi queue mode, each virtio device can have
//! up to X (receive, transmit) pairs. With multiqueue mode disabled, there is 1 receive and 1
//! transmit queue, and optionally a control queue if the `VIRTIO_NET_F_CTRL_VQ` is set.
//!
//! From what I understand, the virtqueues that are used are agreed up during the initial
//! negotiation between driver and device. I am currently kind of mocking that with shared memory at
//! startup, which I think is sufficient for this exercise.
//!
//! Another misconception I had was that the buffers are explicitly sized (i.e each descriptor had
//! an underlying buffer of size N). The buffers themselves are not sized, and instead point into
//! the VMM virtual memory space with a specific size.

mod descriptor;
mod descriptor_table;
mod device;
mod driver;
mod error;
mod rings;
pub mod run;
mod virtqueue;
