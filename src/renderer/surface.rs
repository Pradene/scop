use ash::{khr, vk, Entry, Instance};
use sdl3::video::Window;

use crate::renderer::VkInstance;

pub struct VkSurface {
    pub loader: khr::surface::Instance,
    pub handle: vk::SurfaceKHR,
}

impl VkSurface {
    pub fn new(window: &Window, entry: &Entry, instance: &VkInstance) -> Result<VkSurface, String> {
        let loader = khr::surface::Instance::new(entry, &instance.handle);
        let handle = VkSurface::create_surface(window, &instance.handle)?;

        return Ok(VkSurface { loader, handle });
    }

    fn create_surface(window: &Window, instance: &Instance) -> Result<vk::SurfaceKHR, String> {
        let vk_instance = instance.handle();
        let surface = unsafe {
            window
                .vulkan_create_surface(vk_instance)
                .expect("Failed to create Vulkan surface")
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
