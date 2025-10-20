use ash::vk;
use std::ffi::CStr;
use thiserror::Error;
use wgpu::hal::Device;

#[derive(Error, Debug)]
pub enum Error {}

fn required_vulkan_extensions() -> Vec<&'static CStr> {
    let mut required = Vec::new();
    if cfg!(target_vendor = "apple") {
        required.push(ash::ext::metal_objects::NAME);
    }
    if cfg!(windows) {
        required.push(ash::khr::external_memory_win32::NAME);
        required.push(ash::khr::external_semaphore_win32::NAME);
    }
    if cfg!(target_os = "linux") {
        required.push(ash::khr::external_memory_fd::NAME);
        required.push(ash::khr::external_semaphore_fd::NAME);
    }
    required
}

fn required_vk_version() -> u32 {
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
            for ext in required_vulkan_extensions() {
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
            let hal = device.as_hal::<wgpu::hal::vulkan::Api>().unwrap();
            // Wait or wgpu to do this
            let buffer_usages = unimplemented!();
            let buffer = hal
                .raw_device()
                .create_buffer(
                    &vk::BufferCreateInfo::default()
                        .size(desc.size)
                        .sharing_mode(vk::SharingMode::EXCLUSIVE)
                        .usage(buffer_usages),
                    None,
                )
                .unwrap();
            let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::default().buffer(buffer);
        },
        _ => unreachable!(),
    }
    todo!()
}
