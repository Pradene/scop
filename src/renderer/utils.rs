use ash::{khr, vk};

use crate::renderer::{VkContext, VkInstance, VkPhysicalDevice};

#[derive(Clone)]
pub struct SwapChainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub fn query_swapchain_support(
    inner: &vk::PhysicalDevice,
    surface_loader: &khr::surface::Instance,
    surface: &vk::SurfaceKHR,
) -> Result<SwapChainSupportDetails, String> {
    let capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(*inner, *surface)
            .map_err(|e| format!("Failed to get surface capabilities: {}", e))?
    };

    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(*inner, *surface)
            .map_err(|e| format!("Failed to get surface formats: {}", e))?
    };

    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(*inner, *surface)
            .map_err(|e| format!("Failed to get surface present modes: {}", e))?
    };

    return Ok(SwapChainSupportDetails {
        capabilities,
        formats,
        present_modes,
    });
}

pub fn find_depth_format(instance: &VkInstance, physical_device: &VkPhysicalDevice) -> Result<vk::Format, String> {
    let candidates = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];
    for format in candidates {
        let props = unsafe {
            instance.inner.get_physical_device_format_properties(physical_device.inner, format)
        };
        if (props.optimal_tiling_features & vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
            == vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
        {
            return Ok(format);
        }
    }
    Err("Failed to find supported depth format".to_string())
}

pub fn find_memory_type(
    context: &VkContext,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> Result<u32, String> {
    let memory_properties = unsafe {
        context.instance
            .inner
            .get_physical_device_memory_properties(context.physical_device.inner)
    };

    for index in 0..memory_properties.memory_type_count {
        if (type_filter & (1 << index) != 0)
            && ((memory_properties.memory_types[index as usize].property_flags & properties)
                == properties)
        {
            return Ok(index);
        }
    }

    Err("Failed to find suitable memory type for requirements".to_string())
}
