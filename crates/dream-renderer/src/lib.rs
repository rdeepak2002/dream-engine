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

use wgpu::util::DeviceExt;

use crate::camera_uniform::CameraUniform;
use crate::image::Image;
use crate::instance::{Instance, InstanceRaw};
use crate::model::{DrawModel, Model, ModelVertex, Vertex};
use crate::path_not_found_error::PathNotFoundError;

pub mod camera;
pub mod camera_uniform;
pub mod gltf_loader;
pub mod image;
pub mod instance;
pub mod material;
pub mod model;
pub mod path_not_found_error;
pub mod render_map_key;
pub mod texture;

#[derive(Hash, PartialEq, Eq, Clone)]
struct RenderMapKey {
    pub model_guid: String,
    pub mesh_index: i32,
}

pub struct RendererWgpu {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,
    frame_texture: texture::Texture,
    pub frame_texture_view: Option<wgpu::TextureView>,
    // TODO: move these icons to editor
    pub play_icon_texture: texture::Texture,
    pub file_icon_texture: texture::Texture,
    pub directory_icon_texture: texture::Texture,
    pub camera: camera::Camera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    pub surface_format: wgpu::TextureFormat,
    model_guids: std::collections::HashMap<String, Box<Model>>,
    render_map: std::collections::HashMap<RenderMapKey, Vec<Instance>>,
    instance_buffer_map: std::collections::HashMap<RenderMapKey, wgpu::Buffer>,
    pbr_material_factors_bind_group_layout: wgpu::BindGroupLayout,
    pbr_material_textures_bind_group_layout: wgpu::BindGroupLayout,
}

impl RendererWgpu {
    pub async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // let mut web_gl_limits = wgpu::Limits::downlevel_webgl2_defaults();
        // web_gl_limits.max_texture_dimension_2d = 4096;

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

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let pbr_material_factors_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("texture_bind_group_layout"),
            });

        let pbr_material_textures_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // base color texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // metallic texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // normal map texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // emissive texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // occlusion texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 9,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("pbr_textures_bind_group_layout"),
            });

        let camera = camera::Camera {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.01,
            zfar: 1000.0,
        };

        let mut camera_uniform = CameraUniform::new();
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

        let play_icon_texture_bytes = include_bytes!("icons/PlayIcon.png");
        let mut play_icon_image = Image::default();
        play_icon_image
            .load_from_bytes(play_icon_texture_bytes, "icons/PlayIcon.png", None)
            .await;
        let rgba = play_icon_image.to_rgba8();
        let play_icon_texture =
            texture::Texture::new(&device, &queue, rgba.to_vec(), rgba.dimensions(), None)
                .expect("Unable to load play icon texture");

        let file_icon_texture_bytes = include_bytes!("icons/FileIcon.png");
        let mut file_icon_image = Image::default();
        file_icon_image
            .load_from_bytes(file_icon_texture_bytes, "icons/FileIcon.png", None)
            .await;
        let rgba = file_icon_image.to_rgba8();
        let file_icon_texture =
            texture::Texture::new(&device, &queue, rgba.to_vec(), rgba.dimensions(), None)
                .expect("Unable to load file icon texture");

        let directory_icon_texture_bytes = include_bytes!("icons/DirectoryIcon.png");
        let mut directory_icon_image = Image::default();
        directory_icon_image
            .load_from_bytes(
                directory_icon_texture_bytes,
                "icons/DirectoryIcon.png",
                None,
            )
            .await;
        let rgba = directory_icon_image.to_rgba8();
        let directory_icon_texture =
            texture::Texture::new(&device, &queue, rgba.to_vec(), rgba.dimensions(), None)
                .expect("Unable to load directory icon texture");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let frame_texture =
            texture::Texture::create_frame_texture(&device, &config, "frame_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &pbr_material_factors_bind_group_layout,
                    &pbr_material_textures_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
                // buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            size,
            config,
            render_pipeline,
            depth_texture,
            frame_texture,
            frame_texture_view: None,
            play_icon_texture,
            file_icon_texture,
            directory_icon_texture,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            surface_format,
            model_guids: Default::default(),
            render_map: Default::default(),
            instance_buffer_map: Default::default(),
            pbr_material_factors_bind_group_layout,
            pbr_material_textures_bind_group_layout,
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

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            // resize frame textures
            self.frame_texture =
                texture::Texture::create_frame_texture(&self.device, &self.config, "frame_texture");
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn draw_mesh(&mut self, model_guid: &str, mesh_index: i32, model_mat: Instance) {
        let key = RenderMapKey {
            model_guid: model_guid.parse().unwrap(),
            mesh_index,
        };
        {
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
    }

    pub async fn store_model(
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
            &self.queue,
            &self.pbr_material_factors_bind_group_layout,
            &self.pbr_material_textures_bind_group_layout,
        )
        .await;
        self.model_guids
            .insert(model_guid.parse().unwrap(), Box::new(model));
        log::debug!("model with guid {} stored", model_guid);
        Ok(str::parse(model_guid).unwrap())
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

        {
            // define render pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);

            // camera bind group
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            // setup instance buffer for meshes
            for (render_map_key, transforms) in &self.render_map {
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
                    self.instance_buffer_map
                        .insert(render_map_key.clone(), instance_buffer);
                }
            }

            // TODO: combine this with loop below to make things more concise
            // update materials
            for (render_map_key, _transforms) in &self.render_map {
                let model_map = &mut self.model_guids;
                let model_guid = render_map_key.model_guid.clone();
                let model = model_map.get_mut(&*model_guid).unwrap_or_else(|| {
                    panic!("no model loaded in renderer with guid {}", model_guid)
                });
                let mesh_index = render_map_key.mesh_index;
                let mesh = model
                    .meshes
                    .get_mut(mesh_index as usize)
                    .unwrap_or_else(|| {
                        panic!(
                            "no mesh at index {} for model with guid {}",
                            mesh_index, model_guid
                        )
                    });
                let material = model
                    .materials
                    .get_mut(mesh.material)
                    .expect("No material at index");
                material.update();
            }

            // iterate through all meshes that should be instanced drawn
            for (render_map_key, transforms) in &self.render_map {
                let model_map = &self.model_guids;
                // get the mesh to be instance drawn
                let model_guid = render_map_key.model_guid.clone();
                if model_map.get(&*model_guid).is_none() {
                    log::warn!("skipping drawing of model {}", model_guid);
                    continue;
                }
                let model = model_map.get(&*model_guid).unwrap_or_else(|| {
                    panic!("no model loaded in renderer with guid {}", model_guid)
                });
                let mesh_index = render_map_key.mesh_index;
                let mesh = model.meshes.get(mesh_index as usize).unwrap_or_else(|| {
                    panic!(
                        "no mesh at index {} for model with guid {}",
                        mesh_index, model_guid
                    )
                });
                // setup instancing buffer
                let instance_buffer = self
                    .instance_buffer_map
                    .get(render_map_key)
                    .expect("No instance buffer found in map");
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                // get the material and set it in the bind group
                let material = model
                    .materials
                    .get(mesh.material)
                    .expect("No material at index");
                render_pass.set_bind_group(1, &material.pbr_material_factors_bind_group, &[]);
                render_pass.set_bind_group(2, &material.pbr_material_textures_bind_group, &[]);
                // draw the mesh
                render_pass.draw_mesh_instanced(mesh, 0..transforms.len() as u32);
            }
        }

        self.queue.submit(iter::once(encoder.finish()));

        let output_texture_view = self
            .frame_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.frame_texture_view = Some(output_texture_view);

        self.render_map.clear();

        Ok(())
    }
}
