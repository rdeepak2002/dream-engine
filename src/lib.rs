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

mod texture;
use std::iter;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    window: Window,
    egui_wgpu_renderer: egui_wgpu::Renderer,
    egui_winit_context: egui::Context,
    egui_winit_state: egui_winit::State,
    #[allow(dead_code)]
    demo_app: egui_demo_lib::DemoWindows,
    #[allow(dead_code)]
    diffuse_texture: texture::Texture,
    depth_texture_1: texture::Texture,
    frame_texture: texture::Texture,
    frame_texture_view: Option<wgpu::TextureView>,
}

impl State {
    async fn new(window: Window, event_loop: &EventLoop<()>) -> Self {
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

        let (device, queue, egui_wgpu_renderer) = adapter
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
                let egui_wgpu_renderer = egui_wgpu::Renderer::new(&device, surface_format, None, 1);
                return (device, queue, egui_wgpu_renderer);
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

        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let depth_texture_1 =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let frame_texture =
            texture::Texture::create_frame_texture(&device, &config, "frame_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
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
            // depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let mut egui_winit_state = egui_winit::State::new(&event_loop);
        egui_winit_state.set_pixels_per_point(window.scale_factor() as f32);
        let egui_winit_context = egui::Context::default();
        let demo_app = egui_demo_lib::DemoWindows::default();

        Self {
            surface,
            device,
            queue,
            size,
            config,
            render_pipeline,
            window,
            egui_wgpu_renderer,
            egui_winit_state,
            egui_winit_context,
            demo_app,
            diffuse_texture,
            depth_texture_1,
            frame_texture,
            frame_texture_view: None,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture_1 =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.frame_texture =
                texture::Texture::create_frame_texture(&self.device, &self.config, "frame_texture");
        }
    }

    fn update(&mut self) {}

    fn render1(&mut self) -> Result<(), wgpu::SurfaceError> {
        let texture_view = self
            .frame_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder 1"),
            });

