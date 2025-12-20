use crate::{Log, LogLevel};
use wgpu::{
    Adapter, BackendOptions, Backends, BindGroupLayout, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, CommandEncoder, CommandEncoderDescriptor,
    ComputePass, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device,
    DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceFlags, Limits, MapMode,
    MemoryBudgetThresholds, MemoryHints, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PollType, PowerPreference, Queue, RequestAdapterOptions, ShaderModuleDescriptor, ShaderStages,
    Trace,
};

pub(crate) fn create_instance() -> Instance {
    Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::from_env().unwrap_or_else(Backends::all),
        flags: InstanceFlags::default(),
        memory_budget_thresholds: MemoryBudgetThresholds::default(),
        backend_options: BackendOptions::default(),
    })
}

pub(crate) async fn create_adapter(instance: &Instance) -> Result<Adapter, Vec<Log>> {
    let adapter_request = RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: None,
    };
    instance
        .request_adapter(&adapter_request)
        .await
        .map_err(|err| {
            vec![Log {
                level: LogLevel::Error,
                msg: format!("no supported graphic adapter found: {err}"),
                loc: None,
                inner: vec![],
            }]
        })
}

pub(crate) async fn create_device(adapter: &Adapter) -> Result<(Device, Queue), Vec<Log>> {
    let device_descriptor = DeviceDescriptor {
        label: Some("gpex:device"),
        required_features: Features::default(),
        required_limits: Limits::default(),
        experimental_features: ExperimentalFeatures::default(),
        memory_hints: MemoryHints::Performance,
        trace: Trace::Off,
    };
    adapter
        .request_device(&device_descriptor)
        .await
        .map_err(|err| {
            vec![Log {
                level: LogLevel::Error,
                msg: format!("cannot retrieve graphic device: {err}"),
                loc: None,
                inner: vec![],
            }]
        })
}

pub(crate) fn create_buffer(device: &Device, label: &str, size: u64) -> Option<Buffer> {
    (size > 0).then(|| {
        device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size,
            usage: BufferUsages::STORAGE
                | BufferUsages::COPY_SRC
                | BufferUsages::COPY_DST
                | BufferUsages::UNIFORM
                | BufferUsages::VERTEX
                | BufferUsages::INDEX,
            mapped_at_creation: false,
        })
    })
}

pub(crate) fn create_bind_group_layout(
    device: &Device,
    stages: ShaderStages,
    storage_count: u32,
) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("gpex:bind_group_layout"),
        entries: &(0..storage_count)
            .map(|binding| BindGroupLayoutEntry {
                binding,
                visibility: stages,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect::<Vec<_>>(),
    })
}

pub(crate) fn create_encoder(device: &Device) -> CommandEncoder {
    device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("gpex:encoder"),
    })
}

pub(crate) fn start_compute_pass(encoder: &mut CommandEncoder) -> ComputePass<'_> {
    encoder.begin_compute_pass(&ComputePassDescriptor {
        label: Some("gpex:compute_pass"),
        timestamp_writes: None,
    })
}

pub(crate) fn create_compute_pipeline(
    device: &Device,
    layout: &BindGroupLayout,
    code: &str,
) -> ComputePipeline {
    let module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("gpex:shader_module"),
        source: wgpu::ShaderSource::Wgsl(code.into()),
    });
    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("gpex:compute_pipeline"),
        layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("gpex:compute_pipeline_layout"),
            bind_group_layouts: &[layout],
            immediate_size: 0,
        })),
        module: &module,
        entry_point: None,
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    })
}

pub(crate) fn read_buffer(
    device: &Device,
    queue: &Queue,
    buffer: &Buffer,
    offset: u64,
    size: u64,
) -> Vec<u8> {
    let read_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("gpex:buffer:storage_read"),
        size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = create_encoder(device);
    encoder.copy_buffer_to_buffer(buffer, offset, &read_buffer, 0, Some(size));
    let submission_index = queue.submit(Some(encoder.finish()));
    let slice = read_buffer.slice(..);
    slice.map_async(MapMode::Read, |_| ());
    #[allow(clippy::expect_used)] // should never happen
    device
        .poll(PollType::Wait {
            submission_index: Some(submission_index),
            timeout: None,
        })
        .expect("internal error: cannot read buffer");
    let view = slice.get_mapped_range();
    let content = view.to_vec();
    drop(view);
    read_buffer.unmap();
    content
}
