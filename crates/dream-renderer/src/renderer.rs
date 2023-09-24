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
use crate::path_not_found_error::PathNotFoundError;
use crate::pbr_bind_groups_and_layouts::PbrBindGroupsAndLayouts;
use crate::render_storage::RenderStorage;
use crate::{camera, texture};

#[cfg(not(feature = "wgpu/webgl"))]
pub fn is_webgpu_enabled() -> bool {
    true
}

#[cfg(feature = "wgpu/webgl")]
pub fn is_webgpu_enabled() -> bool {
    false
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

        let frame_texture = texture::Texture::create_frame_texture(
            &device,
            config.width,
            config.height,
            "frame_texture",
            preferred_texture_format.unwrap(),
        );

        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            config.width,
            config.height,
            "depth_texture",
        );

        let pbr_bind_groups_and_layouts =
            PbrBindGroupsAndLayouts::new(&device, &camera_bind_group_layout);
        let deferred_rendering_tech = DeferredRenderingTech::new(
            &device,
            &pbr_bind_groups_and_layouts.render_pipeline_pbr_layout,
            config.format,
            config.width,
            config.height,
        );
        let forward_rendering_tech = ForwardRenderingTech::new(
            &device,
            &pbr_bind_groups_and_layouts.render_pipeline_pbr_layout,
            config.format,
        );

        let render_storage = RenderStorage {
            model_guids: Default::default(),
            render_map: Default::default(),
            instance_buffer_map: Default::default(),
        };

        Self {
            frame_texture_view: None,
            surface,
            device,
            queue,
            config,
            depth_texture,
            frame_texture,
            render_storage,
            // TODO: combine below four variables
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            preferred_texture_format,
            pbr_bind_groups_and_layouts,
            deferred_rendering_tech,
            forward_rendering_tech,
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
        // update config width and height
        if let Some(new_size) = new_size {
            if new_size.width > 0 && new_size.height > 0 {
                self.config.width = new_size.width;
                self.config.height = new_size.height;
            }
        }
        // update surface using new config
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
        // resize gbuffers for deferred rendering
        self.deferred_rendering_tech
            .resize(&self.device, self.config.width, self.config.height);
    }

    pub fn draw_mesh(&mut self, model_guid: &str, mesh_index: i32, model_mat: Instance) {
        self.render_storage
            .queue_for_drawing(model_guid, mesh_index, model_mat);
    }

    pub fn is_model_stored(&self, model_guid: &str) -> bool {
        self.render_storage.is_model_stored(model_guid)
    }

    pub fn store_model(
        &mut self,
        model_guid_in: Option<&str>,
        model_path: &str,
    ) -> Result<String, PathNotFoundError> {
        self.render_storage.store_model(
            model_guid_in,
            model_path,
            &self.device,
            &self
                .pbr_bind_groups_and_layouts
                .pbr_material_factors_bind_group_layout,
        )
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // create instance buffers for mesh positions and update loading of textures for materials
        self.render_storage
            .update_mesh_instance_buffer_and_materials(
                &self.device,
                &self.queue,
                &self
                    .pbr_bind_groups_and_layouts
                    .pbr_material_textures_bind_group_layout,
            );

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

        // update the output texture view, so editor can display it in a panel
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
