use crate::lights::Lights;

pub struct ShadowTech {}

impl ShadowTech {
    pub fn new() -> Self {
        Self {
            // TODO
        }
    }

    pub fn render_shadow_depth_buffers(&mut self, lights: &Lights) {
        for light in &lights.renderer_lights {
            if light.cast_shadow {
                // log::debug!(
                //     "TODO: compute depth buffer stuff for this light for shadows {:?}",
                //     light
                // );
            }
        }
    }
}
