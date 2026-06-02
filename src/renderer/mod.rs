mod buffer;
mod command_pool;
mod context;
mod descriptor;
mod device;
mod fence;
mod frame;
mod image;
mod instance;
mod mesh;
mod physical_device;
mod pipeline;
mod queue;
mod render_pass;
mod renderer;
mod resources;
mod semaphore;
mod shaders;
mod surface;
mod swapchain;
mod texture;
mod utils;
mod vertex;

pub use buffer::*;
pub use command_pool::*;
pub use context::*;
pub use descriptor::*;
pub use device::*;
pub use fence::*;
pub use frame::*;
pub use image::*;
pub use instance::*;
pub use mesh::*;
pub use physical_device::*;
pub use pipeline::*;
pub use queue::*;
pub use render_pass::*;
pub use renderer::*;
pub use resources::*;
pub use semaphore::*;
pub use shaders::*;
pub use surface::*;
pub use swapchain::*;
pub use texture::*;
pub use utils::*;
pub use vertex::*;

use ash::vk;
use std::ffi::CStr;

pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;

// pub const VALIDATION_LAYERS_ENABLED: bool = cfg!(debug_assertions);
pub const VALIDATION_LAYERS_ENABLED: bool = false;
pub const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub const DEVICE_EXTENSIONS: [&CStr; 1] = [vk::KHR_SWAPCHAIN_NAME];
