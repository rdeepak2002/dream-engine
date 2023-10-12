/**********************************************************************************
 *  Dream is a software for developing real-time 3D experiences.
 *  Copyright (C) 2023 Deepak Ramalingam
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

use wgpu::{CompositeAlphaMode, PresentMode};
use winit::dpi::PhysicalSize;

use dream_math::{Point3, Quaternion, Vector3};

use crate::camera_bones_light_bind_group::CameraBonesLightBindGroup;
use crate::deferred_rendering_tech::DeferredRenderingTech;
use crate::forward_rendering_tech::ForwardRenderingTech;
use crate::instance::Instance;
use crate::lights::{Lights, RendererLight};
use crate::path_not_found_error::PathNotFoundError;
use crate::pbr_material_tech::PbrMaterialTech;
use crate::render_storage::RenderStorage;
use crate::shadow_tech::ShadowTech;
use crate::skinning::SkinningTech;
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
    pub preferred_texture_format: Option<wgpu::TextureFormat>,
    pub frame_texture: texture::Texture,
    pub deferred_rendering_tech: DeferredRenderingTech,
    render_storage: RenderStorage,
    camera: camera::Camera,
    depth_texture: texture::Texture,
    forward_rendering_tech: ForwardRenderingTech,
    pbr_material_tech: PbrMaterialTech,
    skinning_tech: SkinningTech,
    lights: Lights,
    shadow_tech: ShadowTech,
    camera_bones_light_bind_group: CameraBonesLightBindGroup,
}

impl RendererWgpu {
    pub async fn new(window: Option<&winit::window::Window>) -> Self {
        // instance is a handle to our GPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // size is the dimensions of the window
        let size;
        // surface is the surface of our window
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

        // adapter is the physical gpu driver
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

        // device is an open connection to a gpu device
        // queue is for writing to buffers and textures by executing command buffers
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

        // preferred texture format is the format of the surface we draw to
        let preferred_texture_format;
        // surface configuration describes a surface like its dimensions
        let config;
        if surface.is_some() {
            let surface_caps = surface.as_ref().unwrap().get_capabilities(&adapter);

            // Shader code in this tutorial assumes an Srgb surface texture. Using a different
            // one will result all the colors coming out darker. If you want to support non
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

        // main camera
        let camera = camera::Camera::new_perspective(
            Point3::new(3.0, 3.0, 3.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            config.width as f32 / config.height as f32,
            std::f32::consts::FRAC_PI_4,
            0.01,
            1000.0,
            &device,
        );

        // texture we draw our final result to
        let frame_texture = texture::Texture::create_frame_texture(
            &device,
            config.width,
            config.height,
            "frame_texture",
            preferred_texture_format.unwrap(),
        );

        // texture to keep track of depth buffer
        let depth_texture = texture::Texture::create_depth_texture(
            &device,
            config.width,
            config.height,
            "depth_texture",
        );

        // lights storage
        let lights = Lights::new(&device);

        // skinning tech
        let skinning_tech = SkinningTech::new(&device);

        // bind group for camera, bones, and lights
        let camera_bones_light_bind_group =
            CameraBonesLightBindGroup::new(&device, &camera, &lights, &skinning_tech);

        // bind groups and layouts for physically based rendering textures
        let pbr_material_tech = PbrMaterialTech::new(&device);

        // shadow tech
        let shadow_tech = ShadowTech::new(&device, &camera_bones_light_bind_group, &camera);

        // algorithms for deferred rendering
        let deferred_rendering_tech = DeferredRenderingTech::new(
            &device,
            config.format,
            config.width,
            config.height,
            &depth_texture,
            &pbr_material_tech,
            &shadow_tech,
            &camera_bones_light_bind_group,
        );

        // algorithms for forward rendering
        let forward_rendering_tech = ForwardRenderingTech::new(
            &device,
            config.format,
            &pbr_material_tech,
            &camera_bones_light_bind_group,
        );

        // storage for all 3D mesh data and positions
        let render_storage = RenderStorage {
            model_guids: Default::default(),
            render_map: Default::default(),
            instance_buffer_map: Default::default(),
        };

        Self {
            surface,
            device,
            queue,
            config,
            preferred_texture_format,
            render_storage,
            camera,
            frame_texture,
            depth_texture,
            deferred_rendering_tech,
            forward_rendering_tech,
            pbr_material_tech,
            lights,
            skinning_tech,
            shadow_tech,
            camera_bones_light_bind_group,
        }
    }

    /// User-facing API to set the aspect ratio of the main camera. This is primarily used by the
    /// editor to change the aspect ratio of the camera when the renderer panel is resized.
    pub fn set_camera_aspect_ratio(&mut self, new_aspect_ratio: f32) {
        self.camera.set_aspect_ratio(&self.queue, new_aspect_ratio);
    }

    /// User-facing API to resize all textures and surface configuration
    pub fn resize(&mut self, new_size: Option<PhysicalSize<u32>>) {
        // update config width and height
        if let Some(new_size) = new_size {
            if new_size.width > 0 && new_size.height > 0 {
                self.config.width = new_size.width;
                self.config.height = new_size.height;
            }
        }
        log::warn!(
            "New config size {} {}",
            self.config.width,
            self.config.height
        );
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

    /// User-facing API to invoke render loop once
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
                    .pbr_material_tech
                    .pbr_material_textures_bind_group_layout,
            );

        // update light buffers
        self.lights.update_light_buffer(&self.device, &self.queue);

        // update bones buffer
        self.skinning_tech.update_all_bones_buffer(&self.queue);

        // figure out shadows
        self.shadow_tech.render_shadow_depth_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.lights,
            &self.render_storage,
            &self.camera_bones_light_bind_group,
        );

        // render to gbuffers
        self.deferred_rendering_tech.render_to_gbuffers(
            &mut encoder,
            &self.depth_texture,
            &self.render_storage,
            &self.camera_bones_light_bind_group,
        );

        // combine gbuffers into one final texture result
        self.deferred_rendering_tech.combine_gbuffers_to_texture(
            &self.device,
            &mut encoder,
            &mut self.frame_texture,
            &mut self.depth_texture,
            // &self.camera, // TODO: revert back to &self.camera ; &self.shadow_tech.shadow_cameras[0]
            // &self.shadow_tech.shadow_cameras[0],
            // &self.lights,
            &self.shadow_tech,
            &self.camera_bones_light_bind_group,
        );

        // forward render translucent objects
        self.forward_rendering_tech.render_translucent_objects(
            &mut encoder,
            &mut self.frame_texture,
            &mut self.depth_texture,
            // &self.camera, // TODO: revert back to &self.camera ; &self.shadow_tech.shadow_cameras[0]
            // &self.shadow_tech.shadow_cameras[0],
            // &self.lights,
            &self.render_storage,
            // &self.skinning_tech,
            &self.camera_bones_light_bind_group,
        );

        // submit all drawing commands to gpu
        self.queue.submit(iter::once(encoder.finish()));

        // update the output texture view, so editor can display it in a panel
        // let output_texture_view = &self
        //     .frame_texture.view;
        // self.frame_texture_view = Some(output_texture_view);

        Ok(())
    }

    /// User-facing API to specify what should be drawn and where
    ///
    /// # Arguments
    ///
    /// * `model_guid`
    /// * `mesh_index`
    /// * `model_mat`
    pub fn draw_mesh(&mut self, model_guid: &str, mesh_index: i32, model_mat: Instance) {
        self.render_storage
            .queue_for_drawing(model_guid, mesh_index, model_mat);
    }

    /// User-facing API to draw a light at a specific position and color
    ///
    /// # Arguments
    ///
    /// * `position`
    /// * `color`
    pub fn draw_light(
        &mut self,
        light_type: u32,
        position: Vector3<f32>,
        color: Vector3<f32>,
        radius: f32,
        direction: Vector3<f32>,
        cast_shadow: bool,
    ) {
        self.lights.renderer_lights.push(RendererLight {
            position,
            color,
            radius,
            light_type,
            direction,
            cast_shadow,
        });
    }

    /// User-facing API to store a model and associate it with a guid
    ///
    /// # Arguments
    ///
    /// * `model_guid`
    /// * `path`
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
                .pbr_material_tech
                .pbr_material_textures_bind_group_layout,
        )
    }

    /// User-facing API to verify if a model is stored
    ///
    /// # Arguments
    ///
    /// * `model_guid`
    pub fn is_model_stored(&self, model_guid: &str) -> bool {
        self.render_storage.is_model_stored(model_guid)
    }

    /// User-facing API to remove all models, meshes, and instance buffers
    pub fn clear(&mut self) {
        self.render_storage.render_map.clear();
        self.lights.renderer_lights.clear();
    }

    pub fn set_bone_transform(&mut self, bone_id: u32, mat: dream_math::Matrix4<f32>) {
        // phase 2 (this will allow u to model the same instance of a model in different ways)
        // TODO: the entity ID of the root bone tells us which armature we are using
        // TODO: associate bones for that armature
        // todo!();
        self.skinning_tech.update_bone(bone_id, mat);
    }

    pub fn set_camera(&mut self, position: Point3<f32>, orientation: Quaternion<f32>) {
        self.camera
            .set_position_and_orientation(&self.queue, position, orientation);
    }
}
