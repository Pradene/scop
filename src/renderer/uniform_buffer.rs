use ash::vk;
use std::ffi::c_void;
use std::sync::Arc;

use super::{VkBuffer, VkContext, VkDevice};
use crate::math::Mat4;

pub struct Uniforms {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

pub struct UniformBuffer {
    device: Arc<VkDevice>,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub mapped: *mut c_void,
}

unsafe impl Send for UniformBuffer {}
unsafe impl Sync for UniformBuffer {}

impl UniformBuffer {
    pub fn new(context: &VkContext) -> Result<UniformBuffer, String> {
        let device = context.device();
        let size = std::mem::size_of::<Uniforms>() as u64;

        let (buffer, memory) = VkBuffer::create_buffer(
            context,
            &size,
            &vk::BufferUsageFlags::UNIFORM_BUFFER,
            &(vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT),
        )?;

        let mapped = unsafe {
            device
                .inner
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|e| format!("Failed to map uniform buffer memory: {}", e))?
        };

        Ok(Self { device, buffer, memory, mapped })
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