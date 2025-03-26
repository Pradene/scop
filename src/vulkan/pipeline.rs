use super::{Vertex, VkDescriptorSetLayout, VkDevice, VkRenderPass, VkShaderModule};

use ash::vk;
use std::ffi::CString;
use std::sync::Arc;

pub struct VkPipeline {
    device: Arc<VkDevice>,
    pub inner: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

impl VkPipeline {
    pub fn new(
        device: Arc<VkDevice>,
        render_pass: &VkRenderPass,
        descriptor_set_layout: &VkDescriptorSetLayout,
    ) -> Result<VkPipeline, String> {
        let frag_shader_module = VkShaderModule::new(device.clone(), "shaders/shader.frag.spv")?;
        let vert_shader_module = VkShaderModule::new(device.clone(), "shaders/shader.vert.spv")?;

        let entrypoint = CString::new("main").unwrap();
        let vert_shader_create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::VERTEX,
            module: vert_shader_module.inner,
            p_name: entrypoint.as_ptr(),
            ..Default::default()
        };

        let frag_shader_create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: frag_shader_module.inner,
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
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
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
            p_set_layouts: &descriptor_set_layout.inner,
            push_constant_range_count: 0,
            p_push_constant_ranges: std::ptr::null(),
            ..Default::default()
        };

        let layout = unsafe {
            device
                .inner
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
            layout,
            render_pass: render_pass.inner,
            subpass: 0,
            ..Default::default()
        };

        let pipeline_create_infos = [pipeline_create_info];
        let pipeline_cache = vk::PipelineCache::null();
        let inner = unsafe {
            device
                .inner
                .create_graphics_pipelines(pipeline_cache, &pipeline_create_infos, None)
                .map_err(|_| format!("Failed to create graphics pipeline"))?
                .remove(0)
        };

        return Ok(VkPipeline {
            device,
            inner,
            layout,
        });
    }
}

impl Drop for VkPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.destroy_pipeline_layout(self.layout, None);
            self.device.inner.destroy_pipeline(self.inner, None);
        }
    }
}
