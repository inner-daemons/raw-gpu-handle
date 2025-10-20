#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: wgpu::InstanceFlags::advanced_debugging(),
        ..Default::default()
    });
    let adapters = instance.enumerate_adapters(wgpu::Backends::all()).await;
    let adapter = adapters
        .iter()
        .find(|a| raw_gpu_handle_wgpu::adapter_supports_external_resources(a))
        .expect("No adapter found that supports external memory");
    let (device, queue) =
        raw_gpu_handle_wgpu::create_device(adapter, &wgpu::DeviceDescriptor::default());
    let (externally_usable_buffer, import_info) = raw_gpu_handle_wgpu::create_and_export_buffer(
        adapter.get_info().backend,
        &device,
        &wgpu::BufferDescriptor {
            label: None,
            size: 64,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        },
    );
    let download_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let data = [0, 1, 2, 3, 5, 6, 7, 8];
    queue.write_buffer(&externally_usable_buffer, 0, &data);
    {
        let mut cr = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        cr.copy_buffer_to_buffer(&externally_usable_buffer, 0, &download_buffer, 0, 8);
        queue.submit(std::iter::once(cr.finish()));
    }
    download_buffer.map_async(wgpu::MapMode::Read, 0..8, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let range = download_buffer.get_mapped_range(0..8);
    let slice: &[u8] = &range;
    assert_eq!(&data, slice);
}
