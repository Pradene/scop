use std::collections::HashMap;
use std::sync::Arc;

use ash::vk;

use super::{
    GpuGroup, GpuMaterial, GpuMesh, Vertex, VkBuffer, VkCommandPool, VkContext, VkDevice, VkQueue,
    VkTexture,
};
use crate::parser::ObjFileParser;
use crate::scene::{Material, Mesh};
pub type TextureHandle = usize;
pub type MaterialHandle = usize;
pub type MeshHandle = usize;

pub struct ResourcesManager {
    pub textures: Vec<VkTexture>,
    pub texture_cache: HashMap<String, TextureHandle>,

    pub materials: Vec<GpuMaterial>,

    pub meshes: Vec<GpuMesh>,
    pub mesh_cache: HashMap<String, MeshHandle>,

    upload_queue: VkQueue,
    upload_pool: VkCommandPool,
    device: Arc<VkDevice>,
}

impl ResourcesManager {
    pub fn new(context: Arc<VkContext>) -> Result<Self, String> {
        let upload_queue = VkQueue::new(context.device(), context.graphics_family());
        let upload_pool = VkCommandPool::new(
            context.device(),
            context.graphics_family(),
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        )?;

        let white = VkTexture::white(&context, &upload_queue, &upload_pool)?;
        let default_material = GpuMaterial::default();

        Ok(Self {
            textures: vec![white],
            texture_cache: HashMap::new(),
            materials: vec![default_material],
            meshes: Vec::new(),
            mesh_cache: HashMap::new(),

            upload_pool,
            upload_queue,
            device: context.device(),
        })
    }

    pub fn white_texture() -> TextureHandle {
        0
    }

    pub fn default_material() -> MaterialHandle {
        0
    }

    pub fn load_texture(&mut self, context: &VkContext, path: &str) -> TextureHandle {
        if path.is_empty() {
            return Self::white_texture();
        }

        if let Some(&handle) = self.texture_cache.get(path) {
            return handle;
        }

        match VkTexture::from_path(context, &self.upload_queue, &self.upload_pool, path) {
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

    /// Converts a Material (string paths) into a GpuMaterial (TextureHandles),
    /// uploading any textures that haven't been loaded yet.
    fn resolve_material(&mut self, context: &VkContext, raw: &Material) -> GpuMaterial {
        let map_kd = self.load_texture(context, &raw.map_kd);
        let map_ks = self.load_texture(context, &raw.map_ks);
        let map_ka = self.load_texture(context, &raw.map_ka);

        GpuMaterial {
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

    pub fn save_mesh(&mut self, context: &VkContext, mesh: &Mesh) -> Result<MeshHandle, String> {
        let mut all_vertices: Vec<Vertex> = Vec::new();
        let mut all_indices: Vec<u32> = Vec::new();
        let mut groups: Vec<GpuGroup> = Vec::new();

        let mut materials: Vec<MaterialHandle> = Vec::new();
        for raw in &mesh.materials {
            let handle = self.materials.len();
            let mat = self.resolve_material(context, raw);
            self.materials.push(mat);
            materials.push(handle);
        }

        for group in &mesh.groups {
            if group.indices.is_empty() {
                continue;
            }

            let index_offset = all_indices.len() as u32;
            let vertex_offset = all_vertices.len() as i32;

            let material = group
                .material
                .map(|i| materials[i])
                .unwrap_or(Self::default_material());

            all_vertices.extend_from_slice(&group.vertices);
            all_indices.extend_from_slice(&group.indices);

            groups.push(GpuGroup {
                index_offset,
                index_count: group.indices.len() as u32,
                vertex_offset,
                material,
            });
        }

        if all_vertices.is_empty() || all_indices.is_empty() {
            return Err("Mesh has no geometry".to_string());
        }

        let vertex_buffer = VkBuffer::device_local(
            context,
            &self.upload_queue,
            &self.upload_pool,
            &all_vertices,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        )?;

        let index_buffer = VkBuffer::device_local(
            context,
            &self.upload_queue,
            &self.upload_pool,
            &all_indices,
            vk::BufferUsageFlags::INDEX_BUFFER,
        )?;

        let handle = self.meshes.len();
        self.meshes.push(GpuMesh {
            vertex_buffer,
            index_buffer,
            groups,
        });

        Ok(handle)
    }

    pub fn load_mesh(&mut self, context: &VkContext, path: &str) -> Result<MeshHandle, String> {
        if let Some(&handle) = self.mesh_cache.get(path) {
            return Ok(handle);
        }

        let mesh = ObjFileParser::parse(path)
            .map_err(|e| format!("Failed to parse mesh '{}': {}", path, e))?;

        let handle = self.save_mesh(context, &mesh)?;
        self.mesh_cache.insert(path.to_string(), handle);

        Ok(handle)
    }

    pub fn get_texture(&self, handle: TextureHandle) -> &VkTexture {
        &self.textures[handle]
    }

    pub fn get_material(&self, handle: MaterialHandle) -> &GpuMaterial {
        &self.materials[handle]
    }

    pub fn get_mesh(&self, handle: MeshHandle) -> &GpuMesh {
        &self.meshes[handle]
    }
}
