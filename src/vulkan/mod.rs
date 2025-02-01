mod buffer;
mod command;
mod context;
mod device;
mod image;
mod instance;
mod physical_device;
mod pipeline;
mod queue;
mod surface;
mod swapchain;
mod sync;
mod vertex;
mod utils;

pub use buffer::*;
pub use command::*;
pub use context::*;
pub use device::*;
pub use image::*;
pub use instance::*;
pub use physical_device::*;
pub use pipeline::*;
pub use queue::*;
pub use surface::*;
pub use swapchain::*;
pub use sync::*;
pub use vertex::*;
pub use utils::*;

use ash::vk;
use std::ffi::CStr;

pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;

pub const VALIDATION_LAYERS_ENABLED: bool = cfg!(debug_assertions);
// pub const VALIDATION_LAYERS_ENABLED: bool = false;
pub const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub const DEVICE_EXTENSIONS: [&CStr; 1] = [vk::KHR_SWAPCHAIN_NAME];
