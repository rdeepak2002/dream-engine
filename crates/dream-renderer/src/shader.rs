pub struct Shader {
    source: String,
    shader_module: wgpu::ShaderModule,
}

impl Shader {
    pub fn new(device: &wgpu::Device, source: String, label: String) -> Shader {
        let mut source = source;

        if source.contains("//include:pbr.wgsl") {
            source = source.replace("//include:pbr.wgsl", include_str!("pbr.wgsl"));
        }

        if source.contains("//include:camera.wgsl") {
            source = source.replace("//include:camera.wgsl", include_str!("camera.wgsl"));
        }

        if source.contains("//include:model.wgsl") {
            source = source.replace("//include:model.wgsl", include_str!("model.wgsl"));
        }

        if source.contains("//include:skinning.wgsl") {
            source = source.replace("//include:skinning.wgsl", include_str!("skinning.wgsl"));
        }

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label.as_str()),
            source: wgpu::ShaderSource::Wgsl(source.clone().into()),
        });
        Shader {
            source,
            shader_module,
        }
    }

    pub fn get_shader_module(&self) -> &wgpu::ShaderModule {
        &self.shader_module
    }
}
