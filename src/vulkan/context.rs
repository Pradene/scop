use crate::objects::Object;
use crate::vulkan::UniformBufferObject;
use crate::vulkan::{
    VkBuffer, VkCommandPool, VkDevice, VkInstance, VkPhysicalDevice, VkPipeline, VkSurface,
    VkSwapchain, VkSyncObjects
};
use crate::vulkan::{MAX_FRAMES_IN_FLIGHT, VALIDATION_LAYERS_ENABLED};

use ash::vk;
use lineal::{Matrix, Vector};
use winit::window::Window;

use std::ffi::c_void;

pub struct VkContext {
    pub instance: VkInstance,
    pub surface: VkSurface,
    pub physical_device: VkPhysicalDevice,
    pub device: VkDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub swapchain: VkSwapchain,
    pub pipeline: VkPipeline,
    pub command: VkCommandPool,
    pub sync: VkSyncObjects,
    pub frame: u32,
    pub vertex_buffer: VkBuffer,
    pub index_buffer: VkBuffer,

    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub uniform_buffers_mapped: Vec<*mut std::ffi::c_void>,

    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

impl VkContext {
    pub fn new(window: &Window, object: &Object) -> Result<VkContext, String> {
        let instance = VkInstance::new(window)?;
        let surface = VkSurface::new(window, &instance)?;
        let physical_device = VkPhysicalDevice::new(&instance, &surface)?;
        let device = VkDevice::new(&instance, &physical_device)?;
        let graphics_queue = unsafe {
            device
                .device
                .get_device_queue(physical_device.queue_families.graphics_family.unwrap(), 0)
        };
        let present_queue = unsafe {
            device
                .device
                .get_device_queue(physical_device.queue_families.present_family.unwrap(), 0)
        };
        let mut swapchain = VkSwapchain::new(window, &instance, &surface, &physical_device, &device)?;
        let pipeline = VkPipeline::new(&instance, &physical_device, &device, swapchain.image_format)?;
        let command = VkCommandPool::new(&physical_device, &device)?;
                
        swapchain.create_depth_ressources(&instance, &physical_device, &device)?;
        swapchain.create_framebuffers(&device, &pipeline.render_pass)?;

        let (vertices, indices) = object.get_vertices_and_indices();
        
        let vertex_usage = vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER;
        let vertex_buffer = VkBuffer::new(
            &instance,
            &physical_device,
            &device,
            &graphics_queue,
            &command,
            &vertices,
            vertex_usage,
        )?;
        
        let index_usage = vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER;
        let index_buffer = VkBuffer::new(
            &instance,
            &physical_device,
            &device,
            &graphics_queue,
            &command,
            &indices,
            index_usage,
        )?;
        
        let (uniform_buffers, uniform_buffers_memory, uniform_buffers_mapped) =
        VkContext::create_uniform_buffers(&instance, &physical_device, &device)?;
        
        let descriptor_pool = VkContext::create_descriptor_pool(&device)?;
        let descriptor_sets = VkContext::create_descriptor_set(
            &device,
            &descriptor_pool,
            &pipeline.descriptor_set_layout,
            &uniform_buffers,
        )?;
        
        let sync = VkSyncObjects::new(&device)?;
        
        return Ok(VkContext {
            instance,
            surface,
            physical_device,
            device,
            graphics_queue,
            present_queue,
            swapchain,
            pipeline,
            command,
            sync,
            frame: 0,
            vertex_buffer,
            index_buffer,
            uniform_buffers,
            uniform_buffers_memory,
            uniform_buffers_mapped,
            descriptor_pool,
            descriptor_sets,
        });
    }

    fn create_descriptor_pool(device: &VkDevice) -> Result<vk::DescriptorPool, String> {
        let pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: MAX_FRAMES_IN_FLIGHT,
        };

        let create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            pool_size_count: 1,
            p_pool_sizes: &pool_size,
            max_sets: MAX_FRAMES_IN_FLIGHT,
            ..Default::default()
        };

