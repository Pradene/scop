use crate::app::App;

use ash_window;

use ash::{khr, vk, Device, Entry, Instance};
use std::collections::BTreeMap;
use winit::{
    event_loop::{ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

// use raw_window_handle::{HasDisplayHandle, HasWindowHandle, HasRawDisplayHandle, HasRawWindowHandle};

pub struct VkContext {
    entry: Entry,
    instance: Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: Device,
    graphics_queue: vk::Queue,
    // present_queue: vk::Queue,
    // surface_loader: khr::surface::Instance,
    surface: vk::SurfaceKHR,
}

#[derive(Clone)]
pub struct QueueFamiliesIndices {
    graphics_family: Option<u32>,
    // present_family: Option<u32>,
}

impl VkContext {
    pub fn new(window: &Window) -> Result<Self, String> {
        let entry = Entry::linked();

        let instance = Self::create_instance(&entry, window)?;
        
        let display_handle = window.display_handle().map_err(|e| format!("Error with display: {}", e))?;
        let window_handle = window.window_handle().map_err(|e| format!("Error with window: {}", e))?;
        
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
            .map_err(|e| format!("Failed to create surface: {}", e))?
        };

        let (physical_device, queue_family) =
            Self::pick_physical_device(&instance)?;

        let logical_device =
            Self::create_logical_device(&instance, &physical_device, &queue_family)?;

        let graphics_queue =
            unsafe { logical_device.get_device_queue(queue_family.graphics_family.unwrap(), 0) };
        // let present_queue =
            // unsafe { logical_device.get_device_queue(queue_family.present_family.unwrap(), 0) };

        return Ok(Self {
            entry,
            instance,
            physical_device,
            logical_device,
            graphics_queue,
            // present_queue,
            // surface_loader,
            surface,
        });
    }

    fn pick_physical_device(
        instance: &Instance,
    ) -> Result<(vk::PhysicalDevice, QueueFamiliesIndices), String> {
        // Enumerate physical devices
        let devices = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(|e| format!("Failed to enumerate physical devices: {:?}", e))?
        };

        if devices.is_empty() {
            return Err("No Vulkan-compatible physical devices found.".to_string());
        }

        let mut candidates: BTreeMap<i32, (vk::PhysicalDevice, QueueFamiliesIndices)> =
            BTreeMap::new();
        for device in &devices {
            // Rate the device and find queue families
            let mut score: i32 = 0;
            let features = unsafe { instance.get_physical_device_features(*device) };
            let properties = unsafe { instance.get_physical_device_properties(*device) };
            let queue_families =
                Self::find_queue_families(instance,device);

            if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                score += 1000;
            }

            // Maximum possible size of textures affects graphics quality
            score += properties.limits.max_image_dimension2_d as i32;

            // Application can't function without geometry shaders or queues
            if features.geometry_shader == 0 || queue_families.graphics_family.is_none() {
                continue; // Skip this device if it doesn't meet the requirements
            }

            candidates.insert(score, (*device, queue_families));
        }

        if let Some((&score, (device, queue_family))) = candidates.iter().rev().next() {
            if score > 0 {
                return Ok((*device, queue_family.clone())); // Clone queue family if necessary
            }
        }

        return Err("Failed to find a suitable GPU.".to_string());
    }

    fn find_queue_families(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> QueueFamiliesIndices {
        let mut graphics_family = None;
        // let mut present_family = None;

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        for (index, queue_family) in queue_families.iter().enumerate() {
            let index = index as u32;

            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && graphics_family.is_none()
            {
                graphics_family = Some(index);
            }

            // let present_support = unsafe {
            //     surface_loader
            //         .get_physical_device_surface_support(*physical_device, index, *surface)
            //         .unwrap()
            // };

            // if present_support && present_family.is_none() {
            //     present_family = Some(index);
            // }

            // if graphics_family.is_some() && present_family.is_some() {
            //     break;
            // }
        }

        return QueueFamiliesIndices {
            graphics_family,
            // present_family,
        };
    }

    fn create_instance(entry: &Entry, window: &Window) -> Result<Instance, String> {
        // Set up Vulkan application information
        let application_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_3,
            ..Default::default()
        };

        let display_handle = window.display_handle().map_err(|e| format!("Error with display: {}", e))?;
        let extension_names = ash_window::enumerate_required_extensions(display_handle.as_raw()).map_err(|e| format!("Error with extension: {}", e))?;

        // Create Vulkan instance
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            ..Default::default()
        };

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|e| format!("Failed to create Vulkan instance: {:?}", e))?
        };

        return Ok(instance);
    }

    fn create_logical_device(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        queue_family: &QueueFamiliesIndices,
    ) -> Result<Device, String> {
        let graphics_family = queue_family.graphics_family.unwrap();
        // let present_family = queue_family.present_family.unwrap();

        let unique_queue_families = vec![graphics_family];

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

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            p_enabled_features: &device_features,
            enabled_extension_count: 0,
            pp_enabled_extension_names: std::ptr::null(),
            ..Default::default()
        };

        let logical_device = unsafe {
            instance
                .create_device(*physical_device, &create_info, None)
                .map_err(|e| format!("Failed to create logical device: {}", e))?
        };

        return Ok(logical_device);
    }
}