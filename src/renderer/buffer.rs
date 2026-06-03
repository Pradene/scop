use super::{VkCommandPool, VkContext, VkDevice, VkQueue};
use ash::vk;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::sync::Arc;

pub struct VkBuffer<T> {
    device: Arc<VkDevice>,
    pub handle: vk::Buffer,
    pub size: vk::DeviceSize,
    pub memory: vk::DeviceMemory,
    pub mapped: Option<*mut c_void>,
    _type: PhantomData<T>,
}

impl<T: Copy> VkBuffer<T> {
    pub fn device_local(
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

        let (staging_buffer, staging_buffer_memory) =
            create_buffer(context, &size, &staging_usage, &staging_properties)?;

        let data_ptr = unsafe {
            device
                .handle
                .map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|e| format!("Failed to map staging buffer memory: {}", e))?
        };

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), data_ptr as *mut T, data.len());
            device.handle.unmap_memory(staging_buffer_memory);
        }

        let target_properties = vk::MemoryPropertyFlags::DEVICE_LOCAL;
        let (handle, memory) = create_buffer(context, &size, &usage, &target_properties)?;

        copy_buffer(
            &device,
            command_pool,
            &queue,
            &staging_buffer,
            &handle,
            &size,
        )?;

        unsafe {
            device.handle.destroy_buffer(staging_buffer, None);
            device.handle.free_memory(staging_buffer_memory, None);
        }

        Ok(VkBuffer {
            device,
            handle,
            size,
            memory,
            mapped: None,
            _type: PhantomData,
        })
    }
}

impl<T> VkBuffer<T> {
    pub fn host_visible(
        context: &VkContext,
        count: usize,
        usage: vk::BufferUsageFlags,
    ) -> Result<Self, String> {
        let device = context.device();
        let size = (std::mem::size_of::<T>() * count) as u64;

        let properties =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let (handle, memory) = create_buffer(context, &size, &usage, &properties)?;

        let mapped = unsafe {
            device
                .handle
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|e| format!("Failed to map host-visible buffer memory: {}", e))?
        };

        Ok(Self {
            device,
            handle,
            size,
            memory,
            mapped: Some(mapped),
            _type: PhantomData,
        })
    }

    pub fn write(&self, data: &[T]) {
        let ptr = self.mapped.expect("Cannot write to a non-mapped buffer!");
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut T, data.len());
        }
    }
}

impl<T> Drop for VkBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            if self.mapped.is_some() {
                self.device.handle.unmap_memory(self.memory);
            }
            self.device.handle.free_memory(self.memory, None);
            self.device.handle.destroy_buffer(self.handle, None);
        }
    }
}

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
        device
            .handle
            .create_buffer(&create_info, None)
            .map_err(|e| format!("Failed to create buffer: {}", e))?
    };

    let memory_requirements = unsafe { device.handle.get_buffer_memory_requirements(buffer) };

    let memory_type_index = context
        .physical_device
        .find_memory_type(memory_requirements.memory_type_bits, *properties)?;

    let allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        allocation_size: memory_requirements.size,
        memory_type_index,
        ..Default::default()
    };

    let buffer_memory = unsafe {
        device
            .handle
            .allocate_memory(&allocate_info, None)
            .map_err(|e| format!("Failed to allocate buffer memory: {}", e))?
    };

    unsafe {
        device
            .handle
            .bind_buffer_memory(buffer, buffer_memory, 0)
            .map_err(|e| format!("Failed to bind buffer memory: {}", e))?
    };

    Ok((buffer, buffer_memory))
}

fn copy_buffer(
    device: &VkDevice,
    command_pool: &VkCommandPool,
    queue: &VkQueue,
    src: &vk::Buffer,
    dst: &vk::Buffer,
    size: &vk::DeviceSize,
) -> Result<(), String> {
    let allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        level: vk::CommandBufferLevel::PRIMARY,
        command_pool: command_pool.handle,
        command_buffer_count: 1,
        ..Default::default()
    };

    let mut buffers = unsafe {
        device
            .handle
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
        device
            .handle
            .begin_command_buffer(command_buffer, &begin_info)
            .map_err(|e| format!("Failed to begin recording command buffer: {}", e))?
    };

    let copy_region = vk::BufferCopy {
        src_offset: 0,
        dst_offset: 0,
        size: *size,
    };

    unsafe {
        device
            .handle
            .cmd_copy_buffer(command_buffer, *src, *dst, &[copy_region]);

        device
            .handle
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
        device
            .handle
            .queue_submit(queue.handle, &[submit_info], vk::Fence::null())
            .map_err(|e| format!("Failed to submit copy queue commands: {}", e))?;

        device
            .handle
            .queue_wait_idle(queue.handle)
            .map_err(|e| format!("Failed waiting for queue idle on copy: {}", e))?;

        device
            .handle
            .free_command_buffers(command_pool.handle, &[command_buffer]);
    };

    Ok(())
}
