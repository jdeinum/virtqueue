pub enum DescriptorType {
    Read,
    Write,
}

/// The flags for a given descriptor
/// The real implementation probably has this as a bitmap. We will leave this as a standard struct
/// for now.
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
