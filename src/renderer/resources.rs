use std::collections::HashMap;
use std::sync::Arc;

use ash::vk;

use super::{
    GpuMaterial, GpuMesh, GpuPrimitive, Vertex, VkBuffer, VkCommandPool, VkContext, VkDevice,
    VkQueue, VkTexture,
};
use crate::parser::{Material, Mesh, ObjFileParser};

pub type TextureHandle = usize;
pub type MaterialHandle = usize;
pub type MeshHandle = usize;

pub struct ResourcesManager {
    pub textures: Vec<VkTexture>,
    pub texture_cache: HashMap<String, TextureHandle>,

    pub materials: Vec<GpuMaterial>,
    pub material_cache: HashMap<String, MaterialHandle>,

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
            material_cache: HashMap::new(),
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
        let map_kd = raw.map_kd.as_deref().map(|p| self.load_texture(context, p));
        let map_ks = raw.map_ks.as_deref().map(|p| self.load_texture(context, p));
        let map_ka = raw.map_ka.as_deref().map(|p| self.load_texture(context, p));

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
        let mut primitives: Vec<GpuPrimitive> = Vec::new();

        for (name, raw) in &mesh.materials {
            if !self.material_cache.contains_key(name) {
                let mat = self.resolve_material(context, raw);
                let handle = self.materials.len();
                self.materials.push(mat);
                self.material_cache.insert(name.clone(), handle);
            }
        }

        for submesh in &mesh.submeshes {
            if submesh.indices.is_empty() {
                continue;
            }

            let index_offset = all_indices.len() as u32;
            let vertex_offset = all_vertices.len() as i32;
            let material = submesh
                .material
                .as_deref()
                .and_then(|n| self.material_cache.get(n))
                .copied()
                .unwrap_or(Self::default_material());

            all_vertices.extend_from_slice(&submesh.vertices);
            all_indices.extend_from_slice(&submesh.indices);
            primitives.push(GpuPrimitive {
                index_offset,
                index_count: submesh.indices.len() as u32,
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
            primitives,
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
