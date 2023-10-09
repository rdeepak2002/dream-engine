use wgpu::util::DeviceExt;

use crate::gltf_loader;
use crate::instance::Instance;
use crate::model::Model;
use crate::path_not_found_error::PathNotFoundError;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct RenderMapKey {
    pub model_guid: String,
    pub mesh_index: i32,
}

pub struct RenderStorage {
    pub model_guids: std::collections::HashMap<String, Box<Model>>,
    pub render_map: std::collections::HashMap<RenderMapKey, Vec<Instance>>,
    pub instance_buffer_map: std::collections::HashMap<RenderMapKey, wgpu::Buffer>,
}

impl RenderStorage {
    pub fn queue_for_drawing(&mut self, model_guid: &str, mesh_index: i32, model_mat: Instance) {
        let key = RenderMapKey {
            model_guid: model_guid.parse().unwrap(),
            mesh_index,
        };
        if let std::collections::hash_map::Entry::Vacant(e) = self.render_map.entry(key) {
            // create new array
            e.insert(vec![model_mat]);
        } else {
            let key = RenderMapKey {
                model_guid: model_guid.parse().unwrap(),
                mesh_index,
            };
            // add to existing array
            let current_vec = &mut self.render_map.get_mut(&key).unwrap();
            current_vec.push(model_mat);
        }
    }

    pub fn is_model_stored(&self, model_guid: &str) -> bool {
        self.model_guids.contains_key(model_guid)
    }

    pub fn store_model(
        &mut self,
        model_guid_in: Option<&str>,
        model_path: &str,
        device: &wgpu::Device,
        pbr_material_factors_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<String, PathNotFoundError> {
        let model_guid;
        if model_guid_in.is_some() {
            model_guid = model_guid_in.unwrap();
        } else {
            // TODO: auto-generate guid
            todo!();
        }
        log::debug!("Storing model {} with guid {}", model_path, model_guid);
        let model =
            gltf_loader::read_gltf(model_path, device, pbr_material_factors_bind_group_layout);
        self.model_guids
            .insert(model_guid.parse().unwrap(), Box::new(model));
        Ok(str::parse(model_guid).unwrap())
    }

    pub fn update_mesh_instance_buffer_and_materials(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pbr_material_textures_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        // update internal meshes and materials
        // setup instance buffer for meshes
        for (render_map_key, transforms) in &self.render_map {
            // TODO: this is generating instance buffers every frame, do it only whenever transforms changes
            let instance_data = transforms.iter().map(Instance::to_raw).collect::<Vec<_>>();
            let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });
            // TODO: use Arc<[T]> for faster clone https://www.youtube.com/watch?v=A4cKi7PTJSs&ab_channel=LoganSmith
            self.instance_buffer_map
                .insert(render_map_key.clone(), instance_buffer);
        }

        // TODO: combine this with loop below to make things more concise
        // update materials
        for (render_map_key, _transforms) in &self.render_map {
            let model_map = &mut self.model_guids;
            // TODO: use Arc<[T]> for faster clone https://www.youtube.com/watch?v=A4cKi7PTJSs&ab_channel=LoganSmith
            let model_guid = render_map_key.model_guid.clone();
            let model = model_map
                .get_mut(&*model_guid)
                .unwrap_or_else(|| panic!("no model loaded in renderer with guid {}", model_guid));
            let mesh_index = render_map_key.mesh_index;
            let mesh = model
                .meshes
                .get_mut(mesh_index as usize)
                .unwrap_or_else(|| {
                    panic!("no mesh at index {mesh_index} for model with guid {model_guid}",)
                });
            let material = model
                .materials
                .get_mut(mesh.material)
                .expect("No material at index");
            if !material.loaded() {
                material.update_images();
                material.update_textures(device, queue, pbr_material_textures_bind_group_layout);
                // log::debug!(
                //     "material loading progress: {:.2}%",
                //     material.get_progress() * 100.0
                // );
            }
        }
    }
}
