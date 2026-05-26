use std::ffi::CString;

use ash::{vk, Entry, Instance};
use sdl3::video::Window;

use super::{VALIDATION_LAYERS, VALIDATION_LAYERS_ENABLED};

pub struct VkInstance {
    pub handle: Instance,
}

impl VkInstance {
    pub fn new(entry: &Entry, window: &Window) -> Result<VkInstance, String> {
        let handle = VkInstance::create_instance(&entry, window)?;

        return Ok(VkInstance { handle });
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
        if VALIDATION_LAYERS_ENABLED && !Self::check_validation_layer_support(entry) {
            return Err("Validation layers not supported".to_string());
        }

        let application_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_3,
            ..Default::default()
        };

        let extension_names = window.vulkan_instance_extensions().unwrap();

        let extension_cstrings: Vec<CString> = extension_names
            .iter()
            .map(|s| CString::new(s.as_str()).unwrap())
            .collect();

        let extension_names_raw: Vec<*const i8> =
            extension_cstrings.iter().map(|s| s.as_ptr()).collect();

        let validation_layers_cstring: Vec<CString> = VALIDATION_LAYERS
            .iter()
            .map(|&layer| CString::new(layer).unwrap_or_default())
            .collect();

        let validation_layers: Vec<*const i8> = validation_layers_cstring
            .iter()
            .map(|layer| layer.as_ptr())
            .collect();

        let mut create_info = vk::InstanceCreateInfo {
            p_application_info: &application_info,
            pp_enabled_extension_names: extension_names_raw.as_ptr(),
            enabled_extension_count: extension_names_raw.len() as u32,
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

        Ok(instance)
    }
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_instance(None);
        }
    }
}
