/**********************************************************************************
 *  Dream is a software for developing real-time 3D experiences.
 *  Copyright (C) 2023 Deepak Ramalignam
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Affero General Public License as published
 *  by the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Affero General Public License for more details.
 *
 *  You should have received a copy of the GNU Affero General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 **********************************************************************************/

use std::iter;

use nalgebra::{Point3, Vector3};
use wgpu::util::DeviceExt;
use wgpu::{CompositeAlphaMode, PresentMode};
use winit::dpi::PhysicalSize;

use crate::camera_uniform::CameraUniform;
use crate::deferred_rendering_tech::DeferredRenderingTech;
use crate::forward_rendering_tech::ForwardRenderingTech;
use crate::instance::Instance;
use crate::model::Model;
use crate::path_not_found_error::PathNotFoundError;
use crate::pbr_bind_groups_and_layouts::PbrBindGroupsAndLayouts;
use crate::{camera, gltf_loader, texture};

#[cfg(not(feature = "wgpu/webgl"))]
pub fn is_webgpu_enabled() -> bool {
    true
}

#[cfg(feature = "wgpu/webgl")]
pub fn is_webgpu_enabled() -> bool {
    false
}

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

pub struct RendererWgpu {
    pub surface: Option<wgpu::Surface>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub frame_texture_view: Option<wgpu::TextureView>,
    pub preferred_texture_format: Option<wgpu::TextureFormat>,
    // TODO: combine below 4 camera variables
    render_storage: RenderStorage,
    camera: camera::Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    frame_texture: texture::Texture,
    depth_texture: texture::Texture,
    deferred_rendering_tech: DeferredRenderingTech,
    forward_rendering_tech: ForwardRenderingTech,
    pbr_bind_groups_and_layouts: PbrBindGroupsAndLayouts,
}

