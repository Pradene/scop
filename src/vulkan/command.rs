use crate::vulkan::MAX_FRAMES_IN_FLIGHT;
use crate::vulkan::{VkDevice, VkPhysicalDevice};

use ash::vk;

pub struct VkCommand {
    pub pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
}

impl VkCommand {
    pub fn new(physical_device: &VkPhysicalDevice, device: &VkDevice) -> Result<VkCommand, String> {
        let pool = VkCommand::create_pool(device, &physical_device)?;
        let buffers = VkCommand::create_buffers(device, &pool)?;

        return Ok(VkCommand { pool, buffers });
    }

    fn create_pool(
        device: &VkDevice,
        physical_device: &VkPhysicalDevice,
    ) -> Result<vk::CommandPool, String> {
        let create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: physical_device.queue_families.graphics_family.unwrap(),
            ..Default::default()
        };

        let command_pool = unsafe {
            device
                .device
                .create_command_pool(&create_info, None)
                .map_err(|e| format!("Failed to create command pool: {}", e))?
        };

        return Ok(command_pool);
    }

    fn create_buffers(
        device: &VkDevice,
        command_pool: &vk::CommandPool,
    ) -> Result<Vec<vk::CommandBuffer>, String> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: *command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MAX_FRAMES_IN_FLIGHT,
            ..Default::default()
        };

        let command_buffer = unsafe {
            device
                .device
                .allocate_command_buffers(&allocate_info)
                .map_err(|e| format!("Failed to allocate command buffers: {}", e))?
        };

        return Ok(command_buffer);
    }
}
