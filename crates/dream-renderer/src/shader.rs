pub struct Shader {
    source: String,
    shader_module: wgpu::ShaderModule,
}

impl Shader {
    pub fn new(device: &wgpu::Device, source: String, label: String) -> Shader {
        let mut source = source;

        if source.contains("//include:pbr.wgsl") {
            source = source.replace("//include:pbr.wgsl", include_str!("shader/pbr.wgsl"));
        }

        if source.contains("//include:camera.wgsl") {
            source = source.replace("//include:camera.wgsl", include_str!("shader/camera.wgsl"));
        }

        if source.contains("//include:model.wgsl") {
            source = source.replace("//include:model.wgsl", include_str!("shader/model.wgsl"));
        }

        if source.contains("//include:skinning.wgsl") {
            source = source.replace(
                "//include:skinning.wgsl",
                include_str!("shader/skinning.wgsl"),
            );
        }

        if source.contains("//include:shadow.wgsl") {
            source = source.replace("//include:shadow.wgsl", include_str!("shader/shadow.wgsl"));
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
