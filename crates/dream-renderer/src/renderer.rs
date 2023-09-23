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
use crate::instance::{Instance, InstanceRaw};
use crate::model::{DrawModel, Model, ModelVertex, Vertex};
use crate::path_not_found_error::PathNotFoundError;
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
struct RenderMapKey {
    pub model_guid: String,
    pub mesh_index: i32,
}

pub struct RendererWgpu {
    pub surface: Option<wgpu::Surface>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub frame_texture_view: Option<wgpu::TextureView>,
    pub g_buffer_texture_views: [Option<wgpu::TextureView>; 4],
    pub preferred_texture_format: Option<wgpu::TextureFormat>,
    camera: camera::Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    render_pipeline_write_g_buffers: wgpu::RenderPipeline,
    render_pipeline_forward_rendering: wgpu::RenderPipeline,
    depth_texture_g_buffers: texture::Texture,
    depth_texture_forward_rendering: texture::Texture,
    frame_texture: texture::Texture,
    camera_bind_group: wgpu::BindGroup,
    model_guids: std::collections::HashMap<String, Box<Model>>,
    render_map: std::collections::HashMap<RenderMapKey, Vec<Instance>>,
    instance_buffer_map: std::collections::HashMap<RenderMapKey, wgpu::Buffer>,
    pbr_material_factors_bind_group_layout: wgpu::BindGroupLayout,
    pbr_material_textures_bind_group_layout: wgpu::BindGroupLayout,
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

        // let mut web_gl_limits = wgpu::Limits::downlevel_webgl2_defaults();
        // web_gl_limits.max_texture_dimension_2d =
        //     std::cmp::max(4096, web_gl_limits.max_texture_dimension_2d);
        //
        // let (device, queue) = adapter
        //     .request_device(
        //         &wgpu::DeviceDescriptor {
        //             label: None,
        //             features: wgpu::Features::empty(),
        //             limits: web_gl_limits,
        //         },
        //         None,
        //     )
        //     .await
        //     .map(|(device, queue)| -> (wgpu::Device, wgpu::Queue) { (device, queue) })
        //     .unwrap();

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
                    // metallic roughness texture
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