impl RendererWgpu {
    pub async fn default(window: Option<&winit::window::Window>) -> Self {
        let preferred_texture_format;

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let size;
        let surface;

        if window.is_some() {
            size = window.as_ref().unwrap().inner_size();
            // # Safety
            //
            // The surface needs to live as long as the window that created it.
            // State owns the window so this should be safe.
            surface = Some(unsafe { instance.create_surface(window.as_ref().unwrap()) }.unwrap());
        } else {
            size = PhysicalSize::new(100, 100);
            surface = None;
        }

        let adapter;
        if surface.is_some() {
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: surface.as_ref(),
                    force_fallback_adapter: false,
                })
                .await
                .expect("(1) Unable to request for adapter to initialize renderer");
        } else {
            adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: None,
                    force_fallback_adapter: true,
                })
                .await
                .expect("(2) Unable to request for adapter to initialize renderer");
        }

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map(|(device, queue)| -> (wgpu::Device, wgpu::Queue) { (device, queue) })
            .unwrap();

        let config;

        if surface.is_some() {
            let surface_caps = surface.as_ref().unwrap().get_capabilities(&adapter);

            // Shader code in this tutorial assumes an Srgb surface texture. Using a different
            // one will result all the colors comming out darker. If you want to support non
            // Srgb surfaces, you'll need to account for that when drawing to the frame.
            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);

            preferred_texture_format = Some(surface_format);

            config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
            };
        } else {
            // case where there is no surface
            cfg_if::cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    preferred_texture_format = Some(wgpu::TextureFormat::Bgra8Unorm);
                    config = wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: preferred_texture_format.unwrap(),
                        width: size.width,
                        height: size.height,
                        present_mode: PresentMode::AutoNoVsync,
                        alpha_mode: CompositeAlphaMode::Auto,
                        view_formats: vec![],
                    };
                } else {
                    preferred_texture_format = Some(wgpu::TextureFormat::Bgra8UnormSrgb);
                    config = wgpu::SurfaceConfiguration {
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        format: preferred_texture_format.unwrap(),
                        width: size.width,
                        height: size.height,
                        present_mode: PresentMode::AutoNoVsync,
                        alpha_mode: CompositeAlphaMode::Auto,
                        view_formats: vec![],
                    };
                }
            }
        }

        let camera = camera::Camera::new(
            Point3::new(5.0, 5.0, 5.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            config.width as f32 / config.height as f32,
            std::f32::consts::FRAC_PI_4,
            0.01,
            1000.0,
        );

        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            config.width,
            config.height,
            "depth_texture",
        );

        let frame_texture = texture::Texture::create_frame_texture(
            &device,
            config.width,
            config.height,
            "frame_texture",
            preferred_texture_format.unwrap(),
        );

        let pbr_rendering_tech = PbrBindGroupsAndLayouts::new(&device, &camera_bind_group_layout);
        let deferred_rendering_tech = DeferredRenderingTech::new(
            &device,
            &pbr_rendering_tech.render_pipeline_pbr_layout,
            config.format,
            config.width,
            config.height,
        );
        let forward_rendering_tech = ForwardRenderingTech::new(
            &device,
            &pbr_rendering_tech.render_pipeline_pbr_layout,
            config.format,
        );

        Self {
            surface,
            device,
            queue,
            config,
            depth_texture,
            frame_texture,
            frame_texture_view: None,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            render_storage: RenderStorage {
                model_guids: Default::default(),
                render_map: Default::default(),
                instance_buffer_map: Default::default(),
            },
            preferred_texture_format,
            deferred_rendering_tech,
            forward_rendering_tech,
            pbr_bind_groups_and_layouts: pbr_rendering_tech,
        }
    }

    pub fn set_camera_aspect_ratio(&mut self, new_aspect_ratio: f32) {
        if self.camera.aspect != new_aspect_ratio {
            self.camera.aspect = new_aspect_ratio;
            self.camera.build_view_projection_matrix();
            self.camera_uniform.update_view_proj(&self.camera);
            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
        }
    }

    pub fn resize(&mut self, new_size: Option<PhysicalSize<u32>>) {
        if let Some(new_size) = new_size {
            if new_size.width > 0 && new_size.height > 0 {
                self.config.width = new_size.width;
                self.config.height = new_size.height;
            }
        }
        if self.surface.is_some() {
            self.surface
                .as_mut()
                .unwrap()
                .configure(&self.device, &self.config);
        }
        // resize frame textures
        self.frame_texture = texture::Texture::create_frame_texture(
            &self.device,
            self.config.width,
            self.config.height,
            "frame_texture",
            self.preferred_texture_format.unwrap(),
        );
        // resize depth texture for g buffers
        self.depth_texture = texture::Texture::create_depth_texture(
            &self.device,
            self.config.width,
            self.config.height,
            "depth_texture",
        );
    }

    pub fn draw_mesh(&mut self, model_guid: &str, mesh_index: i32, model_mat: Instance) {
        let key = RenderMapKey {
            model_guid: model_guid.parse().unwrap(),
            mesh_index,
        };
        {
            if let std::collections::hash_map::Entry::Vacant(e) =
                self.render_storage.render_map.entry(key)
            {
                // create new array
                e.insert(vec![model_mat]);
            } else {
                let key = RenderMapKey {
                    model_guid: model_guid.parse().unwrap(),
                    mesh_index,
                };
                // add to existing array
                let current_vec = &mut self.render_storage.render_map.get_mut(&key).unwrap();
                current_vec.push(model_mat);
            }
        }
    }

    pub fn is_model_stored(&self, model_guid: &str) -> bool {
        self.render_storage.model_guids.contains_key(model_guid)
    }

    pub fn store_model(
        &mut self,
        model_guid_in: Option<&str>,
        model_path: &str,
    ) -> Result<String, PathNotFoundError> {
        let model_guid;
        if model_guid_in.is_some() {
            model_guid = model_guid_in.unwrap();
        } else {
            // TODO: auto-generate guid
            todo!();
        }
        log::debug!("Storing model {} with guid {}", model_path, model_guid);
        let model = gltf_loader::read_gltf(
            model_path,
            &self.device,
            &self
                .pbr_bind_groups_and_layouts
                .pbr_material_factors_bind_group_layout,
        );
        self.render_storage
            .model_guids
            .insert(model_guid.parse().unwrap(), Box::new(model));
        log::debug!("Model with guid {} stored", model_guid);
        Ok(str::parse(model_guid).unwrap())
    }

    pub fn update_mesh_instance_buffer_and_materials(&mut self) {
        // update internal meshes and materials
        // setup instance buffer for meshes
        for (render_map_key, transforms) in &self.render_storage.render_map {
            // TODO: this is generating instance buffers every frame, do it only whenever transforms changes
            {
                let instance_data = transforms.iter().map(Instance::to_raw).collect::<Vec<_>>();
                let instance_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Instance Buffer"),
                            contents: bytemuck::cast_slice(&instance_data),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                // TODO: use Arc<[T]> for faster clone https://www.youtube.com/watch?v=A4cKi7PTJSs&ab_channel=LoganSmith
                self.render_storage
                    .instance_buffer_map
                    .insert(render_map_key.clone(), instance_buffer);
            }
        }

        // TODO: combine this with loop below to make things more concise
        // update materials
        for (render_map_key, _transforms) in &self.render_storage.render_map {
            let model_map = &mut self.render_storage.model_guids;
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
                material.update_textures(
                    &self.device,
                    &self.queue,
                    &self
                        .pbr_bind_groups_and_layouts
                        .pbr_material_textures_bind_group_layout,
                );
                // log::debug!(
                //     "material loading progress: {:.2}%",
                //     material.get_progress() * 100.0
                // );
            }
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let texture_view = self
            .frame_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.update_mesh_instance_buffer_and_materials();

        // render to gbuffers
        self.deferred_rendering_tech.render_to_gbuffers(
            &mut encoder,
            &self.depth_texture,
            &self.camera_bind_group,
            &self.render_storage,
        );

        // combine gbuffers into one final texture result
        self.deferred_rendering_tech.combine_gbuffers_to_texture(
            &self.device,
            &mut encoder,
            &mut self.frame_texture,
        );

        // forward render translucent objects
        self.forward_rendering_tech.render_translucent_objects(
            &mut encoder,
            &mut self.frame_texture,
            &mut self.depth_texture,
            &self.camera_bind_group,
            &self.render_storage,
        );

        self.queue.submit(iter::once(encoder.finish()));

        let output_texture_view = self
            .frame_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.frame_texture_view = Some(output_texture_view);

        Ok(())
    }

    pub fn clear(&mut self) {
        self.render_storage.render_map.clear();
        self.render_storage.instance_buffer_map.clear();
    }
}