        let descriptor_pool = unsafe {
            device
                .device
                .create_descriptor_pool(&create_info, None)
                .map_err(|e| format!("Failed to create descriptor pool: {}", e))?
        };
        return Ok(descriptor_pool);
    }

    fn create_descriptor_set(
        device: &VkDevice,
        descriptor_pool: &vk::DescriptorPool,
        descriptor_set_layout: &vk::DescriptorSetLayout,
        uniform_buffers: &Vec<vk::Buffer>,
    ) -> Result<Vec<vk::DescriptorSet>, String> {
        let layouts = vec![*descriptor_set_layout; MAX_FRAMES_IN_FLIGHT as usize];

        let allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: *descriptor_pool,
            descriptor_set_count: MAX_FRAMES_IN_FLIGHT,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };

        let descriptor_sets = unsafe {
            device
                .device
                .allocate_descriptor_sets(&allocate_info)
                .map_err(|e| format!("Failed to allocate descriptor sets: {}", e))?
        };

        for index in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info = vk::DescriptorBufferInfo {
                buffer: uniform_buffers[index as usize],
                offset: 0,
                range: std::mem::size_of::<UniformBufferObject>() as u64,
            };

            let descriptor_write = vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                dst_set: descriptor_sets[index as usize],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &buffer_info,
                ..Default::default()
            };

            unsafe {
                device
                    .device
                    .update_descriptor_sets(&[descriptor_write], &[])
            };
        }

        return Ok(descriptor_sets);
    }

    fn create_uniform_buffers(
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        device: &VkDevice,
    ) -> Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>, Vec<*mut c_void>), String> {
        let buffer_size: vk::DeviceSize = std::mem::size_of::<UniformBufferObject>() as u64;

        let capacity = MAX_FRAMES_IN_FLIGHT as usize;
        let mut uniform_buffers = Vec::with_capacity(capacity);
        let mut uniform_buffers_memory = Vec::with_capacity(capacity);
        let mut uniform_buffers_mapped = Vec::with_capacity(capacity);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let usage = vk::BufferUsageFlags::UNIFORM_BUFFER;
            let properties =
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
            let (buffer, buffer_memory) = VkBuffer::create_buffer(
                &instance,
                &physical_device,
                &device,
                &buffer_size,
                &usage,
                &properties,
            )
            .unwrap();

            let buffer_mapped = unsafe {
                device
                    .device
                    .map_memory(buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                    .map_err(|e| format!("Failed to map memory: {}", e))?
            };

            uniform_buffers.push(buffer);
            uniform_buffers_memory.push(buffer_memory);
            uniform_buffers_mapped.push(buffer_mapped);
        }

        return Ok((
            uniform_buffers,
            uniform_buffers_memory,
            uniform_buffers_mapped,
        ));
    }

    fn update_uniform_buffer(&mut self, current_image: u32) {
        static mut START_TIME: Option<std::time::Instant> = None;

        unsafe {
            if START_TIME.is_none() {
                START_TIME = Some(std::time::Instant::now());
            }
        }

        let current_time = std::time::Instant::now();
        let elapsed_time = unsafe {
            current_time
                .duration_since(START_TIME.unwrap())
                .as_secs_f32()
        };

        let model = lineal::rotate(
            Matrix::identity(),
            lineal::radian(90. * elapsed_time),
            Vector::new([0., 1., 0.]),
        );
        let view = lineal::look_at(
            Vector::new([0., 2., 200.]),
            Vector::new([0., 0., 0.]),
            Vector::new([0., 1., 0.]),
        );
        let mut proj = lineal::projection(
            lineal::radian(45.),
            self.swapchain.extent.width as f32 / self.swapchain.extent.height as f32,
            0.1,
            500.,
        );

        proj[1][1] = proj[1][1] * -1.;

        let ubo = UniformBufferObject { model, view, proj };

        let src = &ubo as *const _ as *const u8;
        let dst = self.uniform_buffers_mapped[current_image as usize] as *mut u8;
        let size = std::mem::size_of::<UniformBufferObject>();
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, size);
        }
    }

    pub fn draw_frame(&mut self, window: &Window) {
        let _ = unsafe {
            self.device.device.wait_for_fences(
                &[self.sync.in_flight_fences[self.frame as usize]],
                true,
                u64::MAX,
            )
        };

        let acquire_result = unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.instance,
                u64::MAX,
                self.sync.image_available_semaphores[self.frame as usize],
                vk::Fence::null(),
            )
        };

        let image_index;
        match acquire_result {
            Ok((index, suboptimal)) => {
                if suboptimal {
                    self.recreate_swapchain(window);
                    return;
                }

                image_index = index;
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swapchain(window);
                return;
            }
            Err(e) => panic!("Failed to acquire next image: {:?}", e),
        };

        self.update_uniform_buffer(self.frame);

        let _ = unsafe {
            self.device
                .device
                .reset_fences(&[self.sync.in_flight_fences[self.frame as usize]])
        };

        let _ = unsafe {
            self.device.device.reset_command_buffer(
                self.command.buffers[self.frame as usize],
                vk::CommandBufferResetFlags::empty(),
            )
        };

        let _ = self.record_command_buffer(&self.command.buffers[self.frame as usize], image_index);

        let signal_semaphores = [self.sync.render_finished_semaphores[self.frame as usize]];
        let wait_semaphores = [self.sync.image_available_semaphores[self.frame as usize]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command.buffers[self.frame as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };

        let _ = unsafe {
            self.device.device.queue_submit(
                self.graphics_queue,
                &[submit_info],
                self.sync.in_flight_fences[self.frame as usize],
            )
        };

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: [self.swapchain.instance].as_ptr(),
            p_image_indices: &image_index,
            p_results: std::ptr::null_mut(),
            ..Default::default()
        };

        let _ = unsafe {
            self.swapchain
                .loader
                .queue_present(self.present_queue, &present_info)
                .unwrap()
        };

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub fn record_command_buffer(
        &self,
        command_buffer: &vk::CommandBuffer,
        image_index: u32,
    ) -> Result<(), String> {
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            flags: vk::CommandBufferUsageFlags::empty(),
            p_inheritance_info: std::ptr::null(),
            ..Default::default()
        };

        let clear_color = vk::ClearColorValue {
            float32: [0., 0., 0., 1.0],
        };

        let clear_color = vk::ClearValue {
            color: clear_color
        };
        let clear_stencil = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.,
                stencil: 0
            },
        };

        let clear_values  = [clear_color, clear_stencil];

        let render_pass_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.pipeline.render_pass,
            framebuffer: self.swapchain.framebuffers[image_index as usize],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        let viewport = vk::Viewport {
            x: 0.,
            y: 0.,
            width: self.swapchain.extent.width as f32,
            height: self.swapchain.extent.height as f32,
            min_depth: 0.,
            max_depth: 1.,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain.extent,
        };

        unsafe {
            self.device
                .device
                .begin_command_buffer(*command_buffer, &begin_info)
                .map_err(|e| format!("Failed to start command buffer: {}", e))?;

            self.device.device.cmd_begin_render_pass(
                *command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            self.device.device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );

            self.device.device.cmd_bind_vertex_buffers(
                *command_buffer,
                0,
                &[self.vertex_buffer.buffer],
                &[0],
            );

            self.device.device.cmd_bind_index_buffer(
                *command_buffer,
                self.index_buffer.buffer,
                0,
                vk::IndexType::UINT32,
            );

            self.device
                .device
                .cmd_set_viewport(*command_buffer, 0, &[viewport]);

            self.device
                .device
                .cmd_set_scissor(*command_buffer, 0, &[scissor]);

            self.device.device.cmd_bind_descriptor_sets(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline_layout,
                0,
                &[self.descriptor_sets[self.frame as usize]],
                &[],
            );

            self.device.device.cmd_draw_indexed(
                *command_buffer,
                self.index_buffer.size as u32,
                1,
                0,
                0,
                0,
            );

            self.device.device.cmd_end_render_pass(*command_buffer);

            self.device
                .device
                .end_command_buffer(*command_buffer)
                .map_err(|e| format!("Failed to end command buffer: {}", e))?
        };

        return Ok(());
    }

    pub fn cleanup(&mut self) {
        self.swapchain.cleanup(&self.device);

        unsafe {
            self.device
                .device
                .destroy_descriptor_set_layout(self.pipeline.descriptor_set_layout, None);

            self.device
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.device
                .device
                .destroy_buffer(self.vertex_buffer.buffer, None);
            self.device
                .device
                .free_memory(self.vertex_buffer.memory, None);

            self.device
                .device
                .destroy_buffer(self.index_buffer.buffer, None);
            self.device
                .device
                .free_memory(self.index_buffer.memory, None);

            self.device
                .device
                .destroy_pipeline(self.pipeline.pipeline, None);
            self.device
                .device
                .destroy_render_pass(self.pipeline.render_pass, None);

            for index in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .device
                    .destroy_semaphore(self.sync.render_finished_semaphores[index as usize], None);
                self.device
                    .device
                    .destroy_semaphore(self.sync.image_available_semaphores[index as usize], None);
                self.device
                    .device
                    .destroy_fence(self.sync.in_flight_fences[index as usize], None);
            }

            self.device
                .device
                .destroy_command_pool(self.command.pool, None);
            self.device.device.destroy_device(None);

            if VALIDATION_LAYERS_ENABLED {}

            self.surface
                .loader
                .destroy_surface(self.surface.surface, None);
            self.instance.instance.destroy_instance(None);
        }
    }
}
