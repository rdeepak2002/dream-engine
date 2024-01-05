use std::sync::{Arc, Mutex};

use crossbeam_channel::unbounded;
use egui::RawInput;

use crate::assets_panel::AssetsPanel;
use crate::inspector_panel::InspectorPanel;
use crate::menu_bar_panel::MenuBarPanel;
use crate::renderer_controls_panel::RendererControlsPanel;
use crate::renderer_panel::RendererPanel;
use crate::scene_hierarchy_panel::SceneHierarchyPanel;

pub enum EditorEventType {
    ShowEntityInInspector,
    // other kinds of events: open [file], etc.
}

pub struct EditorEvent {
    pub event_type: EditorEventType,
    pub event_data: String,
}

pub(crate) trait Panel {
    fn draw(&mut self, egui_context: &egui::Context);
}

pub struct EditorEguiWgpu {
    pub depth_texture_egui: dream_renderer::texture::Texture,
    pub egui_winit_state: egui_winit::State,
    renderer_panel: Arc<Mutex<RendererPanel>>,
    panels: Vec<Arc<Mutex<dyn Panel>>>,
    egui_wgpu_renderer: egui_wgpu::Renderer,
    egui_context: egui::Context,
}

pub fn generate_egui_wgpu_renderer(
    state: &dream_renderer::renderer::RendererWgpu,
) -> egui_wgpu::Renderer {
    egui_wgpu::Renderer::new(
        &state.device,
        // state.preferred_texture_format.unwrap(),
        state.surface_texture_format.unwrap(),
        Some(dream_renderer::texture::Texture::DEPTH_FORMAT),
        1,
    )
}

pub fn generate_egui_wgpu_depth_texture(
    state: &dream_renderer::renderer::RendererWgpu,
) -> dream_renderer::texture::Texture {
    dream_renderer::texture::Texture::create_depth_texture(
        &state.device,
        state.config.width,
        state.config.height,
        "depth_texture_egui",
    )
}

impl EditorEguiWgpu {
    pub async fn new(
        app: &dream_app::app::App,
        renderer: &dream_renderer::renderer::RendererWgpu,
        scale_factor: f32,
        event_loop: &winit::event_loop::EventLoop<()>,
    ) -> Self {
        let depth_texture_egui = generate_egui_wgpu_depth_texture(renderer);
        let mut egui_wgpu_renderer = generate_egui_wgpu_renderer(renderer);
        let mut egui_winit_state = egui_winit::State::new(&event_loop);
        egui_winit_state.set_pixels_per_point(scale_factor);
        let egui_winit_context = egui::Context::default();

        let (sx, rx) = unbounded::<EditorEvent>();

        let inspector_panel = Arc::new(Mutex::new(InspectorPanel::new(
            rx,
            Arc::downgrade(&app.scene),
        )));
        let assets_panel = Arc::new(Mutex::new(AssetsPanel::new(
            renderer,
            &mut egui_wgpu_renderer,
        )));
        let renderer_controls_panel = Arc::new(Mutex::new(RendererControlsPanel::new(
            renderer,
            &mut egui_wgpu_renderer,
        )));
        let scene_hierarchy_panel = Arc::new(Mutex::new(SceneHierarchyPanel::new(
            sx,
            Arc::downgrade(&app.scene),
        )));
        let renderer_panel = Arc::new(Mutex::new(RendererPanel::default()));

        Self {
            egui_wgpu_renderer,
            egui_context: egui_winit_context,
            egui_winit_state,
            depth_texture_egui,
            renderer_panel,
            panels: vec![
                Arc::new(Mutex::new(MenuBarPanel::default())),
                inspector_panel,
                assets_panel,
                scene_hierarchy_panel,
                renderer_controls_panel,
            ],
        }
    }

    pub fn render_egui_editor_content(&mut self) {
        for i in 0..self.panels.len() {
            self.panels[i].lock().unwrap().draw(&self.egui_context);
        }
        self.renderer_panel.lock().unwrap().draw(&self.egui_context);
    }

    pub fn handle_event(&mut self, window_event: &winit::event::WindowEvent) -> bool {
        self.egui_winit_state
            .on_event(&self.egui_context, window_event)
            .consumed
    }

    pub fn render_wgpu(
        &mut self,
        state: &dream_renderer::renderer::RendererWgpu,
        input: RawInput,
        pixels_per_point: f32,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = state
            .surface
            .as_ref()
            .expect("No surface available for editor to draw to")
            .get_current_texture()?;
        // TODO: should we really be creating a view every time?
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.egui_context.begin_frame(input);

        self.renderer_panel
            .lock()
            .unwrap()
            .update_texture(state, &mut self.egui_wgpu_renderer);
        self.render_egui_editor_content();

        let egui_full_output = self.egui_context.end_frame();
        let egui_paint_jobs = self.egui_context.tessellate(egui_full_output.shapes);
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("EGUI Render Encoder"),
            });

        {
            for (id, image_delta) in &egui_full_output.textures_delta.set {
                self.egui_wgpu_renderer.update_texture(
                    &state.device,
                    &state.queue,
                    *id,
                    image_delta,
                )
            }

            let egui_screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                size_in_pixels: [state.config.width, state.config.height],
                pixels_per_point,
            };

            self.egui_wgpu_renderer.update_buffers(
                &state.device,
                &state.queue,
                &mut encoder,
                &egui_paint_jobs,
                &egui_screen_descriptor,
            );

            // draw editor
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("EGUI Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.10588235294,
                                g: 0.10588235294,
                                b: 0.10588235294,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture_egui.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });

                self.egui_wgpu_renderer.render(
                    &mut render_pass,
                    &egui_paint_jobs,
                    &egui_screen_descriptor,
                );
            }

            state.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        for id in &egui_full_output.textures_delta.free {
            self.egui_wgpu_renderer.free_texture(id);
        }

        Ok(())
    }

    pub fn handle_resize(&mut self, state: &dream_renderer::renderer::RendererWgpu) {
        self.depth_texture_egui = generate_egui_wgpu_depth_texture(state);
    }

    pub fn get_renderer_aspect_ratio(&self) -> f32 {
        self.renderer_panel.lock().unwrap().get_aspect_ratio()
    }
}
