use crate::runner::utils;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, ComputePass, ComputePipeline, Device, ShaderStages,
};

#[derive(Debug)]
pub(crate) struct ComputeShader {
    pub(crate) pipeline: ComputePipeline,
    pub(crate) bind_group: BindGroup,
    pub(crate) is_init_done: bool,
}

impl ComputeShader {
    pub(crate) fn new(device: &Device, buffer: &Buffer, code: &str) -> Self {
        let layout = utils::create_bind_group_layout(device, ShaderStages::COMPUTE, 1);
        let pipeline = utils::create_compute_pipeline(device, &layout, code);
        let bind_group = Self::create_bind_group(device, &layout, buffer);
        Self {
            pipeline,
            bind_group,
            is_init_done: false,
        }
    }

    pub(crate) fn run(&mut self, pass: &mut ComputePass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(1, 1, 1);
        self.is_init_done = true;
    }

    fn create_bind_group(device: &Device, layout: &BindGroupLayout, buffer: &Buffer) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpex:bind_group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }
}
