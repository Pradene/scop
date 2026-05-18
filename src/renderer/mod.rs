mod buffer;
mod command_pool;
mod context;
mod descriptor;
mod device;
mod fence;
mod image;
mod instance;
mod physical_device;
mod pipeline;
mod queue;
mod render_pass;
mod renderer;
mod semaphore;
mod shaders;
mod surface;
mod swapchain;
mod utils;
mod vertex;

pub use buffer::*;
pub use command_pool::*;
pub use context::*;
pub use descriptor::*;
pub use device::*;
pub use fence::*;
pub use image::*;
pub use instance::*;
pub use physical_device::*;
pub use pipeline::*;
pub use queue::*;
pub use render_pass::*;
pub use renderer::*;
pub use semaphore::*;
pub use shaders::*;
pub use surface::*;
pub use swapchain::*;
pub use utils::*;
pub use vertex::*;

use ash::vk;
use std::ffi::CStr;

pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;

// pub const VALIDATION_LAYERS_ENABLED: bool = cfg!(debug_assertions);
pub const VALIDATION_LAYERS_ENABLED: bool = false;
pub const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub const DEVICE_EXTENSIONS: [&CStr; 1] = [vk::KHR_SWAPCHAIN_NAME];
