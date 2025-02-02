use crate::vulkan::{VALIDATION_LAYERS, VALIDATION_LAYERS_ENABLED};

use std::ffi::CString;

use ash::{vk, Entry, Instance};

use ash_window;
use winit::{raw_window_handle::HasDisplayHandle, window::Window};

pub struct VkInstance {
    pub entry: Entry,
    pub instance: Instance,
}

impl VkInstance {
    pub fn new(window: &Window) -> Result<VkInstance, String> {
        let entry = Entry::linked();
        let instance = VkInstance::create_instance(&entry, window)?;

        return Ok(VkInstance { entry, instance });
    }

    fn check_validation_layer_support(entry: &Entry) -> bool {
        let available_layers: Vec<vk::LayerProperties>;

        unsafe {
            match entry.enumerate_instance_layer_properties() {
                Ok(layers_properties) => available_layers = layers_properties,
                Err(_) => return false,
            }
        }

        for layer_name in VALIDATION_LAYERS {
            let mut found = false;

            for layer_properties in &available_layers {
                let layer_properties: Vec<u8> = layer_properties
                    .layer_name
                    .iter()
                    .map(|&b| b as u8)
                    .collect();

                if layer_name.as_bytes() == layer_properties.as_slice() {
                    found = true;
                    break;
                }
            }

            if found == false {
                return false;
            }
        }

        return true;
    }

    fn create_instance(entry: &Entry, window: &Window) -> Result<Instance, String> {
        // if VALIDATION_LAYERS_ENABLED && !Self::check_validation_layer_support(entry) {
        //     return Err("Validation layers not supported".to_string());
        // }

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

        let validation_layers: Vec<CString> = VALIDATION_LAYERS
            .iter()
            .map(|&layer| CString::new(layer).unwrap())
            .collect();

        // Get raw pointers to the CStrings
        let validation_layers: Vec<*const i8> = validation_layers
            .iter()
            .map(|layer| layer.as_ptr())
            .collect();

        // Create Vulkan instance
        let mut create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            ..Default::default()
        };

        if VALIDATION_LAYERS_ENABLED {
            create_info.pp_enabled_layer_names = validation_layers.as_ptr();
            create_info.enabled_layer_count = validation_layers.len() as u32;
        }

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|e| format!("Failed to create Vulkan instance: {:?}", e))?
        };

        return Ok(instance);
    }
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
