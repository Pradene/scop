use ash::vk;

use crate::materials::{Material, MaterialPushConstants};
use crate::math::{Mat4, Vec3};
use crate::objects::Object;
use super::{query_swapchain_support};
use crate::scene::Scene;
use super::MAX_FRAMES_IN_FLIGHT;
use super::{
    Uniforms, Vertex, VkBuffer, VkCommandPool, VkDescriptorPool, VkDescriptorSet,
    VkDescriptorSetLayout, VkFence, VkPipeline, VkQueue,
    VkRenderPass, VkSemaphore, VkSwapchain, VkContext, UniformBuffer
};

use winit::window::Window;

pub struct GpuGroup {
    pub vertex_buffer: VkBuffer<Vertex>,
    pub index_buffer: VkBuffer<u32>,
    pub material: Material,
}

pub struct GpuMesh {
    pub groups: Vec<GpuGroup>,
}

pub struct Renderer {
    // Sync primitives
    pub image_available_semaphores: Vec<VkSemaphore>,
    pub render_finished_semaphores: Vec<VkSemaphore>,
    pub in_flight_fences: Vec<VkFence>,

    // Descriptors
    pub descriptor_sets: Vec<VkDescriptorSet>,
    pub descriptor_pool: VkDescriptorPool,
    pub descriptor_set_layout: VkDescriptorSetLayout,

    // Buffers
    pub uniform_buffers: Vec<UniformBuffer>,
    meshes: Vec<GpuMesh>,

    // Commands
    pub command_pool: VkCommandPool,

    // Swapchain
    pub swapchain: VkSwapchain,

    // Pipeline & render pass
    pub pipeline: VkPipeline,
    pub render_pass: VkRenderPass,

    // Queues
    pub present_queue: VkQueue,
    pub graphics_queue: VkQueue,

    // Core
    pub context: VkContext,

    pub frame: u32,
    pub start: std::time::Instant,
}

impl Renderer {
    pub fn new(window: &Window, context: VkContext) -> Result<Renderer, String> {
        let graphics_queue = VkQueue::new(
            context.device(),
            context.graphics_family(),
        );
        let present_queue = VkQueue::new(
            context.device(),
            context.present_family(),
        );

        let support_details = query_swapchain_support(&context.physical_device.inner, &context.surface.loader, &context.surface.inner)?;

        let capabilities = support_details.capabilities;
        let surface_format = Renderer::choose_surface_format(&support_details.formats);
        let present_mode = Renderer::choose_present_mode(&support_details.present_modes);
        let (width, height) = window.inner_size().into();
        let extent = Renderer::choose_extent(&support_details.capabilities, width, height);

        let render_pass = VkRenderPass::new(
            &context,
            surface_format.format,
        )?;

        let swapchain = VkSwapchain::new(
            &context,
            &render_pass, capabilities, surface_format, present_mode, extent,
        )?;

        let descriptor_set_layout = VkDescriptorSetLayout::new(context.device())?;
        let pipeline = VkPipeline::new(context.device(), &render_pass, &descriptor_set_layout)?;
        let command_pool = VkCommandPool::new(&context.physical_device, context.device())?;

        let uniform_buffers = Renderer::create_uniform_buffers(&context)?;

        let descriptor_pool = VkDescriptorPool::new(context.device())?;
        let descriptor_sets =
            descriptor_pool.create_sets(&descriptor_set_layout, &uniform_buffers)?;

        let mut image_available_semaphores = Vec::new();
        let mut render_finished_semaphores = Vec::new();
        let mut in_flight_fences = Vec::new();

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores.push(VkSemaphore::new(context.device())?);
            render_finished_semaphores.push(VkSemaphore::new(context.device())?);
            in_flight_fences.push(VkFence::new(context.device())?);
        }

