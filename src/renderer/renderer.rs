use std::path::Path;

use ash::vk;

use super::query_swapchain_support;
use super::MAX_FRAMES_IN_FLIGHT;
use super::{
    VkCommandPool, VkContext, VkDescriptorPool, VkDescriptorSetLayout, VkPipeline, VkQueue,
    VkRenderPass, VkSwapchain,
};
use crate::camera::Camera;
use crate::math::Mat4;
use crate::math::Vec3;
use crate::renderer::FrameData;
use crate::renderer::MeshHandle;
use crate::renderer::MeshPushConstants;
use crate::renderer::ResourceManager;
use crate::renderer::{Mesh, SubMesh};

// use winit::window::Window;
use sdl3::video::Window;

#[repr(C)]
pub struct MaterialPushConstants {
    pub ambient: Vec3,
    pub dissolve: f32,
    pub diffuse: Vec3,
    pub shininess: f32,
    pub specular: Vec3,
    pub optical_density: f32,
    pub illum: i32,
    pub tex_diffuse: u32,
    pub tex_ambient: u32,
    pub tex_specular: u32,
}

pub struct Uniforms {
    pub view: Mat4,
    pub proj: Mat4,
}

pub struct Renderer {
    frames: Vec<FrameData>,
    frame: usize,

    resources: ResourceManager,
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
        let (width, height) = window.size().into();
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

        let descriptor_pool = VkDescriptorPool::new(context.device(), MAX_FRAMES_IN_FLIGHT)?;

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

        let resources = ResourceManager::new(&context, &graphics_queue, &command_pool)
            .map_err(|e| format!("Failed to create resources manager: {}", e))?;

        let white = resources.get_texture(ResourceManager::white_texture());
        for frame in &frames {
            descriptor_pool.register_texture_to_descriptor(
                frame.descriptor_set,
                ResourceManager::white_texture(),
                white,
            );
        }

