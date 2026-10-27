# Virtqueue

One of the key milestones early in my research will be building a DST harness
for para-virtualized devices. These devices make heavy use of virtqueues for
performance. Naturally, before building the harness, I want to make sure I
understand how these queues work so I can understand where the non-determinism
lives. This repo is my hand written attempt to make a virtqueue from scratch
without looking at any other sources.

## Goal

I am not trying to write a robust production ready virtqueue, the virtqueue I'll
be building the harness for already exists under
[rustvmm](https://github.com/rust-vmm/rust-vmm/tree/main/vm-virtio). My goal is
to write enough of a virtqueue according to the
[spec](https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html#x1-90002)
that I feel convertable converting the existing rustvmm implementation to use [loom](<>) primitives to
check for concurrency issues. Once this is done, I'll write a DST harness for the rustvmm virtio.
