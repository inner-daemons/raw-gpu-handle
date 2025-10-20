#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RawWin32Handle {
    inner: isize,
}
impl RawWin32Handle {
    pub fn new(handle: isize) -> Self {
        Self { inner: handle }
    }
    pub fn get(&self) -> isize {
        self.inner
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RawLinuxFd {
    inner: i32,
}
impl RawLinuxFd {
    pub fn new(handle: i32) -> Self {
        Self { inner: handle }
    }
    pub fn get(&self) -> i32 {
        self.inner
    }
}

/// To be figured out at a later date
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RawMetalResourceHandle {
    inner: i64,
}
impl RawMetalResourceHandle {
    pub fn new(handle: i64) -> Self {
        Self { inner: handle }
    }
    pub fn get(&self) -> i64 {
        self.inner
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RawResourceHandle {
    Win32Handle(RawWin32Handle),
    LinuxFd(RawLinuxFd),
    MetalResourceHandle(RawMetalResourceHandle),
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryImportInfo {
    pub raw_handle: RawResourceHandle,
    pub offset: u64,
    pub length: u64,
    pub alignment_guarantees: u64,
    pub is_dedicated_allocation: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SemaphoreImportInfo {
    pub raw_handle: RawResourceHandle,
}

#[derive(Clone, Copy, Debug)]
pub struct BufferImportInfo {
    pub memory: MemoryImportInfo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureType {
    D1,
    D2,
    D2Array,
    D3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureImageTiling {
    /// Optimal for GPU drivers, which might mean e.g. stored in chunks rather than rows for better cache efficiency
    Optimal,
    /// Standard expected image layout, with the exception of some possible padding.
    Linear,
}
#[derive(Clone, Copy, Debug)]
pub struct TextureImportInfo {
    pub memory: MemoryImportInfo,
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
    pub tiling: TextureImageTiling,
}
