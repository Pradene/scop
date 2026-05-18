use ash::vk;

use super::query_swapchain_support;
use super::MAX_FRAMES_IN_FLIGHT;
use super::{
    VkCommandPool, VkContext, VkDescriptorPool, VkDescriptorSetLayout, VkPipeline, VkQueue,
    VkRenderPass, VkSwapchain,
};
use crate::renderer::{VkBuffer, FrameData};
use crate::camera::Camera;
use crate::math::Mat4;
use crate::scene::ModelPushConstants;
use crate::scene::Scene;
use crate::scene::{MaterialPushConstants, Object};
use crate::scene::{Mesh, SubMesh};

use winit::window::Window;

pub struct Uniforms {
    pub view: Mat4,
    pub proj: Mat4,
}

pub struct Renderer {
    frames: Vec<FrameData>,
    frame: usize,

    meshes: Vec<Mesh>,
    command_pool: VkCommandPool,
    swapchain: VkSwapchain,
    pipeline: VkPipeline,
    render_pass: VkRenderPass,
    descriptor_pool: VkDescriptorPool,
    descriptor_set_layout: VkDescriptorSetLayout,
    present_queue: VkQueue,
    graphics_queue: VkQueue,
    context: VkContext,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Renderer, String> {
        let context = VkContext::new(window)
            .map_err(|e| format!("Failed to create Vulkan context: {:?}", e))?;

        let graphics_queue = VkQueue::new(context.device(), context.graphics_family());
        let present_queue = VkQueue::new(context.device(), context.present_family());

        let support_details = query_swapchain_support(
            &context.physical_device.handle,
            &context.surface.loader,
            &context.surface.handle,
        )?;

        let capabilities = support_details.capabilities;
        let surface_format = Renderer::choose_surface_format(&support_details.formats);
        let present_mode = Renderer::choose_present_mode(&support_details.present_modes);
        let (width, height) = window.inner_size().into();
        let extent = Renderer::choose_extent(&support_details.capabilities, width, height);

        let render_pass = VkRenderPass::new(&context, surface_format.format)?;
        let swapchain = VkSwapchain::new(
            &context,
            &render_pass,
            capabilities,
            surface_format,
            present_mode,
            extent,
        )?;

        let descriptor_set_layout = VkDescriptorSetLayout::new(context.device())?;
        let pipeline = VkPipeline::new(context.device(), &render_pass, &descriptor_set_layout)?;
        let command_pool = VkCommandPool::new(
            context.device(),
            context.graphics_family(),
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )?;

        let descriptor_pool =
            VkDescriptorPool::new(context.device(), MAX_FRAMES_IN_FLIGHT)?;

        let frames = (0..MAX_FRAMES_IN_FLIGHT as usize)
            .map(|_| {
                FrameData::new(
                    &context,
                    &command_pool,
                    &descriptor_pool,
                    &descriptor_set_layout,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Renderer {
            context,
            graphics_queue,
            present_queue,
            swapchain,
            render_pass,
            pipeline,
            command_pool,
            descriptor_pool,
            descriptor_set_layout,
            frames,
            frame: 0,
            meshes: Vec::new(),
        })
    }

    pub fn upload_mesh(&mut self, object: &Object) -> Result<(), String> {
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut primitives = Vec::new();

        for group in &object.groups {
            let (vertices, indices) = object.get_group_vertices_and_indices(group);
            if indices.is_empty() {
                continue;
            }

            let index_offset = all_indices.len() as u32;
            let index_count = indices.len() as u32;
            let vertex_offset = all_vertices.len() as i32;

            let material = group
                .material
                .as_ref()
                .and_then(|name| object.materials.get(name))
                .cloned()
                .unwrap_or_default();

            all_vertices.extend_from_slice(&vertices);
            all_indices.extend_from_slice(&indices);

            primitives.push(SubMesh {
                index_offset,
                index_count,
                vertex_offset,
                material,
            });
        }

        if all_vertices.is_empty() || all_indices.is_empty() {
            return Ok(());
        }

        let vertex_buffer = VkBuffer::device_local(
            &self.context,
            &self.graphics_queue,
            &self.command_pool,
            &all_vertices,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        )?;

        let index_buffer = VkBuffer::device_local(
            &self.context,
            &self.graphics_queue,
            &self.command_pool,
            &all_indices,
            vk::BufferUsageFlags::INDEX_BUFFER,
        )?;

        self.meshes.push(Mesh {
            vertex_buffer,
            index_buffer,
            primitives,
        });

        Ok(())
    }

    pub fn draw(&mut self, window: &Window, _scene: &Scene, camera: &Camera) -> Result<(), String> {
        self.wait_for_frame()?;

        let image_index = match self.acquire_image()? {
            Some(index) => index,
            None => {
                let (w, h) = window.inner_size().into();
                self.resize(w, h)?;
                return Ok(());
            }
        };

        self.frames[self.frame].update_uniforms(camera);
        self.reset_frame()?;
        self.record(image_index)?;
        self.submit()?;

        if self.present(image_index)? {
            let (w, h) = window.inner_size().into();
            self.resize(w, h)?;
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT as usize;
        Ok(())
    }

    fn wait_for_frame(&self) -> Result<(), String> {
        let fence = self.frames[self.frame].in_flight.handle;
        unsafe {
            self.context.device().handle
                .wait_for_fences(&[fence], true, u64::MAX)
                .map_err(|e| format!("Failed to wait for fence: {}", e))
        }
    }

    fn acquire_image(&self) -> Result<Option<u32>, String> {
        let semaphore = self.frames[self.frame].image_available.handle;
        match unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                semaphore,
                vk::Fence::null(),
            )
        } {
            Ok((index, false)) => Ok(Some(index)),
            Ok((_, true)) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => Ok(None),
            Err(e) => Err(format!("Failed to acquire next image: {:?}", e)),
        }
    }

    fn reset_frame(&self) -> Result<(), String> {
        let frame = &self.frames[self.frame];
        let device = self.context.device();
        unsafe {
            device.handle
                .reset_fences(&[frame.in_flight.handle])
                .map_err(|e| format!("Failed to reset fence: {}", e))?;
            device.handle
                .reset_command_buffer(
                    frame.command_buffer,
                    vk::CommandBufferResetFlags::empty(),
                )
                .map_err(|e| format!("Failed to reset command buffer: {}", e))
        }
    }

    fn submit(&self) -> Result<(), String> {
        let frame = &self.frames[self.frame];
        let wait_semaphores = [frame.image_available.handle];
        let signal_semaphores = [frame.render_finished.handle];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        self.graphics_queue.submit(
            &frame.command_buffer,
            &wait_semaphores,
            &signal_semaphores,
            &wait_stages,
            &frame.in_flight.handle,
        )
    }

    fn present(&self, image_index: u32) -> Result<bool, String> {
        let signal_semaphores = [self.frames[self.frame].render_finished.handle];
        self.swapchain.queue_present(
            &self.present_queue.handle,
            &signal_semaphores,
            image_index,
        )
    }

    fn record(&self, image_index: u32) -> Result<(), String> {
        let frame = &self.frames[self.frame];
        let cmd = frame.command_buffer;
        let device = self.context.device();

        self.begin_command_buffer(cmd)?;
        self.begin_render_pass(cmd, image_index);
        self.bind_pipeline_and_viewport(cmd, frame);

        unsafe {
            device.handle.cmd_set_cull_mode(cmd, vk::CullModeFlags::FRONT);
            self.draw_meshes(&device.handle, &cmd);
            device.handle.cmd_set_cull_mode(cmd, vk::CullModeFlags::BACK);
            self.draw_meshes(&device.handle, &cmd);
            device.handle.cmd_end_render_pass(cmd);
        }

        self.end_command_buffer(cmd)
    }

    fn begin_command_buffer(&self, cmd: vk::CommandBuffer) -> Result<(), String> {
        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            ..Default::default()
        };
        unsafe {
            self.context.device().handle
                .begin_command_buffer(cmd, &begin_info)
                .map_err(|e| format!("Failed to begin command buffer: {}", e))
        }
    }

