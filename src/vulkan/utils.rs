use ash::{khr, vk};

use crate::vulkan::{VkInstance, VkPhysicalDevice};

fn find_supported_format(
    instance: &VkInstance,
    physical_device: &VkPhysicalDevice,
    candidates: &Vec<vk::Format>,
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> Result<vk::Format, String> {
    for format in candidates {
        let props = unsafe {
            instance
                .inner
                .get_physical_device_format_properties(physical_device.inner, *format)
        };

        if (tiling == vk::ImageTiling::LINEAR
            && (props.linear_tiling_features & features) == features)
            || (tiling == vk::ImageTiling::OPTIMAL
                && (props.optimal_tiling_features & features) == features)
        {
            return Ok(*format);
        }
    }

    return Err(format!("Failed to find supported format"));
}

pub fn find_depth_format(
    instance: &VkInstance,
    physical_device: &VkPhysicalDevice,
) -> Result<vk::Format, String> {
    let candidates = vec![
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    let tiling = vk::ImageTiling::OPTIMAL;
    let features = vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT;

    return find_supported_format(instance, physical_device, &candidates, tiling, features);
}

#[derive(Clone)]
pub struct SwapChainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub fn query_swapchain_support(
    physical_device: &vk::PhysicalDevice,
    surface_loader: &khr::surface::Instance,
    surface: &vk::SurfaceKHR,
) -> Result<SwapChainSupportDetails, String> {
    let capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(*physical_device, *surface)
            .map_err(|e| format!("Failed to get surface capabilities: {}", e))?
    };

    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(*physical_device, *surface)
            .map_err(|e| format!("Failed to get surface formats: {}", e))?
    };

    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(*physical_device, *surface)
            .map_err(|e| format!("Failed to get surface present modes: {}", e))?
    };

    return Ok(SwapChainSupportDetails {
        capabilities,
        formats,
        present_modes,
    });
}
