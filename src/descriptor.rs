/// Marks a descriptor as device-writable, matching `VIRTQ_DESC_F_WRITE` from the spec
pub const DESC_F_WRITE: u16 = 2;

#[expect(unused, reason = "It's kinda in the spec, but unsued for now")]
pub enum DescriptorType {
    Read,
    Write,
}

/// The flags for a given descriptor
/// The real implementation probably has this as a bitmap. We will leave this as a standard struct
/// for now.
#[expect(unused, reason = "It's kinda in the spec, but unsued for now")]
pub struct Flags {
    pub dtype: DescriptorType,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Descriptor {
    pub addr: u64,
    pub len: usize,
    pub flags: u16,

    /// The next descriptor in the chain
    /// next is an index into the descriptor table
    pub next: Option<usize>,
}
