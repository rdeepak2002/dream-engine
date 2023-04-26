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
use std::ops::Deref;

use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use crate::model::{DrawModel, Model, ModelVertex, Vertex};

pub mod camera;
pub mod gltf_loader;
pub mod model;
pub mod texture;

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
pub struct PathNotFoundError;

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl std::fmt::Display for PathNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to find file at path")
    }
}

// const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        let scale_mat: cgmath::Matrix4<f32> = cgmath::Matrix4::from_scale(1.0);
        let rotation_mat_x: cgmath::Matrix4<f32> = cgmath::Matrix4::from_angle_x(cgmath::Rad(0.0));
        let rotation_mat_y: cgmath::Matrix4<f32> = cgmath::Matrix4::from_angle_y(cgmath::Rad(0.0));
        let rotation_mat_z: cgmath::Matrix4<f32> = cgmath::Matrix4::from_angle_z(cgmath::Rad(0.0));
        // let translation_mat: cgmath::Matrix4<f32> =
        //     cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 0.0, 0.0));
        let translation_mat: cgmath::Matrix4<f32> =
            cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, -2.0, -5.0));
        let model_mat =
            scale_mat * rotation_mat_z * rotation_mat_y * rotation_mat_x * translation_mat;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            model: model_mat.into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &camera::Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

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
    // vertex_buffer: wgpu::Buffer,
    // index_buffer: wgpu::Buffer,
    diffuse_bind_group: wgpu::BindGroup,
    pub camera: camera::Camera,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    pub surface_format: wgpu::TextureFormat,
    // pub mesh_list: Vec<model::Mesh>,
    // pub mesh_guids: std::collections::HashMap<String, Rc<Model>>,
    model_guids: std::collections::HashMap<String, Model>,
    render_map: std::collections::HashMap<RenderMapKey, Box<Vec<cgmath::Matrix4<f32>>>>,
}

impl RendererWgpu {
    pub async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
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
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let mut web_gl_limits = wgpu::Limits::downlevel_webgl2_defaults();
        web_gl_limits.max_texture_dimension_2d = 4096;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        web_gl_limits
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .map(|(device, queue)| {
                return (device, queue);
            })
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

        // let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_bytes = include_bytes!("container.jpg");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "container.jpg").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
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
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
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

        // in new() after creating `camera`

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
        let play_icon_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            play_icon_texture_bytes,
            "icons/PlayIcon.png",
        )
        .unwrap();

        let file_icon_texture_bytes = include_bytes!("icons/FileIcon.png");
        let file_icon_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            file_icon_texture_bytes,
            "icons/FileIcon.png",
        )
        .unwrap();

        let directory_icon_texture_bytes = include_bytes!("icons/DirectoryIcon.png");
        let directory_icon_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            directory_icon_texture_bytes,
            "icons/FileIcon.png",
        )
        .unwrap();

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
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[ModelVertex::desc()],
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
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        // let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Vertex Buffer"),
        //     contents: bytemuck::cast_slice(VERTICES),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        // let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Index Buffer"),
        //     contents: bytemuck::cast_slice(INDICES),
        //     usage: wgpu::BufferUsages::INDEX,
        // });

        // let cube_path = "filesystem:http://127.0.0.1:8080/temporary/Box.glb";
        // let mesh_path = "cube.glb";
        // let mesh_list = gltf_loader::read_gltf(mesh_path, &device).await;
        // let model = Model::new(Vec::new(), Vec::new());
        // let mesh_list = gltf_loader::read_gltf("Box.glb", &device).await;
        // let mesh_list = gltf_loader::read_gltf("ice_cube.glb", &device).await;
        // let mesh_list = gltf_loader::read_gltf("cube_sketchfab.glb", &device).await;
        // TODO: do something with this cube mesh

        // let index_buffer = cube_mesh.index_buffer;
        // let vertex_buffer = cube_mesh.vertex_buffer;

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
            // vertex_buffer,
            // index_buffer,
            diffuse_bind_group,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            surface_format,
            // mesh_list,
            model_guids: Default::default(),
            render_map: Default::default(),
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

    // pub fn draw_model(&mut self, model_guid: &str, model_mat: cgmath::Matrix4<f32>) -> bool {
    //     todo!();
    //     // TODO: just call draw mesh on all the mesh's that belong to a model with a specific guid
    //     return true;
    // }

    pub fn draw_mesh(
        &mut self,
        model_guid: &str,
        mesh_index: i32,
        model_mat: cgmath::Matrix4<f32>,
    ) {
        let key = RenderMapKey {
            model_guid: model_guid.parse().unwrap(),
            mesh_index,
        };
        {
            if let std::collections::hash_map::Entry::Vacant(e) = self.render_map.entry(key) {
                // create new array
                e.insert(Box::new(vec![model_mat]));
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
        // let model_guid = "dummy_guid";
        // let mesh_path = "cube.glb";
        let model_guid;
        if model_guid_in.is_some() {
            model_guid = model_guid_in.unwrap();
        } else {
            // TODO: auto-generate guid
            todo!();
            model_guid = "dummy_guid";
        }
        let model = gltf_loader::read_gltf(model_path, &self.device).await;
        self.model_guids.insert(model_guid.parse().unwrap(), model);
        return Ok(model_guid.parse().unwrap());
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

        // draw triangle
        {
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

            // let num_vertices = VERTICES.len() as u32;
            // let num_indices = INDICES.len() as u32;

            render_pass.set_pipeline(&self.render_pipeline);
            // diffuse texture
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            // camera
            {
                // TODO: this should model mat be read from renderer vector or wtv its called
                // let scale_mat: cgmath::Matrix4<f32> = cgmath::Matrix4::from_scale(1.0);
                // let rotation_mat_x: cgmath::Matrix4<f32> =
                //     cgmath::Matrix4::from_angle_x(cgmath::Rad(0.0));
                // let rotation_mat_y: cgmath::Matrix4<f32> =
                //     cgmath::Matrix4::from_angle_y(cgmath::Rad(0.0));
                // let rotation_mat_z: cgmath::Matrix4<f32> =
                //     cgmath::Matrix4::from_angle_z(cgmath::Rad(0.0));
                // let translation_mat: cgmath::Matrix4<f32> =
                //     cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, -2.0, -10.0));
                // let model_mat =
                //     scale_mat * rotation_mat_z * rotation_mat_y * rotation_mat_x * translation_mat;
                // TODO: don't hardcode the guid and model index
                // self.camera_uniform.model = model_mat.into();
            }
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            // vertex drawing
            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // TODO: actually add a model with string key 'dummy_guid'
            let model_guid = "dummy_guid";
            let model = self.model_guids.get(model_guid).expect(&*format!(
                "no model loaded in renderer with guid {}",
                model_guid
            ));
            let mesh_index = 0;
            let transform_matrix_index = 0;
            let model_mats = self
                .render_map
                .get(&RenderMapKey {
                    model_guid: model_guid.parse().unwrap(),
                    mesh_index,
                })
                .expect("No render map key found");
            let model_mat = model_mats.get(transform_matrix_index).expect(&*format!(
                "No transform matrix at index {}",
                transform_matrix_index
            ));
            // key of self.render map contains mesh guid and array index
            // value of self.render map is an array of all transforms it should be drawn at (using instancing if multiple)
            // TODO: is clone performance intense?
            let model_mat = model_mat.clone().into();
            self.camera_uniform.model = model_mat;
            self.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(&[self.camera_uniform]),
            );
            render_pass.draw_mesh(model.meshes.get(mesh_index as usize).expect(&*format!(
                "no mesh at index {} for model with guid {}",
                mesh_index, model_guid
            )));
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
