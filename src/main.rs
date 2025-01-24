use scop::app::App;

use ash::{vk, Device, Entry, Instance};
use std::collections::BTreeMap;
use winit::event_loop::{ControlFlow, EventLoop};

pub struct VkContext {
    entry: Entry,
    instance: Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: Device,
    graphics_queue: vk::Queue,
}

pub struct QueueFamiliesIndices {
    graphics_index: Option<u32>,
}

impl VkContext {
    fn new() -> Result<Self, String> {
        let entry = Entry::linked();

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

        let physical_device = Self::pick_physical_device(&instance)?;

        let (logical_device, graphics_queue) = Self::create_logical_device(&instance, &physical_device)?;

        return Ok(Self {
            entry,
            instance,
            physical_device,
            logical_device,
            graphics_queue,
        });
    }

    fn pick_physical_device(instance: &Instance) -> Result<vk::PhysicalDevice, String> {
        // Enumerate physical devices
        let devices = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(|e| format!("Failed to enumerate physical devices: {:?}", e))?
        };

        if devices.is_empty() {
            return Err("No Vulkan-compatible physical devices found.".to_string());
        }

        let mut candidates: BTreeMap<i32, vk::PhysicalDevice> = BTreeMap::new();
        for device in &devices {
            let score = Self::rate_physical_device(instance, device);
            candidates.insert(score, *device);
        }

        if let Some((&score, &best_device)) = candidates.iter().rev().next() {
            if score > 0 {
                return Ok(best_device);
            }
        }

        return Err("Failed to find a suitable GPU.".to_string());
    }

    fn rate_physical_device(instance: &Instance, physical_device: &vk::PhysicalDevice) -> i32 {
        let mut score: i32 = 0;

        let features = unsafe { instance.get_physical_device_features(*physical_device) };
        let properties = unsafe { instance.get_physical_device_properties(*physical_device) };
        let queue_families = Self::find_queue_families(instance, physical_device);

        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            score += 1000;
        }

        // Maximum possible size of textures affects graphics quality
        score += properties.limits.max_image_dimension2_d as i32;

        // Application can't function without geometry shaders or queues
        if features.geometry_shader == 0 || queue_families.graphics_index.is_none() {
            return 0;
        }

        return score;
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

    fn create_logical_device(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<(Device, vk::Queue), String> {
        let queue_family = Self::find_queue_families(instance, physical_device);

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

        let graphics_queue = unsafe { logical_device.get_device_queue(graphics_index, 0) };

        return Ok((logical_device, graphics_queue));
    }
}

fn main() -> Result<(), String> {
    // Create the event loop
    let event_loop = EventLoop::new().map_err(|e| format!("Failed to create event loop: {}", e))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create the application instance
    let mut app = App::default();

    // Load Vulkan entry points
    let _ = VkContext::new().map_err(|e| format!("Failed to create Vulkan Context: {}", e));

    // Run the application
    let _ = event_loop.run_app(&mut app);

    return Ok(());
}
