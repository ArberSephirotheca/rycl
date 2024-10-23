use std::sync::Arc;

use rspirv::{binary::Assemble, spirv};
use shared_type::KernelFn;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::library::VulkanLibrary;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::compute::ComputePipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::shader::{spirv as vulkano_spirv, ShaderModule, ShaderModuleCreateInfo};
use vulkano::sync::{self, GpuFuture};

use super::device_ctx::DeviceCtx;

pub struct Vulkan<'a> {
    device_id: i32,
    device_type: i32,
    entry_point: &'a str,
}

impl<'a> DeviceCtx for Vulkan<'a> {
    fn device_id(&self) -> i32 {
        self.device_id
    }
    fn device_type(&self) -> i32 {
        self.device_type
    }
    fn entry_point(&self) -> &str {
        &self.entry_point
    }
}

impl<'a> Vulkan<'a> {
    pub(crate) fn new(device_id: i32, device_type: i32, entry_point: &'a str) -> Self {
        Self {
            device_id,
            device_type,
            entry_point,
        }
    }

    pub(crate) fn build_spirv(&self, kernel: &str) -> Vec<u32> {
        // Building
        let mut b = rspirv::dr::Builder::new();
        b.set_version(1, 0);
        b.capability(spirv::Capability::Shader);
        b.memory_model(spirv::AddressingModel::Logical, spirv::MemoryModel::GLSL450);
        let void = b.type_void();
        let voidf = b.type_function(void, vec![]);
        let fun = b
            .begin_function(
                void,
                None,
                spirv::FunctionControl::DONT_INLINE | spirv::FunctionControl::CONST,
                voidf,
            )
            .unwrap();
        b.begin_block(None).unwrap();
        // fixme: add spir-v code
        b.ret().unwrap();
        b.end_function().unwrap();
        b.entry_point(spirv::ExecutionModel::Vertex, fun, self.entry_point, vec![]);
        let module = b.module();

        // Assembling
        module.assemble()
    }



    // fixme: add real implementation
    pub(crate) fn run(&self, spirv_binary: &Vec<u32>) {
        // fixme: error handling
        let library = VulkanLibrary::new().unwrap();
        let instance_create_info = InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            ..Default::default()
        };
        let instance = Instance::new(library, instance_create_info).unwrap();

        // Choose which physical device to use.
        let device_extensions = DeviceExtensions {
            khr_storage_buffer_storage_class: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                // The Vulkan specs guarantee that a compliant implementation must provide at least one
                // queue that supports compute operations.
                p.queue_family_properties()
                    .iter()
                    .position(|q| q.queue_flags.intersects(QueueFlags::COMPUTE))
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .unwrap();

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        // Now initializing the device.
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
        // implementation we use only one queue, so we just retrieve the first and only element of the
        // iterator and throw it away.
        let queue = queues.next().unwrap();

        let pipeline = {
            let cs = {
                let code = spirv_binary.as_slice();

                let module = unsafe {
                    ShaderModule::new(device.clone(), ShaderModuleCreateInfo::new(code)).unwrap()
                };

                module.entry_point(&self.entry_point).unwrap()
            };
            let stage = PipelineShaderStageCreateInfo::new(cs);
            let layout = PipelineLayout::new(
                device.clone(),
                PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                    .into_pipeline_layout_create_info(device.clone())
                    .unwrap(),
            )
            .unwrap();
            ComputePipeline::new(
                device.clone(),
                None,
                ComputePipelineCreateInfo::stage_layout(stage, layout),
            )
            .unwrap()
        };

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());

        let data_buffer = Buffer::from_iter(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            // Iterator that produces the data.
            0..65536u32,
        )
        .unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            &command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Note that we clone the pipeline and the set. Since they are both wrapped in an `Arc`,
        // this only clones the `Arc` and not the whole pipeline or set (which aren't cloneable
        // anyway). In this example we would avoid cloning them since this is the last time we use
        // them, but in real code you would probably need to clone them.
        builder
            .bind_pipeline_compute(pipeline.clone())
            .unwrap()
            .push_descriptor_set(
                PipelineBindPoint::Compute,
                pipeline.layout().clone(),
                0,
                [WriteDescriptorSet::buffer(0, data_buffer.clone())]
                    .into_iter()
                    .collect(),
            )
            .unwrap();

        // Finish building the command buffer by calling `build`.
        let command_buffer = builder.build().unwrap();

        // Let's execute this command buffer now.
        let future = sync::now(device)
            .then_execute(queue, command_buffer)
            .unwrap()
            // This line instructs the GPU to signal a *fence* once the command buffer has finished
            // execution. A fence is a Vulkan object that allows the CPU to know when the GPU has
            // reached a certain point. We need to signal a fence here because below we want to block
            // the CPU until the GPU has reached that point in the execution.
            .then_signal_fence_and_flush()
            .unwrap();

        // Blocks execution until the GPU has finished the operation. This method only exists on the
        // future that corresponds to a signalled fence. In other words, this method wouldn't be
        // available if we didn't call `.then_signal_fence_and_flush()` earlier. The `None` parameter
        // is an optional timeout.
        //
        // Note however that dropping the `future` variable (with `drop(future)` for example) would
        // block execution as well, and this would be the case even if we didn't call
        // `.then_signal_fence_and_flush()`. Therefore the actual point of calling
        // `.then_signal_fence_and_flush()` and `.wait()` is to make things more explicit. In the
        // future, if the Rust language gets linear types vulkano may get modified so that only
        // fence-signalled futures can get destroyed like this.
        future.wait(None).unwrap();

        // Now that the GPU is done, the content of the buffer should have been modified. Let's check
        // it out. The call to `read()` would return an error if the buffer was still in use by the
        // GPU.
        let data_buffer_content = data_buffer.read().unwrap();
        for n in 0..65536u32 {
            assert_eq!(data_buffer_content[n as usize], n * 12);
        }

        println!("Success");
    }
}
