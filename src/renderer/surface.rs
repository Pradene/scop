use ash::{khr, vk, Entry, Instance};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

use super::VkInstance;

pub struct VkSurface {
    pub loader: khr::surface::Instance,
    pub handle: vk::SurfaceKHR,
}

impl VkSurface {
    pub fn new(window: &Window, entry: &Entry, instance: &VkInstance) -> Result<VkSurface, String> {
        let loader = khr::surface::Instance::new(entry, &instance.handle);
        let handle = VkSurface::create_surface(window, entry, &instance.handle)?;

        return Ok(VkSurface { loader, handle });
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
            self.loader.destroy_surface(self.handle, None);
        }
    }
}
