use crate::vulkan::{Vertex, VkDevice, find_depth_format, VkInstance, VkPhysicalDevice};

use ash::vk;
use std::ffi::CString;
use std::fs::File;

pub struct VkPipeline {
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
}

impl VkPipeline {
    pub fn new(instance: &VkInstance, physical_device: &VkPhysicalDevice, device: &VkDevice, format: vk::Format) -> Result<VkPipeline, String> {
        let render_pass = VkPipeline::create_render_pass(instance, physical_device, device, format)?;
        let descriptor_set_layout = VkPipeline::create_descriptor_set_layout(device)?;
        let (pipeline, pipeline_layout) =
            VkPipeline::create_graphics_pipeline(device, &render_pass, &descriptor_set_layout)?;

        return Ok(VkPipeline {
            render_pass,
            pipeline,
            descriptor_set_layout,
            pipeline_layout,
        });
    }

    fn create_render_pass(
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        device: &VkDevice,
        swapchain_format: vk::Format
    ) -> Result<vk::RenderPass, String> {
        let color_attachment = vk::AttachmentDescription {
            format: swapchain_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attachment = vk::AttachmentDescription {
            format: find_depth_format(instance, physical_device)?,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };

        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: &depth_attachment_ref,
            ..Default::default()
        };

        let dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let attachments = [color_attachment, depth_attachment];
        let render_pass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &dependency,
            ..Default::default()
        };

        let render_pass = unsafe {
            device
                .device
                .create_render_pass(&render_pass_create_info, None)
                .map_err(|e| format!("Failed to create render pass: {}", e))?
        };

        return Ok(render_pass);
    }

    fn create_graphics_pipeline(
        device: &VkDevice,
        render_pass: &vk::RenderPass,
        descriptor_set_layout: &vk::DescriptorSetLayout,
    ) -> Result<(vk::Pipeline, vk::PipelineLayout), String> {
        let frag = VkPipeline::read_spv_file("shaders/shader.frag.spv")?;
        let vert = VkPipeline::read_spv_file("shaders/shader.vert.spv")?;

        let frag_shader_module = VkPipeline::create_shader_module(device, &frag)?;
        let vert_shader_module = VkPipeline::create_shader_module(device, &vert)?;

        let entrypoint = CString::new("main").unwrap();
        let vert_shader_create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::VERTEX,
            module: vert_shader_module,
            p_name: entrypoint.as_ptr(),
            ..Default::default()
        };

        let frag_shader_create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: frag_shader_module,
            p_name: entrypoint.as_ptr(),
            ..Default::default()
        };

        let shader_stages = [vert_shader_create_info, frag_shader_create_info];

        let binding_description = Vertex::get_binding_description();
        let attribute_descriptions = Vertex::get_attribute_description();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: &binding_description,
            vertex_attribute_description_count: attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: attribute_descriptions.as_ptr(),
            ..Default::default()
        };

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            viewport_count: 1,
            scissor_count: 1,
            ..Default::default()
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            depth_bias_constant_factor: 0.,
            depth_bias_clamp: 0.,
            depth_bias_slope_factor: 0.,
            ..Default::default()
        };

        let multisampling = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            min_sample_shading: 1.,
            p_sample_mask: std::ptr::null(),
            alpha_to_coverage_enable: vk::FALSE,
            alpha_to_one_enable: vk::FALSE,
            ..Default::default()
        };

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: vk::FALSE,
            stencil_test_enable: vk::FALSE,
            ..Default::default()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        };

        let color_blending = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: &color_blend_attachment,
            blend_constants: [0.; 4],
            ..Default::default()
        };

        let dynamic_states = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            set_layout_count: 1,
            p_set_layouts: descriptor_set_layout,
            push_constant_range_count: 0,
            p_push_constant_ranges: std::ptr::null(),
            ..Default::default()
        };

        let pipeline_layout = unsafe {
            device
                .device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .map_err(|e| format!("Failed to create pipeline layout: {}", e))?
        };

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_info,
            p_input_assembly_state: &input_assembly,
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterizer,
            p_multisample_state: &multisampling,
            p_color_blend_state: &color_blending,
            p_dynamic_state: &dynamic_state,
            p_depth_stencil_state: &depth_stencil,
            layout: pipeline_layout,
            render_pass: *render_pass,
            subpass: 0,
            ..Default::default()
        };

        let pipeline_create_infos = [pipeline_create_info];
        let pipeline_cache = vk::PipelineCache::null();
        let pipeline = unsafe {
            device
                .device
                .create_graphics_pipelines(pipeline_cache, &pipeline_create_infos, None)
                .map_err(|_| format!("Failed to create graphics pipeline"))?
                .remove(0)
        };

        return Ok((pipeline, pipeline_layout));
    }

    fn create_descriptor_set_layout(device: &VkDevice) -> Result<vk::DescriptorSetLayout, String> {
        let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
            ..Default::default()
        };

        let create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            binding_count: 1,
            p_bindings: &ubo_layout_binding,
            ..Default::default()
        };

        let descriptor_set_layout = unsafe {
            device
                .device
                .create_descriptor_set_layout(&create_info, None)
                .map_err(|e| format!("Failed to create descriptor set layout: {}", e))?
        };

        return Ok(descriptor_set_layout);
    }

    fn create_shader_module(
        device: &VkDevice,
        code: &Vec<u32>,
    ) -> Result<vk::ShaderModule, String> {
        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            code_size: code.len() * std::mem::size_of::<u32>(),
            p_code: code.as_ptr(),
            ..Default::default()
        };

        let shader_module = unsafe {
            device
                .device
                .create_shader_module(&create_info, None)
                .map_err(|e| format!("Failed to create shader module: {}", e))?
        };

        return Ok(shader_module);
    }

    fn read_spv_file(path: &str) -> Result<Vec<u32>, String> {
        let mut file =
            File::open(path).map_err(|e| format!("Failed to open file {}: {}", path, e))?;

        let content = ash::util::read_spv(&mut file)
            .map_err(|e| format!("Failed to decode SPIR-V file {}: {}", path, e))?;

        return Ok(content);
    }
}
