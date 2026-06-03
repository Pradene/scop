use ash::{khr, vk};

use crate::renderer::{VkInstance, VkPhysicalDevice};

#[derive(Clone)]
pub struct SwapChainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub fn query_swapchain_support(
    handle: &vk::PhysicalDevice,
    surface_loader: &khr::surface::Instance,
    surface: &vk::SurfaceKHR,
) -> Result<SwapChainSupportDetails, String> {
    let capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(*handle, *surface)
            .map_err(|e| format!("Failed to get surface capabilities: {}", e))?
    };

    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(*handle, *surface)
            .map_err(|e| format!("Failed to get surface formats: {}", e))?
    };

    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(*handle, *surface)
            .map_err(|e| format!("Failed to get surface present modes: {}", e))?
    };

    return Ok(SwapChainSupportDetails {
        capabilities,
        formats,
        present_modes,
    });
}

pub fn find_depth_format(
    instance: &VkInstance,
    physical_device: &VkPhysicalDevice,
) -> Result<vk::Format, String> {
    let candidates = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    for format in candidates {
        let props = unsafe {
            instance
                .handle
                .get_physical_device_format_properties(physical_device.handle, format)
        };

        if (props.optimal_tiling_features & vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
            == vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
        {
            return Ok(format);
        }
    }
    Err("Failed to find supported depth format".to_string())
}