        let shader_write_g_buffers = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Write G Buffers"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_write_g_buffers.wgsl").into()),
        });

        let shader_forward = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Forward"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_forward.wgsl").into()),
        });

        let depth_texture_g_buffers = texture::Texture::create_depth_texture(
            &device,
            config.width,
            config.height,
            "depth_texture_g_buffers",
        );

        let depth_texture_forward_rendering = texture::Texture::create_depth_texture(
            &device,
            config.width,
            config.height,
            "depth_texture_forward_rendering",
        );

        let texture_g_buffer_normal = texture::Texture::create_frame_texture(
            &device,
            config.width,
            config.height,
            "Texture GBuffer Normal",
            wgpu::TextureFormat::Rgba16Float,
        );

        let texture_g_buffer_albedo = texture::Texture::create_frame_texture(
            &device,
            config.width,
            config.height,
            "Texture GBuffer Albedo",
            wgpu::TextureFormat::Bgra8Unorm,
        );

        let texture_g_buffer_emissive = texture::Texture::create_frame_texture(
            &device,
            config.width,
            config.height,
            "Texture GBuffer Emissive",
            wgpu::TextureFormat::Bgra8Unorm,
        );

        let texture_g_buffer_ao_roughness_metallic = texture::Texture::create_frame_texture(
            &device,
            config.width,
            config.height,
            "Texture GBuffer AO Roughness Metallic",
            wgpu::TextureFormat::Bgra8Unorm,
        );

        let g_buffer_texture_views = [
            Some(texture_g_buffer_normal.view),
            Some(texture_g_buffer_albedo.view),
            Some(texture_g_buffer_emissive.view),
            Some(texture_g_buffer_ao_roughness_metallic.view),
        ];

        let frame_texture = texture::Texture::create_frame_texture(
            &device,
            config.width,
            config.height,
            "frame_texture",
            preferred_texture_format.unwrap(),
        );

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

        let render_pipeline_write_g_buffers =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Write G Buffers"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_write_g_buffers,
                    entry_point: "vs_main",
                    buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
                    // buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_write_g_buffers,
                    entry_point: "fs_main",
                    targets: &[
                        // normal
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Rgba16Float,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        // albedo
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        // emissive
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        // ao + roughness + metallic
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8Unorm,
                            blend: None,
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
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

        let render_pipeline_forward_rendering =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline Forward Rendering"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_forward,
                    entry_point: "vs_main",
                    buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
                    // buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_forward,
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
            config,
            render_pipeline_forward_rendering,
            depth_texture_g_buffers,
            depth_texture_forward_rendering,
            frame_texture,
            frame_texture_view: None,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            model_guids: Default::default(),
            render_map: Default::default(),
            instance_buffer_map: Default::default(),
            pbr_material_factors_bind_group_layout,
            pbr_material_textures_bind_group_layout,
            preferred_texture_format,
            render_pipeline_write_g_buffers,
            g_buffer_texture_views,
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
        // update gbuffers
        {
            let texture_g_buffer_normal = texture::Texture::create_frame_texture(
                &self.device,
                self.config.width,
                self.config.height,
                "Texture GBuffer Normal",
                wgpu::TextureFormat::Rgba16Float,
            );

            let texture_g_buffer_albedo = texture::Texture::create_frame_texture(
                &self.device,
                self.config.width,
                self.config.height,
                "Texture GBuffer Albedo",
                wgpu::TextureFormat::Bgra8Unorm,
            );

            let texture_g_buffer_emissive = texture::Texture::create_frame_texture(
                &self.device,
                self.config.width,
                self.config.height,
                "Texture GBuffer Emissive",
                wgpu::TextureFormat::Bgra8Unorm,
            );

            let texture_g_buffer_ao_roughness_metallic = texture::Texture::create_frame_texture(
                &self.device,
                self.config.width,
                self.config.height,
                "Texture GBuffer AO Roughness Metallic",
                wgpu::TextureFormat::Bgra8Unorm,
            );

            let g_buffer_texture_views = [
                Some(texture_g_buffer_normal.view),
                Some(texture_g_buffer_albedo.view),
                Some(texture_g_buffer_emissive.view),
                Some(texture_g_buffer_ao_roughness_metallic.view),
            ];

            self.g_buffer_texture_views = g_buffer_texture_views;
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
        self.depth_texture_g_buffers = texture::Texture::create_depth_texture(
            &self.device,
            self.config.width,
            self.config.height,
            "depth_texture_g_buffers",
        );
        // resize depth texture for forward rendering
        self.depth_texture_forward_rendering = texture::Texture::create_depth_texture(
            &self.device,
            self.config.width,
            self.config.height,
            "depth_texture_forward_rendering",
        );
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

    pub fn is_model_stored(&self, model_guid: &str) -> bool {
        self.model_guids.contains_key(model_guid)
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
            &self.pbr_material_factors_bind_group_layout,
        );
        self.model_guids
            .insert(model_guid.parse().unwrap(), Box::new(model));
        log::debug!("Model with guid {} stored", model_guid);
        Ok(str::parse(model_guid).unwrap())
    }

    pub fn update_mesh_instance_buffer_and_materials(&mut self) {
        // update internal meshes and materials
        {
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
                    // TODO: use Arc<[T]> for faster clone https://www.youtube.com/watch?v=A4cKi7PTJSs&ab_channel=LoganSmith
                    self.instance_buffer_map
                        .insert(render_map_key.clone(), instance_buffer);
                }
            }

            // TODO: combine this with loop below to make things more concise
            // update materials
            for (render_map_key, _transforms) in &self.render_map {
                let model_map = &mut self.model_guids;
                // TODO: use Arc<[T]> for faster clone https://www.youtube.com/watch?v=A4cKi7PTJSs&ab_channel=LoganSmith
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
                if !material.loaded() {
                    material.update_images();
                    material.update_textures(
                        &self.device,
                        &self.queue,
                        &self.pbr_material_textures_bind_group_layout,
                    );
                    // log::debug!(
                    //     "material loading progress: {:.2}%",
                    //     material.get_progress() * 100.0
                    // );
                }
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
        {
            // define render pass to write to GBuffers
            let mut render_pass_write_g_buffers =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass Write G Buffers"),
                    color_attachments: &[
                        // albedo
                        Some(wgpu::RenderPassColorAttachment {
                            view: self.g_buffer_texture_views[0].as_ref().unwrap(),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }),
                        // normal
                        Some(wgpu::RenderPassColorAttachment {
                            view: self.g_buffer_texture_views[1].as_ref().unwrap(),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }),
                        // emissive
                        Some(wgpu::RenderPassColorAttachment {
                            view: self.g_buffer_texture_views[2].as_ref().unwrap(),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }),
                        // ao roughness metallic
                        Some(wgpu::RenderPassColorAttachment {
                            view: self.g_buffer_texture_views[3].as_ref().unwrap(),
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }),
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture_g_buffers.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });
            render_pass_write_g_buffers.set_pipeline(&self.render_pipeline_write_g_buffers);

            // camera bind group
            render_pass_write_g_buffers.set_bind_group(0, &self.camera_bind_group, &[]);

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
                render_pass_write_g_buffers.set_vertex_buffer(1, instance_buffer.slice(..));
                // get the material and set it in the bind group
                let material = model
                    .materials
                    .get(mesh.material)
                    .expect("No material at index");
                let is_opaque = material.factor_alpha >= 1.0;
                if is_opaque && material.pbr_material_textures_bind_group.is_some() {
                    render_pass_write_g_buffers.set_bind_group(
                        1,
                        &material.pbr_material_factors_bind_group,
                        &[],
                    );
                    render_pass_write_g_buffers.set_bind_group(
                        2,
                        material.pbr_material_textures_bind_group.as_ref().unwrap(),
                        &[],
                    );
                    // draw the mesh
                    render_pass_write_g_buffers
                        .draw_mesh_instanced(mesh, 0..transforms.len() as u32);
                }
            }
        }

        // forward render
        {
            // define render pass
            let mut render_pass_forward_rendering =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass Forward Rendering"),
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
                        view: &self.depth_texture_forward_rendering.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });
            render_pass_forward_rendering.set_pipeline(&self.render_pipeline_forward_rendering);

            // camera bind group
            render_pass_forward_rendering.set_bind_group(0, &self.camera_bind_group, &[]);

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
                render_pass_forward_rendering.set_vertex_buffer(1, instance_buffer.slice(..));
                // get the material and set it in the bind group
                let material = model
                    .materials
                    .get(mesh.material)
                    .expect("No material at index");
                // only draw transparent objects
                let is_translucent = material.factor_alpha < 1.0;
                if is_translucent && material.pbr_material_textures_bind_group.is_some() {
                    render_pass_forward_rendering.set_bind_group(
                        1,
                        &material.pbr_material_factors_bind_group,
                        &[],
                    );
                    render_pass_forward_rendering.set_bind_group(
                        2,
                        material.pbr_material_textures_bind_group.as_ref().unwrap(),
                        &[],
                    );
                    // draw the mesh
                    render_pass_forward_rendering
                        .draw_mesh_instanced(mesh, 0..transforms.len() as u32);
                }
            }
        }

        self.queue.submit(iter::once(encoder.finish()));

        let output_texture_view = self
            .frame_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.frame_texture_view = Some(output_texture_view);

        Ok(())
    }

    pub fn clear(&mut self) {
        self.render_map.clear();
        self.instance_buffer_map.clear();
    }
}
