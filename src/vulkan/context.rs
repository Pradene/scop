use ash::vk;
use std::sync::Arc;
use std::ffi::c_void;

use lineal::{Vector, Matrix};

use crate::objects::Object;
use crate::vulkan::{query_swapchain_support, MAX_FRAMES_IN_FLIGHT, UniformBufferObject};
use crate::vulkan::{
    VkBuffer, VkCommandPool, VkDevice, VkInstance, VkPhysicalDevice, VkPipeline, VkQueue,
    VkRenderPass, VkSurface, VkSwapchain, VkSyncObjects
};

use winit::window::Window;

pub struct VkContext {
    pub sync: VkSyncObjects,
    
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,


    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub uniform_buffers_mapped: Vec<*mut std::ffi::c_void>,
    
    pub vertex_buffer: VkBuffer,
    pub index_buffer: VkBuffer,
    
    pub command_pool: VkCommandPool,
    pub render_pass: VkRenderPass,
    pub pipeline: VkPipeline,
    
    pub swapchain: VkSwapchain,
    
    pub present_queue: VkQueue,
    pub graphics_queue: VkQueue,
    
    pub device: Arc<VkDevice>,
    pub physical_device: VkPhysicalDevice,
    pub surface: VkSurface,
    pub instance: VkInstance,
    pub frame: u32,
}

impl VkContext {
    pub fn new(window: &Window, object: &Object) -> Result<VkContext, String> {
        let instance = VkInstance::new(window)?;

        let surface = VkSurface::new(window, &instance)?;

        let physical_device = VkPhysicalDevice::new(&instance, &surface)?;

        let device = Arc::new(VkDevice::new(&instance, &physical_device)?);

        let queue_family_index = physical_device.queue_families.graphics_family.unwrap();
        let graphics_queue = VkQueue::new(device.clone(), queue_family_index);

        let queue_family_index = physical_device.queue_families.present_family.unwrap();
        let present_queue = VkQueue::new(device.clone(), queue_family_index);

        let support_details = query_swapchain_support(
            &physical_device.physical_device,
            &surface.loader,
            &surface.surface,
        )?;

        let capabilities = support_details.capabilities;
        let surface_format = VkContext::choose_surface_format(&support_details.formats);
        let present_mode = VkContext::choose_present_mode(&support_details.present_modes);
        let extent = VkContext::choose_extent(window, &support_details.capabilities);

        let render_pass = VkRenderPass::new(
            &instance,
            &physical_device,
            device.clone(),
            surface_format.format,
        )?;

        let swapchain = VkSwapchain::new(
            &instance,
            &surface,
            &physical_device,
            device.clone(),
            &render_pass,
            capabilities,
            surface_format,
            present_mode,
            extent,
        )?;

        let pipeline = VkPipeline::new(device.clone(), &render_pass)?;

        let command_pool = VkCommandPool::new(&physical_device, device.clone())?;

        let (vertices, indices) = object.get_vertices_and_indices();

        let vertex_usage = vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER;
        let vertex_buffer = VkBuffer::new(
            &instance,
            &physical_device,
            device.clone(),
            &graphics_queue,
            &command_pool,
            &vertices,
            vertex_usage,
        )?;

        let index_usage = vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER;
        let index_buffer = VkBuffer::new(
            &instance,
            &physical_device,
            device.clone(),
            &graphics_queue,
            &command_pool,
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

        let sync = VkSyncObjects::new(device.clone())?;

        return Ok(VkContext {
            instance,
            surface,
            physical_device,
            device,
            graphics_queue,
            present_queue,
            swapchain,
            render_pass,
            pipeline,
            command_pool,
            frame: 0,
            vertex_buffer,
            index_buffer,
            uniform_buffers,
            uniform_buffers_memory,
            uniform_buffers_mapped,
            descriptor_pool,
            descriptor_sets,
            sync,
        });
    }

    fn choose_surface_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *available_format;
            }
        }

        return available_formats[0];
    }

    fn choose_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        for available_present_mode in available_present_modes {
            if *available_present_mode == vk::PresentModeKHR::MAILBOX {
                return *available_present_mode;
            }
        }

        return vk::PresentModeKHR::FIFO;
    }

    fn choose_extent(window: &Window, capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        } else {
            let (width, height): (u32, u32) = window.inner_size().into();

            let extent = vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            };

            return extent;
        }
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
            Vector::new([0., 0., 20.]),
            Vector::new([0., 0., 0.]),
            Vector::new([0., 1., 0.]),
        );
        let mut proj = lineal::projection(
            lineal::radian(45.),
            self.swapchain.extent.width as f32 / self.swapchain.extent.height as f32,
            0.1,
            50.,
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
                self.swapchain.swapchain,
                u64::MAX,
                self.sync.image_available_semaphores[self.frame as usize],
                vk::Fence::null(),
            )
        };

        let image_index;
        match acquire_result {
            Ok((index, suboptimal)) => {
                if suboptimal {
                    self.resize(window).unwrap();
                    return;
                }

                image_index = index;
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.resize(window).unwrap();
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
                self.command_pool.buffers[self.frame as usize],
                vk::CommandBufferResetFlags::empty(),
            )
        };

        let _ = self.record_command_buffer(&self.command_pool.buffers[self.frame as usize], image_index);

        let signal_semaphores = [self.sync.render_finished_semaphores[self.frame as usize]];
        let wait_semaphores = [self.sync.image_available_semaphores[self.frame as usize]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_pool.buffers[self.frame as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };

        let _ = unsafe {
            self.device.device.queue_submit(
                self.graphics_queue.queue,
                &[submit_info],
                self.sync.in_flight_fences[self.frame as usize],
            )
        };

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: [self.swapchain.swapchain].as_ptr(),
            p_image_indices: &image_index,
            p_results: std::ptr::null_mut(),
            ..Default::default()
        };

        let _ = unsafe {
            self.swapchain
                .loader
                .queue_present(self.present_queue.queue, &present_info)
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

        let clear_color = vk::ClearValue { color: clear_color };
        let clear_stencil = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.,
                stencil: 0,
            },
        };

        let clear_values = [clear_color, clear_stencil];

        let render_pass_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.render_pass.render_pass,
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

    pub fn resize(&mut self, window: &Window) -> Result<(), String> {
        let _ = unsafe {
            self.device.device.device_wait_idle()
        };
        
        let support_details = query_swapchain_support(
            &self.physical_device.physical_device,
            &self.surface.loader,
            &self.surface.surface,
        )?;

        let capabilities = support_details.capabilities;
        let surface_format = VkContext::choose_surface_format(&support_details.formats);
        let present_mode = VkContext::choose_present_mode(&support_details.present_modes);
        let extent = VkContext::choose_extent(window, &support_details.capabilities);

        self.swapchain.resize(
            &self.instance,
            &self.surface,
            &self.physical_device,
            self.device.clone(),
            &self.render_pass,
            capabilities,
            surface_format,
            present_mode,
            extent
        );

        return Ok(());
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device.device_wait_idle();
            
            for i  in 0..self.uniform_buffers.len() {
                self.device.device.destroy_buffer(self.uniform_buffers[i], None);
                self.device.device.free_memory(self.uniform_buffers_memory[i], None);
            }

            self.device.device.destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
