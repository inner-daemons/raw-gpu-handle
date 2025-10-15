# `raw-gpu-handle`: a GPU API object interoperability library

This library provides standard types for sharing GPU objects across APIs, such as Vulkan, DirectX, Cuda, and Metal. It also provides safety mechanisms to ensure that objects are reconstructed properly across API boundaries. The goal of the library is to ease interoperability between devices within a single process rather than between processes.

Note that sharing `GPU` objects is always unsafe. Lifetime requirements for example cannot be enforced across APIs by this crate. The sender of objects cannot verify that the receiver will behave properly, and vice versa. `raw-gpu-handle` shouldn't be used in security critical applications.

## Abstractions & safety mechanisms
All safety mechanisms are simply rules that other libraries must follow. The `raw-gpu-handle` crate does not itself do any checks.

### General
All reconstructed objects must be reconstructed by the same driver. That means a semaphore created by an integrated device driver cannot be imported into a discrete or virtual device driver.

### Semaphores
Semaphores are the simplest objects. In order to be used properly, the semaphores must only be signalled on the device-side. Additionally, they must be timeline semaphores.

### Memory
Because some objects may share the same memory object, memory is exported separately from objects (this is ignored on metal). Therefore, when buffers or textures are being exported, not only is the memory object needed, but also an offset, size, and alignment guarantees.

The information about memory location must also be shared, e.g. memory heap and type.

### Buffers

### Textures
When reconstructing textures, more information is required. The texture format must match, as well texture type information (D2 vs D2array vs D3). Additionally, information about the memory alignment of rows/slices of textures must be given.

Textures cannot have mip levels, and cannot be multisampled (e.g. `VK_SAMPLE_COUNT_1_BIT = 1`). Textures must have 

### Synchroniation
Aside from semaphores, resource transfers (such as between queue families in vulkan) must be done when sharing data. Image layout transitions must also be done properly.

## Supported platforms

### Windows
Raw objects are shared using `win32` handles. Supported APIs include Vulkan, DirectX, and Cuda.

### Linux
Raw objects are shared using unix file descriptors (`fd`'s). Supported APIs include Vulkan and Cuda.

### MacOS
Raw objects are shared using metal resource handles. Supported APIs include Metal and Vulkan.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.