use std::sync::Arc;
use winit::window::Window;

use super::{
    physical_device::VkPhysicalDevice,
    device::VkDevice,
    surface::VkSurface,
    instance::VkInstance
};

pub struct VkContext {
    device: Arc<VkDevice>,
    pub physical_device: VkPhysicalDevice,
    pub surface: VkSurface,
    pub instance: VkInstance,
}

impl VkContext {
    pub fn new(window: &Window) -> Result<VkContext, String> {
        let instance = VkInstance::new(window)?;
        let surface = VkSurface::new(window, &instance)?;
        let physical_device = VkPhysicalDevice::new(&instance, &surface)?;
        let device = Arc::new(VkDevice::new(&instance, &physical_device)?);

        Ok(Self {
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