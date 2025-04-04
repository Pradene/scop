use ash::{khr, vk, Entry, Instance};

use crate::vulkan::VkInstance;
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

pub struct VkSurface {
    pub loader: khr::surface::Instance,
    pub inner: vk::SurfaceKHR,
}

impl VkSurface {
    pub fn new(window: &Window, instance: &VkInstance) -> Result<VkSurface, String> {
        let loader = khr::surface::Instance::new(&instance.entry, &instance.inner);
        let inner = VkSurface::create_surface(window, &instance.entry, &instance.inner)?;

        return Ok(VkSurface { loader, inner });
    }

    fn create_surface(
        window: &Window,
        entry: &Entry,
        instance: &Instance,
    ) -> Result<vk::SurfaceKHR, String> {
        let display_handle = window
            .display_handle()
            .map_err(|e| format!("Error with display: {}", e))?;
        let window_handle = window
            .window_handle()
            .map_err(|e| format!("Error with window: {}", e))?;

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

        return Ok(surface);
    }
}

impl Drop for VkSurface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.inner, None);
        }
    }
}
