use std::u64;

use crate::vulkan::VkContext;
use ash::vk;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    context: Option<VkContext>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            println!("Creating window...");
            let window_attributes = Window::default_attributes().with_title("Scop");
            let window = event_loop
                .create_window(window_attributes)
                .expect("Failed to create window");
            println!("Window created successfully.");

            println!("Initializing Vulkan context...");
            match VkContext::new(&window) {
                Ok(context) => {
                    self.context = Some(context);
                    println!("Vulkan context initialized successfully.");
                }
                Err(e) => {
                    println!("Failed to create Vulkan context: {:?}", e);
                    return;
                }
            }

            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(context) = &mut self.context {
                    self.draw_frame()
                }

                self.window.as_ref().unwrap().request_redraw();
            }

            _ => (),
        }
    }
}

impl App {
    fn draw_frame(&self) {
        let context = self.context.as_ref().unwrap();

        unsafe { context.logical_device.wait_for_fences(&[context.fence], true, u64::MAX) };
        unsafe { context.logical_device.reset_fences(&[context.fence]) };
        
        let image_index = unsafe { context.swapchain_loader.acquire_next_image(context.swapchain, u64::MAX, context.image_available_semaphore, context.fence).unwrap() };
        unsafe { context.logical_device.reset_command_buffer(context.command_buffer, vk::CommandBufferResetFlags::empty()) };
        context.record_command_buffer(&context.command_buffer, image_index.0);

        let signal_semaphores = [context.render_finished_semaphore];
        let wait_semaphores = [context.image_available_semaphore];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &context.command_buffer,
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };

        unsafe { context.logical_device.queue_submit(context.graphics_queue, &[submit_info], context.fence) };

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: &context.swapchain,
            p_image_indices: &image_index.0,
            p_results: std::ptr::null_mut(),
            ..Default::default()
        };

        unsafe { context.swapchain_loader.queue_present(context.present_queue, &present_info) };

        println!("Hello world");
    }
}
