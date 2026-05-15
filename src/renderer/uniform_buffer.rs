use ash::vk;
use std::ffi::c_void;
use std::sync::Arc;

use crate::renderer::{VkBuffer, VkDevice, VkInstance, VkPhysicalDevice};

pub struct UniformBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub mapped: *mut c_void,
    device: Arc<VkDevice>,
}

impl UniformBuffer {
    pub fn new(
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        device: Arc<VkDevice>,
        size: u64,
    ) -> Result<UniformBuffer, String> {
        let usage = vk::BufferUsageFlags::UNIFORM_BUFFER;
        let properties =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        let (buffer, memory) = VkBuffer::create_buffer(
            instance,
            physical_device,
            &device,
            &size,
            &usage,
            &properties,
        )?;

        let mapped = unsafe {
            device
                .inner
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|e| format!("Failed to map uniform buffer memory: {}", e))?
        };

        Ok(UniformBuffer { buffer, memory, mapped, device })
    }

    pub fn write<T: Sized>(&self, data: &T) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                data as *const T as *const u8,
                self.mapped as *mut u8,
                std::mem::size_of::<T>(),
            );
        }
    }
}

impl Drop for UniformBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.unmap_memory(self.memory);
            self.device.inner.destroy_buffer(self.buffer, None);
            self.device.inner.free_memory(self.memory, None);
        }
    }
}