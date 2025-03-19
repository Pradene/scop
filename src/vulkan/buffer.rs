use crate::vulkan::{VkCommandPool, VkDevice, VkInstance, VkPhysicalDevice, VkQueue};

use ash::vk;
use std::sync::Arc;

pub struct VkBuffer {
    device: Arc<VkDevice>,
    pub inner: vk::Buffer,
    pub size: vk::DeviceSize,
    pub memory: vk::DeviceMemory,
}

impl VkBuffer {
    pub fn new(
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        device: Arc<VkDevice>,
        queue: &VkQueue,
        command: &VkCommandPool,
        data: &[f32],
        usage: vk::BufferUsageFlags,
    ) -> Result<VkBuffer, String> {
        let size = (std::mem::size_of::<f32>() * data.len()) as u64;

        // Create a staging buffer
        let staging_usage = vk::BufferUsageFlags::TRANSFER_SRC;
        let staging_properties =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            instance,
            physical_device,
            &device,
            &size,
            &staging_usage,
            &staging_properties,
        )?;

        // Map memory and copy data
        let data_ptr = unsafe {
            device
                .device
                .map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())
                .unwrap()
        };

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr as *mut f32, data.len());
            device.device.unmap_memory(staging_buffer_memory);
        }

        // Create the target buffer
        let target_properties = vk::MemoryPropertyFlags::DEVICE_LOCAL;
        let (inner, memory) = Self::create_buffer(
            instance,
            physical_device,
            &device,
            &size,
            &usage,
            &target_properties,
        )?;

        // Copy data from the staging buffer to the target buffer
        Self::copy_buffer(
            &device,
            &command,
            &queue.queue,
            &staging_buffer,
            &inner,
            &size,
        );

        // Cleanup staging buffer
        unsafe {
            device.device.destroy_buffer(staging_buffer, None);
            device.device.free_memory(staging_buffer_memory, None);
        }

        Ok(VkBuffer {
            device,
            inner,
            size: data.len() as u64,
            memory,
        })
    }

    pub fn create_buffer(
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        device: &VkDevice,
        size: &vk::DeviceSize,
        usage: &vk::BufferUsageFlags,
        properties: &vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory), String> {
        let create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            size: *size,
            usage: *usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe { device.device.create_buffer(&create_info, None).unwrap() };

        let memory_requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index: Self::find_memory_type(
                instance,
                physical_device,
                memory_requirements.memory_type_bits,
                *properties,
            )
            .unwrap(),

            ..Default::default()
        };

        let buffer_memory = unsafe { device.device.allocate_memory(&allocate_info, None).unwrap() };
        let _ = unsafe {
            device
                .device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .unwrap()
        };

        return Ok((buffer, buffer_memory));
    }

    fn copy_buffer(
        device: &VkDevice,
        command: &VkCommandPool,
        queue: &vk::Queue,
        src: &vk::Buffer,
        dst: &vk::Buffer,
        size: &vk::DeviceSize,
    ) {
        let allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            level: vk::CommandBufferLevel::PRIMARY,
            command_pool: command.pool,
            command_buffer_count: 1,
            ..Default::default()
        };

        let command_buffer = unsafe {
            device
                .device
                .allocate_command_buffers(&allocate_info)
                .unwrap()
                .remove(0)
        };

        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };

        let _ = unsafe {
            device
                .device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap()
        };

        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: *size,
        };

        unsafe {
            device
                .device
                .cmd_copy_buffer(command_buffer, *src, *dst, &[copy_region])
        };
        unsafe { device.device.end_command_buffer(command_buffer).unwrap() };

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            ..Default::default()
        };

        unsafe {
            device
                .device
                .queue_submit(*queue, &[submit_info], vk::Fence::null())
                .unwrap()
        };

        unsafe { device.device.queue_wait_idle(*queue).unwrap() };
        unsafe {
            device
                .device
                .free_command_buffers(command.pool, &[command_buffer]);
        };
    }

    pub fn find_memory_type(
        instance: &VkInstance,
        physical_device: &VkPhysicalDevice,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32, String> {
        let memory_properties = unsafe {
            instance
                .instance
                .get_physical_device_memory_properties(physical_device.physical_device)
        };

        for index in 0..memory_properties.memory_type_count {
            if (type_filter & (1 << index) != 0)
                && ((memory_properties.memory_types[index as usize].property_flags & properties)
                    == properties)
            {
                return Ok(index);
            }
        }

        return Err("Failed to find suitable memory type".to_string());
    }
}

impl Drop for VkBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.device.free_memory(self.memory, None);
            self.device.device.destroy_buffer(self.inner, None);
        }
    }
}
