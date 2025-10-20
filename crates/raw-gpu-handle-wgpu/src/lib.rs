use ash::vk;
use std::ffi::CStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {}

fn required_vulkan_extensions() -> Vec<&'static CStr> {
    let mut required = Vec::new();
    if cfg!(target_vendor = "apple") {
        required.push(ash::ext::metal_objects::NAME);
    } else if cfg!(windows) {
        required.push(ash::khr::external_memory_win32::NAME);
        required.push(ash::khr::external_semaphore_win32::NAME);
    } else if cfg!(target_os = "linux") {
        required.push(ash::khr::external_memory_fd::NAME);
        required.push(ash::khr::external_semaphore_fd::NAME);
    } else {
        required.clear();
    }
    required
}

const fn required_vk_version() -> u32 {
    // Enables the following extensions:
    // * VK_KHR_external_memory
    // * VK_KHR_external_semaphore
    // * VK_KHR_external_memory_capabilities
    // * VK_KHR_dedicated_allocation
    ash::vk::API_VERSION_1_1
}

pub fn adapter_supports_external_resources(adapter: &wgpu::Adapter) -> bool {
    match adapter.get_info().backend {
        wgpu::Backend::Vulkan => unsafe {
            let hal = adapter.as_hal::<wgpu::hal::vulkan::Api>().unwrap();
            let caps = hal.physical_device_capabilities();
            if caps.properties().api_version < required_vk_version() {
                return false;
            }
            let exts = required_vulkan_extensions();
            if exts.is_empty() {
                return false;
            }
            for ext in exts {
                if !caps.supports_extension(ext) {
                    return false;
                }
            }
            // We don't actually need to check the properties. We will always assume that exportable objects require dedicated allocation,
            // and we won't be reexporting imported objects.
            true
        },
        _ => false,
    }
}

pub fn create_device(
    adapter: &wgpu::Adapter,
    desc: &wgpu::DeviceDescriptor,
) -> (wgpu::Device, wgpu::Queue) {
    match adapter.get_info().backend {
        wgpu::Backend::Vulkan => unsafe {
            let hal_device: wgpu::hal::OpenDevice<wgpu::hal::vulkan::Api> = adapter
                .as_hal::<wgpu::hal::vulkan::Api>()
                .unwrap()
                .open_with_callback(
                    desc.required_features,
                    &desc.memory_hints,
                    Some(Box::new(|args| {
                        args.extensions
                            .extend_from_slice(&required_vulkan_extensions());
                    })),
                )
                .unwrap();
            adapter.create_device_from_hal(hal_device, desc).unwrap()
        },
        _ => unreachable!(),
    }
}

pub fn create_and_export_buffer(
    backend: wgpu::Backend,
    device: &wgpu::Device,
    desc: &wgpu::BufferDescriptor,
) -> (wgpu::Buffer, raw_gpu_handle::BufferImportInfo) {
    match backend {
        wgpu::Backend::Vulkan => unsafe {
            let handle_flags = if cfg!(windows) {
                vk::ExternalMemoryHandleTypeFlags::OPAQUE_WIN32
            } else if cfg!(target_os = "linux") {
                vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD
            } else {
                vk::ExternalMemoryHandleTypeFlags::empty()
            };
            let hal = device.as_hal::<wgpu::hal::vulkan::Api>().unwrap();
            let memory_properties = hal
                .shared_instance()
                .raw_instance()
                .get_physical_device_memory_properties(hal.raw_physical_device());
            let memory_type = memory_properties
                .memory_types_as_slice()
                .iter()
                .enumerate()
                .find_map(|a| {
                    if a.1
                        .property_flags
                        .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
                    {
                        Some(a.0)
                    } else {
                        None
                    }
                })
                .unwrap();
            // TODO: map buffer uses
            let mut buffer_usages = desc.usage;
            let mut buffer_uses = wgpu::BufferUses::empty();
            buffer_uses.set(
                wgpu::BufferUses::COPY_SRC,
                buffer_usages.contains(wgpu::BufferUsages::COPY_SRC),
            );
            buffer_uses.set(
                wgpu::BufferUses::COPY_DST,
                buffer_usages.contains(wgpu::BufferUsages::COPY_DST),
            );
            buffer_uses.set(
                wgpu::BufferUses::STORAGE_READ_WRITE,
                buffer_usages.contains(wgpu::BufferUsages::STORAGE),
            );
            buffer_usages.remove(
                wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            );
            if !buffer_usages.is_empty() {
                panic!("Unrecognized remaining buffer usage bits: {buffer_usages:?}");
            }
            let buffer_flags = wgpu::hal::vulkan::conv::map_buffer_usage(buffer_uses);
            let buffer = hal
                .raw_device()
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(desc.size)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE)
                        .usage(buffer_flags),
                    None,
                )
                .unwrap();
            let reqs = hal.raw_device().get_buffer_memory_requirements(buffer);
            let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::default().buffer(buffer);
            let mut export_allocate_info =
                vk::ExportMemoryAllocateInfo::default().handle_types(handle_flags);
            let mut allocate_info = vk::MemoryAllocateInfo::default()
                .allocation_size(reqs.size)
                .memory_type_index(memory_type as u32)
                .push_next(&mut dedicated_info);
            if !cfg!(target_vendor = "apple") {
                allocate_info = allocate_info.push_next(&mut export_allocate_info);
            }
            let memory = hal
                .raw_device()
                .allocate_memory(&allocate_info, None)
                .unwrap();
            hal.raw_device()
                .bind_buffer_memory(buffer, memory, 0)
                .unwrap();
            let import_info = if cfg!(windows) {
                let get_handle_info = vk::MemoryGetWin32HandleInfoKHR::default()
                    .handle_type(handle_flags)
                    .memory(memory);
                let thing = ash::khr::external_memory_win32::Device::new(
                    hal.shared_instance().raw_instance(),
                    hal.raw_device(),
                )
                .get_memory_win32_handle(&get_handle_info)
                .unwrap();
                raw_gpu_handle::RawResourceHandle::Win32Handle(raw_gpu_handle::RawWin32Handle::new(
                    thing,
                ))
            } else if cfg!(target_os = "linux") {
                let get_handle_info = vk::MemoryGetFdInfoKHR::default()
                    .handle_type(handle_flags)
                    .memory(memory);
                let thing = ash::khr::external_memory_fd::Device::new(
                    hal.shared_instance().raw_instance(),
                    hal.raw_device(),
                )
                .get_memory_fd(&get_handle_info)
                .unwrap();
                raw_gpu_handle::RawResourceHandle::LinuxFd(raw_gpu_handle::RawLinuxFd::new(thing))
            } else {
                unreachable!()
            };

            let b = wgpu::hal::vulkan::Buffer::from_raw_managed(buffer, memory, 0, reqs.size);
            let final_buffer = device.create_buffer_from_hal::<wgpu::hal::vulkan::Api>(b, desc);
            let import_info = raw_gpu_handle::BufferImportInfo {
                memory: raw_gpu_handle::MemoryImportInfo {
                    raw_handle: import_info,
                    offset: 0,
                    length: reqs.size,
                    alignment_guarantees: reqs.alignment,
                    is_dedicated_allocation: true,
                },
            };
            (final_buffer, import_info)
        },
        _ => unreachable!(),
    }
}