    fn end_command_buffer(&self, cmd: vk::CommandBuffer) -> Result<(), String> {
        unsafe {
            self.context.device().handle
                .end_command_buffer(cmd)
                .map_err(|e| format!("Failed to end command buffer: {}", e))
        }
    }

    fn begin_render_pass(&self, cmd: vk::CommandBuffer, image_index: u32) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue { float32: [0., 0., 0., 1.] },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue { depth: 1., stencil: 0 },
            },
        ];

        let render_pass_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.render_pass.handle,
            framebuffer: self.swapchain.framebuffers[image_index as usize],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        unsafe {
            self.context.device().handle.cmd_begin_render_pass(
                cmd,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    fn bind_pipeline_and_viewport(&self, cmd: vk::CommandBuffer, frame: &FrameData) {
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

        let device = self.context.device();
        unsafe {
            device.handle.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline.handle);
            device.handle.cmd_set_viewport(cmd, 0, &[viewport]);
            device.handle.cmd_set_scissor(cmd, 0, &[scissor]);
            device.handle.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[frame.descriptor_set],
                &[],
            );
        }
    }

    fn draw_meshes(&self, device: &ash::Device, cmd: &vk::CommandBuffer) {
        for mesh in &self.meshes {
            self.bind_mesh(device, cmd, mesh);
            for submesh in &mesh.primitives {
                self.draw_submesh(device, cmd, submesh);
            }
        }
    }

    fn bind_mesh(&self, device: &ash::Device, cmd: &vk::CommandBuffer, mesh: &Mesh) {
        let vpc = ModelPushConstants { model: Mat4::identity() };
        unsafe {
            device.cmd_push_constants(
                *cmd,
                self.pipeline.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                std::slice::from_raw_parts(
                    &vpc as *const _ as *const u8,
                    std::mem::size_of::<ModelPushConstants>(),
                ),
            );
            device.cmd_bind_vertex_buffers(*cmd, 0, &[mesh.vertex_buffer.handle], &[0]);
            device.cmd_bind_index_buffer(*cmd, mesh.index_buffer.handle, 0, vk::IndexType::UINT32);
        }
    }

    fn draw_submesh(&self, device: &ash::Device, cmd: &vk::CommandBuffer, submesh: &SubMesh) {
        let fpc = MaterialPushConstants::from_material(&submesh.material);
        unsafe {
            device.cmd_push_constants(
                *cmd,
                self.pipeline.layout,
                vk::ShaderStageFlags::FRAGMENT,
                64,
                std::slice::from_raw_parts(
                    &fpc as *const _ as *const u8,
                    std::mem::size_of::<MaterialPushConstants>(),
                ),
            );
            device.cmd_draw_indexed(
                *cmd,
                submesh.index_count,
                1,
                submesh.index_offset,
                submesh.vertex_offset,
                0,
            );
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.wait_idle();

        let support_details = query_swapchain_support(
            &self.context.physical_device.handle,
            &self.context.surface.loader,
            &self.context.surface.handle,
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

    fn choose_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .copied()
            .unwrap_or(formats[0])
    }

    fn choose_present_mode(modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        modes
            .iter()
            .find(|&&m| m == vk::PresentModeKHR::MAILBOX)
            .copied()
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    fn choose_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        width: u32,
        height: u32,
    ) -> vk::Extent2D {
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
