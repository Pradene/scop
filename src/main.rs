use scop::app::App;

use ash::{vk, Device, Entry, Instance, khr};
use std::collections::BTreeMap;
use winit::{
    window::Window,
    event_loop::{ControlFlow, EventLoop}
};

pub struct VkContext {
    entry: Entry,
    instance: Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: Device,
    graphics_queue: vk::Queue,
    // surface: vk::SurfaceKHR,
}

#[derive(Clone)]
pub struct QueueFamiliesIndices {
    graphics_index: Option<u32>,
    // present_index: Option<u32>,
}

impl VkContext {
    fn new(window: &Window) -> Result<Self, String> {
        let entry = Entry::linked();

        let instance = Self::create_instance(&entry)?;

        // let surface = khr::surface::Instance::new(&entry, &instance);

        let (physical_device, queue_family) = Self::pick_physical_device(&instance)?;

        let logical_device = Self::create_logical_device(&instance, &physical_device, &queue_family)?;

        let graphics_queue = unsafe { logical_device.get_device_queue(queue_family.graphics_index.unwrap(), 0) };

        return Ok(Self {
            entry,
            instance,
            physical_device,
            logical_device,
            graphics_queue
        });
    }

    fn pick_physical_device(instance: &Instance) -> Result<(vk::PhysicalDevice, QueueFamiliesIndices), String> {
        // Enumerate physical devices
        let devices = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(|e| format!("Failed to enumerate physical devices: {:?}", e))?
        };
    
        if devices.is_empty() {
            return Err("No Vulkan-compatible physical devices found.".to_string());
        }
    
        let mut candidates: BTreeMap<i32, (vk::PhysicalDevice, QueueFamiliesIndices)> = BTreeMap::new();
        for device in &devices {
            // Rate the device and find queue families
            let mut score: i32 = 0;
            let features = unsafe { instance.get_physical_device_features(*device) };
            let properties = unsafe { instance.get_physical_device_properties(*device) };
            let queue_families = Self::find_queue_families(instance, device);
    
            if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                score += 1000;
            }
    
            // Maximum possible size of textures affects graphics quality
            score += properties.limits.max_image_dimension2_d as i32;
    
            // Application can't function without geometry shaders or queues
            if features.geometry_shader == 0 || queue_families.graphics_index.is_none() {
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
        let mut graphics_index = None;
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*physical_device) };

        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_index = Some(index as u32);
            }

            if graphics_index.is_some() {
                break;
            }
        }

        return QueueFamiliesIndices { graphics_index };
    }

    fn create_instance(entry: &Entry) -> Result<Instance, String> {
        // Set up Vulkan application information
        let application_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_3,
            ..Default::default()
        };

        // Create Vulkan instance
        let create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
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
        queue_family: &QueueFamiliesIndices
    ) -> Result<Device, String> {
        let queue_priority = 1.0;
        let graphics_index = queue_family
            .graphics_index
            .ok_or_else(|| "Failed to find a suitable graphics queue family.".to_string())?;

        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: graphics_index,
            queue_count: 1,
            p_queue_priorities: &queue_priority,
            ..Default::default()
        };

        let device_features = vk::PhysicalDeviceFeatures::default();

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_create_info,
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

fn main() -> Result<(), String> {
    // Create the event loop
    let event_loop = EventLoop::new().map_err(|e| format!("Failed to create event loop: {}", e))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create the application instance
    let mut app = App::default();

    // Load Vulkan entry points
    let _ = VkContext::new(&app.get_window().as_ref().unwrap()).map_err(|e| format!("Failed to create Vulkan Context: {}", e));

    // Run the application
    let _ = event_loop.run_app(&mut app);

    return Ok(());
}