        Ok(Renderer {
            context,
            graphics_queue,
            present_queue,
            swapchain,
            render_pass,
            pipeline,
            command_pool,
            frame: 0,
            start: std::time::Instant::now(),
            meshes: Vec::new(),
            uniform_buffers,
            descriptor_pool,
            descriptor_sets,
            descriptor_set_layout,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
        })
    }

    // ── scene sync ────────────────────────────────────────────────────────────

    fn sync_meshes(&mut self, scene: &Scene) -> Result<(), String> {
        // if counts match, assume already in sync (simple Vec approach)
        if self.meshes.len() == scene.objects.len() {
            return Ok(());
        }

        // clear and re-upload everything
        self.meshes.clear();
        for obj in &scene.objects {
            self.upload_mesh(obj)?;
        }

        Ok(())
    }

    fn upload_mesh(&mut self, object: &Object) -> Result<(), String> {
        let mut groups = Vec::new();

        for group in &object.groups {
            let (vertices, indices) = object.get_group_vertices_and_indices(group);
            if indices.is_empty() { continue; }

            let material = group.material.as_ref()
                .and_then(|name| object.materials.get(name))
                .cloned()
                .unwrap_or_default();

            let vertex_buffer = VkBuffer::new(
                &self.context, &self.graphics_queue, &self.command_pool, &vertices,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            )?;
            let index_buffer = VkBuffer::new(
                &self.context, &self.graphics_queue, &self.command_pool, &indices,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            )?;

            groups.push(GpuGroup { vertex_buffer, index_buffer, material });
        }

        self.meshes.push(GpuMesh { groups });
        Ok(())
    }

    // ── per-frame ─────────────────────────────────────────────────────────────

    fn update_uniform_buffer(&mut self, current_image: u32, scene: &Scene) {
        let elapsed = self.start.elapsed().as_secs_f32();

        // take center from first object if present, else zero
        let center = scene.objects.first()
            .map(|o| o.center)
            .unwrap_or_else(|| Vec3::ZERO);

        let model = Mat4::identity()
            .rotate((90.0 * elapsed).to_radians(), Vec3::Y)
            * Mat4::identity().translate(center * -1.);

        let ubo = Uniforms {
            model,
            view: scene.camera.get_view_matrix(),
            proj: scene.camera.get_projection_matrix(),
        };

        self.uniform_buffers[current_image as usize].write(&ubo);
    }

    pub fn draw(&mut self, window: &Window, scene: &Scene) -> Result<(), String> {
        // sync GPU meshes with scene before drawing
        self.sync_meshes(scene)?;

        unsafe {
            self.context.device().inner.wait_for_fences(
                &[self.in_flight_fences[self.frame as usize].inner],
                true,
                u64::MAX,
            ).map_err(|e| format!("Failed to wait for fence: {}", e))?;
        }

        let acquire_result = unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.inner,
                u64::MAX,
                self.image_available_semaphores[self.frame as usize].inner,
                vk::Fence::null(),
            )
        };

        let image_index = match acquire_result {
            Ok((index, suboptimal)) => {
                let (width, height) = window.inner_size().into();
                if suboptimal {
                    self.resize(width, height)?;
                    return Ok(());
                }
                index
            }
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                let (width, height) = window.inner_size().into();
                self.resize(width, height)?;
                return Ok(());
            }
            Err(e) => return Err(format!("Failed to acquire next image: {:?}", e)),
        };

        self.update_uniform_buffer(self.frame, scene);

        unsafe {
            self.context.device().inner
                .reset_fences(&[self.in_flight_fences[self.frame as usize].inner])
                .map_err(|e| format!("Failed to reset fence: {}", e))?;

            self.context.device().inner.reset_command_buffer(
                self.command_pool.buffers[self.frame as usize].inner,
                vk::CommandBufferResetFlags::empty(),
            ).map_err(|e| format!("Failed to reset command buffer: {}", e))?;
        }

        self.record_command_buffer(
            &self.command_pool.buffers[self.frame as usize].inner,
            image_index,
        )?;

        let signal_semaphores = [self.render_finished_semaphores[self.frame as usize].inner];
        let wait_semaphores   = [self.image_available_semaphores[self.frame as usize].inner];
        let wait_stages       = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        self.graphics_queue.submit(
            &self.command_pool.buffers[self.frame as usize].inner,
            &wait_semaphores,
            &signal_semaphores,
            &wait_stages,
            &self.in_flight_fences[self.frame as usize].inner,
        );

        self.swapchain.present_queue(&self.present_queue, &signal_semaphores, image_index);
        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        Ok(())
    }

    fn record_command_buffer(
        &self,
        command_buffer: &vk::CommandBuffer,
        image_index: u32,
    ) -> Result<(), String> {
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            ..Default::default()
        };

        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [0., 0., 0., 1.] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1., stencil: 0 } },
        ];

        let render_pass_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.render_pass.inner,
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
            x: 0., y: 0.,
            width: self.swapchain.extent.width as f32,
            height: self.swapchain.extent.height as f32,
            min_depth: 0., max_depth: 1.,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain.extent,
        };

        unsafe {
            let device = self.context.device();
            device.inner
                .begin_command_buffer(*command_buffer, &begin_info)
                .map_err(|e| format!("Failed to begin command buffer: {}", e))?;

            device.inner.cmd_begin_render_pass(
                *command_buffer, &render_pass_info, vk::SubpassContents::INLINE,
            );

            device.inner.cmd_bind_pipeline(
                *command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.inner,
            );

            device.inner.cmd_set_viewport(*command_buffer, 0, &[viewport]);
            device.inner.cmd_set_scissor(*command_buffer, 0, &[scissor]);

            device.inner.cmd_bind_descriptor_sets(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[self.descriptor_sets[self.frame as usize].inner],
                &[],
            );

            // Two rendering for transparent object
            // Draw back -> front
            device.inner.cmd_set_cull_mode(*command_buffer, vk::CullModeFlags::FRONT);

            for mesh in &self.meshes {
                for group in &mesh.groups {
                    let pc = MaterialPushConstants::from_material(&group.material);
                    device.inner.cmd_push_constants(
                        *command_buffer,
                        self.pipeline.layout,
                        vk::ShaderStageFlags::FRAGMENT,
                        0,
                        std::slice::from_raw_parts(&pc as *const _ as *const u8, std::mem::size_of::<MaterialPushConstants>()),
                    );

                    device.inner.cmd_bind_vertex_buffers(*command_buffer, 0, &[group.vertex_buffer.inner], &[0]);
                    device.inner.cmd_bind_index_buffer(*command_buffer, group.index_buffer.inner, 0, vk::IndexType::UINT32);
                    device.inner.cmd_draw_indexed(*command_buffer, group.index_buffer.size as u32, 1, 0, 0, 0);
                }
            }

            device.inner.cmd_set_cull_mode(*command_buffer, vk::CullModeFlags::BACK);

            for mesh in &self.meshes {
                for group in &mesh.groups {
                    let pc = MaterialPushConstants::from_material(&group.material);
                    device.inner.cmd_push_constants(
                        *command_buffer,
                        self.pipeline.layout,
                        vk::ShaderStageFlags::FRAGMENT,
                        0,
                        std::slice::from_raw_parts(&pc as *const _ as *const u8, std::mem::size_of::<MaterialPushConstants>()),
                    );

                    device.inner.cmd_bind_vertex_buffers(*command_buffer, 0, &[group.vertex_buffer.inner], &[0]);
                    device.inner.cmd_bind_index_buffer(*command_buffer, group.index_buffer.inner, 0, vk::IndexType::UINT32);
                    device.inner.cmd_draw_indexed(*command_buffer, group.index_buffer.size as u32, 1, 0, 0, 0);
                }
            }

            device.inner.cmd_end_render_pass(*command_buffer);

            device.inner
                .end_command_buffer(*command_buffer)
                .map_err(|e| format!("Failed to end command buffer: {}", e))?;
        }

        Ok(())
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    pub fn resize(&mut self, width:u32, height: u32) -> Result<(), String> {
        self.wait_idle();

        let support_details = query_swapchain_support(
            &self.context.physical_device.inner, &self.context.surface.loader, &self.context.surface.inner,
        )?;

        self.swapchain.resize(
            &self.context,
            &self.render_pass,
            support_details.capabilities,
            Renderer::choose_surface_format(&support_details.formats),
            Renderer::choose_present_mode(&support_details.present_modes),
            Renderer::choose_extent(&support_details.capabilities, width, height),
        )
    }

    fn create_uniform_buffers(
        context: &VkContext,
    ) -> Result<Vec<UniformBuffer>, String> {
        (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| UniformBuffer::new(context))
            .collect()
    }

    fn choose_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        formats.iter()
            .find(|f| f.format == vk::Format::B8G8R8A8_SRGB
                   && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .copied()
            .unwrap_or(formats[0])
    }

    fn choose_present_mode(modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        modes.iter()
            .find(|&&m| m == vk::PresentModeKHR::MAILBOX)
            .copied()
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    fn choose_extent(capabilities: &vk::SurfaceCapabilitiesKHR, width: u32, height: u32) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        }
        vk::Extent2D {
            width: width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }

    pub fn wait_idle(&self) {
        self.context.device().wait_idle();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.wait_idle();
    }
}