        Ok(Renderer {
            context,
            graphics_queue,
            present_queue,
            swapchain,
            render_pass,
            pipeline,
            command_pool,
            resources,
            descriptor_pool,
            descriptor_set_layout,
            frames,
            frame: 0,
        })
    }

    pub fn load_object(&mut self, path: &Path) -> Result<MeshHandle, String> {
        let (handle, new_textures) = self.resources.load_object(
            &self.context,
            &self.graphics_queue,
            &self.command_pool,
            path,
        )?;

        // Register only the newly added textures to every frame's descriptor set
        for handle in new_textures {
            let texture = self.resources.get_texture(handle);
            for frame in &self.frames {
                self.descriptor_pool.register_texture_to_descriptor(
                    frame.descriptor_set,
                    handle,
                    texture,
                );
            }
        }

        Ok(handle)
    }

    pub fn draw(&mut self, window: &Window, camera: &Camera) -> Result<(), String> {
        self.wait_for_frame()?;

        let image_index = match self.acquire_image()? {
            Some(index) => index,
            None => {
                let (w, h) = window.size().into();
                self.resize(w, h)?;
                return Ok(());
            }
        };

        self.frames[self.frame].update_uniforms(camera);
        self.reset_frame()?;
        self.record(image_index)?;
        self.submit()?;

        if self.present(image_index)? {
            let (w, h) = window.size().into();
            self.resize(w, h)?;
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT as usize;
        Ok(())
    }

    fn wait_for_frame(&self) -> Result<(), String> {
        let fence = self.frames[self.frame].in_flight.handle;
        unsafe {
            self.context
                .device()
                .handle
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
            device
                .handle
                .reset_fences(&[frame.in_flight.handle])
                .map_err(|e| format!("Failed to reset fence: {}", e))?;
            device
                .handle
                .reset_command_buffer(frame.command_buffer, vk::CommandBufferResetFlags::empty())
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
        self.swapchain
            .queue_present(&self.present_queue.handle, &signal_semaphores, image_index)
    }

    fn record(&self, image_index: u32) -> Result<(), String> {
        let frame = &self.frames[self.frame];
        let cmd = frame.command_buffer;
        let device = self.context.device();

        unsafe {
            device
                .handle
                .begin_command_buffer(
                    cmd,
                    &vk::CommandBufferBeginInfo {
                        s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                        ..Default::default()
                    },
                )
                .map_err(|e| format!("Failed to begin command buffer: {}", e))?;
        }

        self.begin_render_pass(cmd, image_index);
        self.bind_pipeline_and_viewport(cmd, frame);

        unsafe {
            device
                .handle
                .cmd_set_cull_mode(cmd, vk::CullModeFlags::FRONT);
            self.draw_meshes(&device.handle, &cmd, frame);
            device
                .handle
                .cmd_set_cull_mode(cmd, vk::CullModeFlags::BACK);
            self.draw_meshes(&device.handle, &cmd, frame);
            device.handle.cmd_end_render_pass(cmd);

            device
                .handle
                .end_command_buffer(cmd)
                .map_err(|e| format!("Failed to end command buffer: {}", e))?;
        }

        Ok(())
    }

    fn begin_render_pass(&self, cmd: vk::CommandBuffer, image_index: u32) {
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0., 0., 0., 1.],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.,
                    stencil: 0,
                },
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

        let device = self.context.device();
        unsafe {
            device.handle.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.handle,
            );
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

    fn draw_meshes(&self, device: &ash::Device, cmd: &vk::CommandBuffer, frame: &FrameData) {
        unsafe {
            device.cmd_bind_descriptor_sets(
                *cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[frame.descriptor_set],
                &[],
            );
        }

        for mesh in &self.resources.meshes {
            self.bind_mesh(device, cmd, mesh);
            for submesh in &mesh.primitives {
                self.draw_submesh(device, cmd, submesh);
            }
        }
    }

    fn bind_mesh(&self, device: &ash::Device, cmd: &vk::CommandBuffer, mesh: &Mesh) {
        let vpc = MeshPushConstants {
            transform: mesh.transform,
        };

        unsafe {
            device.cmd_push_constants(
                *cmd,
                self.pipeline.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                std::slice::from_raw_parts(
                    &vpc as *const _ as *const u8,
                    std::mem::size_of::<MeshPushConstants>(),
                ),
            );
            device.cmd_bind_vertex_buffers(*cmd, 0, &[mesh.vertex_buffer.handle], &[0]);
            device.cmd_bind_index_buffer(*cmd, mesh.index_buffer.handle, 0, vk::IndexType::UINT32);
        }
    }

    fn draw_submesh(&self, device: &ash::Device, cmd: &vk::CommandBuffer, submesh: &SubMesh) {
        let mat = self.resources.get_material(submesh.material);
        let fpc = MaterialPushConstants {
            ambient: mat.ka.unwrap_or(Vec3::new(0.1, 0.1, 0.1)),
            dissolve: mat.dissolve.unwrap_or(1.0),
            diffuse: mat.kd.unwrap_or(Vec3::new(0.7, 0.7, 0.7)),
            shininess: mat.ns.unwrap_or(32.0),
            specular: mat.ks.unwrap_or(Vec3::new(1.0, 1.0, 1.0)),
            optical_density: mat.ni.unwrap_or(1.0),
            illum: mat.illum.unwrap_or(2),
            tex_diffuse: mat.map_kd.unwrap_or(ResourceManager::white_texture()) as u32,
            tex_specular: mat.map_ks.unwrap_or(ResourceManager::white_texture()) as u32,
            tex_ambient: mat.map_ka.unwrap_or(ResourceManager::white_texture()) as u32,
        };

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

    pub fn get_mesh(&mut self, handle: MeshHandle) -> &mut Mesh {
        self.resources.get_mesh(handle)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.wait_idle();
    }
}
