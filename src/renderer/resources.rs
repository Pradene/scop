use std::collections::HashMap;

use super::{VkCommandPool, VkContext, VkQueue, VkTexture};
use crate::renderer::{Vertex, VkBuffer};
use crate::scene::{Mesh, Object, SubMesh};
use ash::vk;

pub type TextureHandle = usize;
pub type MeshHandle = usize;

pub struct ResourceManager {
    pub textures: Vec<VkTexture>,
    texture_cache: HashMap<String, TextureHandle>,

    pub meshes: Vec<Mesh>,
}

impl ResourceManager {
    pub fn new(
        context: &VkContext,
        graphics_queue: &VkQueue,
        command_pool: &VkCommandPool,
    ) -> Result<Self, String> {
        let white = VkTexture::white(&context, graphics_queue, command_pool)?;

        Ok(Self {
            textures: vec![white],
            texture_cache: HashMap::new(),
            meshes: Vec::new(),
        })
    }

    pub fn white_texture() -> TextureHandle {
        0
    }

    pub fn load_texture(
        &mut self,
        context: &VkContext,
        queue: &VkQueue,
        command_pool: &VkCommandPool,
        path: &str,
    ) -> TextureHandle {
        if let Some(&handle) = self.texture_cache.get(path) {
            return handle;
        }

        match VkTexture::from_path(context, queue, command_pool, path) {
            Ok(tex) => {
                let handle = self.textures.len();
                self.textures.push(tex);
                self.texture_cache.insert(path.to_string(), handle);
                handle
            }
            Err(e) => {
                eprintln!(
                    "Failed to load texture '{}': {} — using white fallback",
                    path, e
                );
                Self::white_texture()
            }
        }
    }

    pub fn get_texture(&self, handle: TextureHandle) -> &VkTexture {
        &self.textures[handle]
    }

    pub fn upload_mesh(
        &mut self,
        context: &VkContext,
        queue: &VkQueue,
        command_pool: &VkCommandPool,
        object: &Object,
    ) -> Result<MeshHandle, String> {
        let mut all_vertices: Vec<Vertex> = Vec::new();
        let mut all_indices: Vec<u32> = Vec::new();
        let mut primitives: Vec<SubMesh> = Vec::new();

        for group in &object.groups {
            let (vertices, indices) = object.get_group_vertices_and_indices(group);
            if indices.is_empty() {
                continue;
            }

            let index_offset = all_indices.len() as u32;
            let index_count = indices.len() as u32;
            let vertex_offset = all_vertices.len() as i32;

            let material = group
                .material
                .as_ref()
                .and_then(|name| object.materials.get(name))
                .cloned()
                .unwrap_or_default();

            // Resolve texture paths relative to the object's base dir
            let tex_diffuse = material
                .map_kd
                .as_deref()
                .map(|p| self.load_texture(context, queue, command_pool, p))
                .unwrap_or(Self::white_texture());

            let tex_specular = material
                .map_ks
                .as_deref()
                .map(|p| self.load_texture(context, queue, command_pool, p))
                .unwrap_or(Self::white_texture());

            let tex_ambient = material
                .map_ka
                .as_deref()
                .map(|p| self.load_texture(context, queue, command_pool, p))
                .unwrap_or(Self::white_texture());

            all_vertices.extend_from_slice(&vertices);
            all_indices.extend_from_slice(&indices);

            primitives.push(SubMesh {
                index_offset,
                index_count,
                vertex_offset,
                material,
                tex_diffuse,
                tex_specular,
                tex_ambient,
            });
        }

        if all_vertices.is_empty() || all_indices.is_empty() {
            return Err("Object has no geometry".to_string());
        }

        let vertex_buffer = VkBuffer::device_local(
            context,
            queue,
            command_pool,
            &all_vertices,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        )?;

        let index_buffer = VkBuffer::device_local(
            context,
            queue,
            command_pool,
            &all_indices,
            vk::BufferUsageFlags::INDEX_BUFFER,
        )?;

        let handle = self.meshes.len();
        self.meshes.push(Mesh {
            vertex_buffer,
            index_buffer,
            primitives,
        });
        Ok(handle)
    }

    pub fn get_mesh(&self, handle: MeshHandle) -> &Mesh {
        &self.meshes[handle]
    }
}
