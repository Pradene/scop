use ash::vk;
use std::sync::Arc;
use std::marker::PhantomData;

use super::find_memory_type;
use super::{VkCommandPool, VkContext, VkDevice, VkQueue};

pub struct VkBuffer<T> {
    device: Arc<VkDevice>,
    pub inner: vk::Buffer,
    pub size: vk::DeviceSize,
    pub memory: vk::DeviceMemory,
    _type: PhantomData<T>,
}

impl<T: Copy> VkBuffer<T> {
    pub fn new(
        context: &VkContext,
        queue: &VkQueue,
        command_pool: &VkCommandPool,
        data: &[T],
        usage: vk::BufferUsageFlags,
    ) -> Result<VkBuffer<T>, String> {
        let device = context.device();
        let size = (std::mem::size_of::<T>() * data.len()) as u64;

        let staging_usage = vk::BufferUsageFlags::TRANSFER_SRC;
        let staging_properties =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        let (staging_buffer, staging_buffer_memory) = VkBuffer::create_buffer(
            context,
            &size,
            &staging_usage,
            &staging_properties,
        )?;

        let data_ptr = unsafe {
            device
                .inner
                .map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|e| format!("Failed to map staging buffer memory: {}", e))?
        };

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr as *mut T, data.len());
            device.inner.unmap_memory(staging_buffer_memory);
        }

        let target_properties = vk::MemoryPropertyFlags::DEVICE_LOCAL;
        let (inner, memory) = VkBuffer::create_buffer(
            context,
            &size,
            &usage,
            &target_properties,
        )?;

        VkBuffer::copy_buffer(
            &device,
            command_pool,
            &queue.inner,
            &staging_buffer,
            &inner,
            &size,
        )?;

        unsafe {
            device.inner.destroy_buffer(staging_buffer, None);
            device.inner.free_memory(staging_buffer_memory, None);
        }

        Ok(VkBuffer {
            device,
            inner,
            size, 
            memory,
            _type: PhantomData,
        })
    }
}

impl VkBuffer<()> {
    pub fn create_buffer(
        context: &VkContext,
        size: &vk::DeviceSize,
        usage: &vk::BufferUsageFlags,
        properties: &vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory), String> {
        let device = context.device();
        let create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            size: *size,
            usage: *usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe {
            device.inner
                .create_buffer(&create_info, None)
                .map_err(|e| format!("Failed to create buffer: {}", e))?
        };

        let memory_requirements = unsafe { device.inner.get_buffer_memory_requirements(buffer) };

        let memory_type_index = find_memory_type(
            context,
            memory_requirements.memory_type_bits,
            *properties,
        )?;

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index,
            ..Default::default()
        };

        let buffer_memory = unsafe {
            device.inner
                .allocate_memory(&allocate_info, None)
                .map_err(|e| format!("Failed to allocate buffer memory: {}", e))?
        };
        
        unsafe {
            device.inner
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .map_err(|e| format!("Failed to bind buffer memory: {}", e))?
        };

        Ok((buffer, buffer_memory))
    }

    fn copy_buffer(
        device: &VkDevice,
        command_pool: &VkCommandPool,
        queue: &vk::Queue,
        src: &vk::Buffer,
        dst: &vk::Buffer,
        size: &vk::DeviceSize,
    ) -> Result<(), String> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            level: vk::CommandBufferLevel::PRIMARY,
            command_pool: command_pool.inner,
            command_buffer_count: 1,
            ..Default::default()
        };

        let mut buffers = unsafe {
            device.inner
                .allocate_command_buffers(&allocate_info)
                .map_err(|e| format!("Failed to allocate staging command buffer: {}", e))?
        };
        
        if buffers.is_empty() {
            return Err("Allocation call succeeded but returned no command buffers".to_string());
        }
        let command_buffer = buffers.remove(0);

        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };

        unsafe {
            device.inner
                .begin_command_buffer(command_buffer, &begin_info)
                .map_err(|e| format!("Failed to begin recording command buffer: {}", e))?
        };

        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: *size,
        };

        unsafe {
            device.inner.cmd_copy_buffer(command_buffer, *src, *dst, &[copy_region]);
            
            device.inner
                .end_command_buffer(command_buffer)
                .map_err(|e| format!("Failed to end command buffer recording: {}", e))?;
        };

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            ..Default::default()
        };

        unsafe {
            device.inner
                .queue_submit(*queue, &[submit_info], vk::Fence::null())
                .map_err(|e| format!("Failed to submit copy queue commands: {}", e))?;

            device.inner
                .queue_wait_idle(*queue)
                .map_err(|e| format!("Failed waiting for queue idle on copy: {}", e))?;

            device.inner.free_command_buffers(command_pool.inner, &[command_buffer]);
        };

        Ok(())
    }
}

impl<T> Drop for VkBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            self.device.inner.free_memory(self.memory, None);
            self.device.inner.destroy_buffer(self.inner, None);
        }
    }
}
