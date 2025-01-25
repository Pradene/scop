use ash_window;

use std::ffi::{CStr, CString};
use ash::{khr, vk, Device, Entry, Instance};
use std::collections::{BTreeMap, HashSet};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

pub struct VkContext {
    entry: Entry,
    instance: Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    surface_loader: khr::surface::Instance,
    surface: vk::SurfaceKHR,
}

#[derive(Clone)]
pub struct QueueFamiliesIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

const DEVICE_EXTENSIONS: [&CStr; 1] = [
    vk::KHR_SWAPCHAIN_NAME,
];

impl VkContext {
    pub fn new(window: &Window) -> Result<Self, String> {
        let entry = Entry::linked();

        let instance = Self::create_instance(&entry, window)?;

        let display_handle = window
            .display_handle()
            .map_err(|e| format!("Error with display: {}", e))?;
        let window_handle = window
            .window_handle()
            .map_err(|e| format!("Error with window: {}", e))?;

        let surface_loader = khr::surface::Instance::new(&entry, &instance);

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
            Self::choose_physical_device(&instance, &surface_loader, &surface)?;

        let logical_device =
            Self::create_logical_device(&instance, &physical_device, &queue_family)?;

        let graphics_queue =
            unsafe { logical_device.get_device_queue(queue_family.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { logical_device.get_device_queue(queue_family.present_family.unwrap(), 0) };

        return Ok(Self {
            entry,
            instance,
            physical_device,
            logical_device,
            graphics_queue,
            present_queue,
            surface_loader,
            surface,
        });
    }



    fn choose_physical_device(
        instance: &Instance,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> Result<(vk::PhysicalDevice, QueueFamiliesIndices), String> {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(|e| format!("Failed to enumerate physical devices: {:?}", e))?
        };

        if physical_devices.is_empty() {
            return Err("No Vulkan-compatible physical devices found.".to_string());
        }

        let mut candidates: BTreeMap<i32, (vk::PhysicalDevice, QueueFamiliesIndices)> = BTreeMap::new();
        for physical_device in physical_devices {
            let (score, queue_families) = Self::rate_device(instance, surface_loader, surface, physical_device)?;
            if score > 0 {
                if Self::is_device_suitable(instance, &physical_device, &queue_families) {
                    candidates.insert(score, (physical_device, queue_families));
                }
            }
        }

        candidates.iter().rev().next().map_or_else(
            || Err("Failed to find a suitable GPU.".to_string()),
            |(_, (device, queue_family))| Ok((*device, queue_family.clone())),
        )
    }



    fn rate_device(
        instance: &Instance,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
    ) -> Result<(i32, QueueFamiliesIndices), String> {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.get_physical_device_features(physical_device) };
        let queue_families = Self::find_queue_families(instance, &physical_device, surface_loader, surface);

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
        physical_device: &vk::PhysicalDevice,
        queue_families: &QueueFamiliesIndices,
    ) -> bool {
        let device_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(*physical_device)
                .map_err(|e| format!("{}", e))
                .unwrap_or_default()
        };

        let mut required_extensions: HashSet<&CStr> = HashSet::from(DEVICE_EXTENSIONS);

        for extension in device_extensions {
            let extension_name = unsafe {
                CStr::from_ptr(extension.extension_name.as_ptr())
            };

            if required_extensions.contains(extension_name) {
                required_extensions.remove(extension_name);
            }
        }

        required_extensions.is_empty() && queue_families.graphics_family.is_some()
    }



    fn find_queue_families(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
        surface_loader: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> QueueFamiliesIndices {
        let mut graphics_family = None;
        let mut present_family = None;

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        for (index, queue_family) in queue_families.iter().enumerate() {
            let index = index as u32;

            if graphics_family.is_none() && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family = Some(index);
            }

            let present_support = unsafe {
                surface_loader
                    .get_physical_device_surface_support(*physical_device, index, *surface)
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



    fn create_instance(entry: &Entry, window: &Window) -> Result<Instance, String> {
        // Set up Vulkan application information
        let application_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_3,
            ..Default::default()
        };

        let display_handle = window
            .display_handle()
            .map_err(|e| format!("Error with display: {}", e))?;
        
        let extension_names = ash_window::enumerate_required_extensions(display_handle.as_raw())
            .map_err(|e| format!("Error with extension: {}", e))?;

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
        let present_family = queue_family.present_family.unwrap();

        let unique_queue_families = vec![graphics_family, present_family];

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
