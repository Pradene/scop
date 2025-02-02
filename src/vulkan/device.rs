use crate::vulkan::DEVICE_EXTENSIONS;
use crate::vulkan::{VkInstance, VkPhysicalDevice};

use ash::{vk, Device};

pub struct VkDevice {
    pub device: Device,
}

impl VkDevice {
    pub fn new(
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
    ) -> Result<VkDevice, String> {
        let graphics_family = physical_device.queue_families.graphics_family.unwrap();
        let present_family = physical_device.queue_families.present_family.unwrap();

        let mut unique_queue_families = vec![graphics_family, present_family];
        unique_queue_families.dedup();

        let queue_priority = 1.0;
        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_families
            .iter()
            .map(|&queue_family| vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: queue_family,
                queue_count: 1,
                p_queue_priorities: &queue_priority,
                ..Default::default()
            })
            .collect();

        let device_features = vk::PhysicalDeviceFeatures::default();

        let device_extensions: Vec<_> = DEVICE_EXTENSIONS
            .iter()
            .map(|extension| extension.as_ptr())
            .collect();

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            p_enabled_features: &device_features,
            enabled_extension_count: device_extensions.len() as u32,
            pp_enabled_extension_names: device_extensions.as_ptr(),
            ..Default::default()
        };

        let device = unsafe {
            instance
                .instance
                .create_device(physical_device.physical_device, &create_info, None)
                .map_err(|e| format!("Failed to create logical device: {}", e))?
        };

        return Ok(VkDevice { device });
    }
}

impl Drop for VkDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
