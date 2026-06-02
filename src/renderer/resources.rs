use std::collections::HashMap;
use std::path::Path;

use super::{VkCommandPool, VkContext, VkQueue, VkTexture};
use crate::math::Mat4;
use crate::parser::ObjectParser;
use crate::renderer::{Mesh, SubMesh, Vertex, VkBuffer};
use crate::scene::{Material, Object, RawMaterial};
use ash::vk;

pub type TextureHandle = usize;
pub type MaterialHandle = usize;
pub type MeshHandle = usize;

pub struct ResourceManager {
    pub textures: Vec<VkTexture>,
    pub texture_cache: HashMap<String, TextureHandle>,

    pub materials: Vec<Material>,
    pub material_cache: HashMap<String, MaterialHandle>,

    pub meshes: Vec<Mesh>,
    pub mesh_cache: HashMap<String, MeshHandle>,
}

impl ResourceManager {
    pub fn new(
        context: &VkContext,
        graphics_queue: &VkQueue,
        command_pool: &VkCommandPool,
    ) -> Result<Self, String> {
        let white = VkTexture::white(&context, graphics_queue, command_pool)?;
        let default_material = Material::default();

        Ok(Self {
            textures: vec![white],
            texture_cache: HashMap::new(),
            materials: vec![default_material],
            material_cache: HashMap::new(),
            meshes: Vec::new(),
            mesh_cache: HashMap::new(),
        })
    }

    pub fn white_texture() -> TextureHandle {
        0
    }

    pub fn default_material() -> MaterialHandle {
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

    /// Converts a RawMaterial (string paths) into a Material (TextureHandles),
    /// uploading any textures that haven't been loaded yet.
    fn resolve_material(
        &mut self,
        context: &VkContext,
        queue: &VkQueue,
        command_pool: &VkCommandPool,
        raw: &RawMaterial,
    ) -> Material {
        let map_kd = raw
            .map_kd
            .as_deref()
            .map(|p| self.load_texture(context, queue, command_pool, p));

        let map_ks = raw
            .map_ks
            .as_deref()
            .map(|p| self.load_texture(context, queue, command_pool, p));

        let map_ka = raw
            .map_ka
            .as_deref()
            .map(|p| self.load_texture(context, queue, command_pool, p));

        Material {
            ka: raw.ka,
            kd: raw.kd,
            ks: raw.ks,
            ns: raw.ns,
            ni: raw.ni,
            dissolve: raw.dissolve,
            illum: raw.illum,
            map_kd,
            map_ks,
            map_ka,
        }
    }

    pub fn load_object<P: AsRef<Path>>(
        &mut self,
        context: &VkContext,
        queue: &VkQueue,
        command_pool: &VkCommandPool,
        path: P,
    ) -> Result<(MeshHandle, std::ops::Range<TextureHandle>), String> {
        let path_str = path.as_ref().to_string_lossy().to_string();

        if let Some(&handle) = self.mesh_cache.get(&path_str) {
            return Ok((handle, 0..0)); // already registered
        }

        let textures_before = self.textures.len();
        let object = ObjectParser::parse(&path)?;
        let handle = self.upload_object(context, queue, command_pool, &object)?;
        let textures_after = self.textures.len();

        self.mesh_cache.insert(path_str, handle);
        Ok((handle, textures_before..textures_after))
    }

    pub fn upload_object(
        &mut self,
        context: &VkContext,
        queue: &VkQueue,
        command_pool: &VkCommandPool,
        object: &Object,
    ) -> Result<MeshHandle, String> {
        let mut all_vertices: Vec<Vertex> = Vec::new();
        let mut all_indices: Vec<u32> = Vec::new();
        let mut primitives: Vec<SubMesh> = Vec::new();

        // Resolve RawMaterial (string paths) → Material (TextureHandles)
        for (name, raw) in &object.materials {
            if self.material_cache.contains_key(name) {
                continue; // already loaded (e.g. shared MTL across objects)
            }
            let material = self.resolve_material(context, queue, command_pool, raw);
            let handle = self.materials.len();
            self.materials.push(material);
            self.material_cache.insert(name.clone(), handle);
        }

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
                .as_deref()
                .and_then(|name| self.material_cache.get(name))
                .copied()
                .unwrap_or(Self::default_material());

            all_vertices.extend_from_slice(&vertices);
            all_indices.extend_from_slice(&indices);

            primitives.push(SubMesh {
                index_offset,
                index_count,
                vertex_offset,
                material,
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
            transform: Mat4::identity(),
        });

        Ok(handle)
    }

    pub fn get_texture(&self, handle: TextureHandle) -> &VkTexture {
        &self.textures[handle]
    }

    pub fn get_material(&self, handle: MaterialHandle) -> &Material {
        &self.materials[handle]
    }

    pub fn get_mesh(&mut self, handle: MeshHandle) -> &mut Mesh {
        &mut self.meshes[handle]
    }
}
