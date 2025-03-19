use std::collections::{BTreeMap, HashSet};
use std::ffi::CStr;

use ash::{khr, vk, Instance};

use crate::vulkan::DEVICE_EXTENSIONS;
use crate::vulkan::{QueueFamiliesIndices, SwapChainSupportDetails, VkInstance, VkSurface};

pub struct VkPhysicalDevice {
    pub inner: vk::PhysicalDevice,
    pub queue_families: QueueFamiliesIndices,
    pub swapchain_support: SwapChainSupportDetails,
}

impl VkPhysicalDevice {
    pub fn new(instance: &VkInstance, surface: &VkSurface) -> Result<VkPhysicalDevice, String> {
        let (inner, queue_families, swapchain_support) = VkPhysicalDevice::choose_physical_device(
            &instance.inner,
            &surface.loader,
            &surface.inner,
        )?;

        return Ok(VkPhysicalDevice {
            inner,
            queue_families,
            swapchain_support,
        });
    }

    fn choose_physical_device(
        instance: &Instance,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> Result<
        (
            vk::PhysicalDevice,
            QueueFamiliesIndices,
            SwapChainSupportDetails,
        ),
        String,
    > {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(|e| format!("Failed to enumerate physical devices: {:?}", e))?
        };

        if physical_devices.is_empty() {
            return Err("No Vulkan-compatible physical devices found.".to_string());
        }

        let mut candidates: BTreeMap<
            i32,
            (
                vk::PhysicalDevice,
                QueueFamiliesIndices,
                SwapChainSupportDetails,
            ),
        > = BTreeMap::new();

        for inner in physical_devices {
            let (score, queue_families) =
                Self::rate_device(instance, surface_loader, surface, &inner)?;

            let swapchain_support;
            match VkPhysicalDevice::query_swapchain_support(&inner, surface_loader, surface) {
                Ok(value) => swapchain_support = value,
                Err(e) => return Err(format!("Swapchain not supported: {}", e)),
            }

            if score > 0 {
                if Self::is_device_suitable(instance, &inner, &queue_families, &swapchain_support) {
                    candidates.insert(score, (inner, queue_families, swapchain_support));
                }
            }
        }

        return candidates.iter().rev().next().map_or_else(
            || Err("Failed to find a suitable GPU.".to_string()),
            |(_, (device, queue_family, swapchain_support))| {
                Ok((*device, queue_family.clone(), swapchain_support.clone()))
            },
        );
    }

    fn rate_device(
        instance: &Instance,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
        inner: &vk::PhysicalDevice,
    ) -> Result<(i32, QueueFamiliesIndices), String> {
        let properties = unsafe { instance.get_physical_device_properties(*inner) };
        let features = unsafe { instance.get_physical_device_features(*inner) };
        let queue_families = Self::find_queue_families(instance, &inner, surface_loader, surface);

        let mut score = 0;

        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        }

        score += properties.limits.max_image_dimension2_d as i32;

        if features.geometry_shader == 0 || queue_families.graphics_family.is_none() {
            return Ok((0, queue_families)); // Skip if no geometry shader or graphics family
        }

        return Ok((score, queue_families));
    }

    fn is_device_suitable(
        instance: &Instance,
        inner: &vk::PhysicalDevice,
        queue_families: &QueueFamiliesIndices,
        swapchain_support: &SwapChainSupportDetails,
    ) -> bool {
        let device_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(*inner)
                .map_err(|e| format!("{}", e))
                .unwrap_or_default()
        };

        let mut required_extensions: HashSet<&CStr> = HashSet::from(DEVICE_EXTENSIONS);

        for extension in device_extensions {
            let extension_name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };

            if required_extensions.contains(extension_name) {
                required_extensions.remove(extension_name);
            }
        }

        return required_extensions.is_empty()
            && queue_families.graphics_family.is_some()
            && !swapchain_support.formats.is_empty()
            && !swapchain_support.present_modes.is_empty();
    }

    fn find_queue_families(
        instance: &Instance,
        inner: &vk::PhysicalDevice,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> QueueFamiliesIndices {
        let mut graphics_family = None;
        let mut present_family = None;

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*inner) };

        for (index, queue_family) in queue_families.iter().enumerate() {
            let index = index as u32;

            let graphics_flags = queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
            if graphics_family.is_none() && graphics_flags {
                graphics_family = Some(index);
            }

            let present_support = unsafe {
                surface_loader
                    .get_physical_device_surface_support(*inner, index, *surface)
                    .unwrap()
            };

            if present_support && present_family.is_none() {
                present_family = Some(index);
            }

            if graphics_family.is_some() && present_family.is_some() {
                break;
            }
        }

        return QueueFamiliesIndices {
            graphics_family,
            present_family,
        };
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
}