        // draw triangle
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass 1"),
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
                    view: &self.depth_texture_1.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));

        let output_texture_view = self
            .frame_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.frame_texture_view = Some(output_texture_view);

        Ok(())
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            // define rendering of egui elements

            // Begin to draw the UI frame.
            let input = self.egui_winit_state.take_egui_input(&self.window);
            self.egui_winit_context.begin_frame(input);

            // Draw the demo application.
            // self.demo_app.ui(&self.egui_winit_context);

            egui::TopBottomPanel::top("menu_bar").show(&self.egui_winit_context, |ui| {
                egui::menu::bar(ui, |ui| {
                    let save_shortcut =
                        egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);

                    if ui.input_mut(|i| i.consume_shortcut(&save_shortcut)) {
                        // TODO: allow saving
                        println!("TODO: save");
                    }

                    ui.menu_button("File", |ui| {
                        ui.set_min_width(100.0);
                        ui.style_mut().wrap = Some(false);

                        if ui
                            .add(
                                egui::Button::new("Save")
                                    .shortcut_text(ui.ctx().format_shortcut(&save_shortcut)),
                            )
                            .clicked()
                        {
                            // TODO: allow saving
                            println!("TODO: save");
                            ui.close_menu();
                        }
                    });
                });
            });

            egui::SidePanel::right("inspector_panel")
                .resizable(true)
                .default_width(200.0)
                .max_width(400.0)
                .min_width(200.0)
                .show(&self.egui_winit_context, |ui| {
                    egui::trace!(ui);
                    ui.vertical_centered(|ui| {
                        ui.label("TODO: inspector");
                    });
                });

            egui::TopBottomPanel::bottom("assets")
                .resizable(false)
                .default_height(200.0)
                .max_height(200.0)
                .min_height(200.0)
                .show(&self.egui_winit_context, |ui| {
                    egui::trace!(ui);
                    ui.vertical_centered(|ui| {
                        ui.label("TODO: assets");
                    });
                });

            egui::SidePanel::left("scene_hierarchy")
                .resizable(true)
                .default_width(200.0)
                .max_width(400.0)
                .min_width(200.0)
                .show(&self.egui_winit_context, |ui| {
                    egui::trace!(ui);
                    ui.vertical_centered(|ui| {
                        ui.label("TODO: scene hierarchy");
                    });
                });

            egui::TopBottomPanel::top("render-controls")
                .resizable(false)
                .default_height(25.0)
                .max_height(25.0)
                .min_height(25.0)
                .show(&self.egui_winit_context, |ui| {
                    egui::trace!(ui);
                    ui.vertical_centered(|ui| {
                        ui.label("TODO: renderer controls");
                    });
                });

            egui::CentralPanel::default().show(&self.egui_winit_context, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label("TODO: renderer");

                    {
                        if self.frame_texture_view.is_some() {
                            let epaint_texture_id =
                                self.egui_wgpu_renderer.register_native_texture(
                                    &self.device,
                                    &self.frame_texture_view.as_ref().unwrap(),
                                    wgpu::FilterMode::default(),
                                );

                            ui.image(epaint_texture_id, egui::Vec2::new(500.0, 500.0));
                        }
                    }
                });
            });
        }

        let egui_full_output = self.egui_winit_context.end_frame();
        let egui_paint_jobs = self.egui_winit_context.tessellate(egui_full_output.shapes);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder 2"),
            });

        {
            for (id, image_delta) in &egui_full_output.textures_delta.set {
                self.egui_wgpu_renderer
                    .update_texture(&self.device, &self.queue, *id, image_delta)
            }

            let egui_screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [self.config.width, self.config.height],
                pixels_per_point: self.window.scale_factor() as f32,
            };

            self.egui_wgpu_renderer.update_buffers(
                &self.device,
                &self.queue,
                &mut encoder,
                &egui_paint_jobs,
                &egui_screen_descriptor,
            );

            // draw editor
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass 2"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
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
                    // depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    //     view: &self.depth_texture_1.view,
                    //     depth_ops: Some(wgpu::Operations {
                    //         load: wgpu::LoadOp::Clear(1.0),
                    //         store: true,
                    //     }),
                    //     stencil_ops: None,
                    // }),
                    depth_stencil_attachment: None,
                });

                self.egui_wgpu_renderer.render(
                    &mut render_pass,
                    &egui_paint_jobs,
                    &egui_screen_descriptor,
                );
            }

            self.queue.submit(iter::once(encoder.finish()));
            output.present();
        }

        for id in &egui_full_output.textures_delta.free {
            self.egui_wgpu_renderer.free_texture(id);
        }

        Ok(())
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        static mut WINDOW_WIDTH: u32 = 2000;
        static mut WINDOW_HEIGHT: u32 = 1500;
        static mut NEED_TO_RESIZE_WINDOW: bool = false;
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
        pub fn resize_window(width: u32, height: u32) {
            unsafe {
                WINDOW_WIDTH = width;
                WINDOW_HEIGHT = height;
                NEED_TO_RESIZE_WINDOW = true;
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    {
        // example execution of javascript code
        // let js_code = "console.log('hello world');";
        let js_code = "7 * 8.1";
        let mut context = boa_engine::Context::default();
        match context.eval(js_code) {
            Ok(res) => {
                println!("{}", res.to_string(&mut context).unwrap());
                log::warn!("{}", res.to_string(&mut context).unwrap());
            }
            Err(e) => {
                // Pretty print the error
                eprintln!("Uncaught {}", e.display());
                log::error!("Uncaught {}", e.display());
            }
        };
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(3000, 1750))
        .build(&event_loop)
        .unwrap();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
        } else {
            window.set_title("Dream Engine");
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        window.set_inner_size(winit::dpi::PhysicalSize::new(2000, 1500));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(window, &event_loop).await;
    event_loop.run(move |event, _, control_flow| {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                if NEED_TO_RESIZE_WINDOW {
                    state
                        .window()
                        .set_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
                    state.resize(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
                    NEED_TO_RESIZE_WINDOW = false;
                }
            }
        }
        match event {
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();

                match state.render1() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }

                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // We're ignoring timeouts
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::WindowEvent { event, .. } => {
                let exclusive = state
                    .egui_winit_state
                    .on_event(&state.egui_winit_context, &event);
                if !exclusive.consumed {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            state.resize(physical_size);
                        }
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &mut so w have to dereference it twice
                            state.resize(*new_inner_size);
                        }
                        _ => (),
                    }
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
