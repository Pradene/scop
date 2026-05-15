use super::VkDevice;
use ash::vk;
use std::fs::File;
use std::sync::Arc;

pub struct VkShaderModule {
    device: Arc<VkDevice>,
    pub inner: vk::ShaderModule,
}

impl VkShaderModule {
    pub fn new(device: Arc<VkDevice>, path: &str) -> Result<VkShaderModule, String> {
        let code = VkShaderModule::read_spv_file(path)?;

        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            code_size: code.len() * std::mem::size_of::<u32>(),
            p_code: code.as_ptr(),
            ..Default::default()
        };

        let inner = unsafe {
            device
                .inner
                .create_shader_module(&create_info, None)
                .map_err(|e| format!("Failed to create shader module: {}", e))?
        };

        return Ok(VkShaderModule { device, inner });
    }

    fn read_spv_file(path: &str) -> Result<Vec<u32>, String> {
        let mut file =
            File::open(path).map_err(|e| format!("Failed to open file {}: {}", path, e))?;

        let content = ash::util::read_spv(&mut file)
            .map_err(|e| format!("Failed to decode SPIR-V file {}: {}", path, e))?;

        return Ok(content);
    }
}

impl Drop for VkShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.destroy_shader_module(self.inner, None);
        }
    }
}
