use ash::Entry;
use sdl3::video::Window;
use std::sync::Arc;

use super::{VkDevice, VkInstance, VkPhysicalDevice, VkSurface};

pub struct VkContext {
    device: Arc<VkDevice>,
    pub physical_device: VkPhysicalDevice,
    pub surface: VkSurface,
    pub instance: VkInstance,
    pub entry: Entry,
}

impl VkContext {
    pub fn new(window: &Window) -> Result<VkContext, String> {
        let entry = Entry::linked();
        let instance = VkInstance::new(&entry, window)?;
        let surface = VkSurface::new(window, &entry, &instance)?;
        let physical_device = VkPhysicalDevice::new(&instance, &surface)?;
        let device = Arc::new(VkDevice::new(&instance, &physical_device)?);

        Ok(Self {
            entry,
            instance,
            surface,
            physical_device,
            device,
        })
    }

    pub fn device(&self) -> Arc<VkDevice> {
        self.device.clone()
    }

    pub fn graphics_family(&self) -> u32 {
        self.physical_device.queue_families.graphics_family.unwrap()
    }

    pub fn present_family(&self) -> u32 {
        self.physical_device.queue_families.present_family.unwrap()
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        self.device.wait_idle();
    }
}
