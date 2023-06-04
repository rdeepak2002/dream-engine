use image::{DynamicImage, ImageFormat, RgbaImage};

use dream_tasks::task_pool::get_task_pool;

#[derive(Default)]
pub struct Image {
    dynamic_image: Option<DynamicImage>,
    rgba: Option<RgbaImage>,
}

fn dynamic_image_from_bytes(bytes: &[u8], _label: &str, mime_type: Option<String>) -> DynamicImage {
    if mime_type.is_none() {
        image::load_from_memory(bytes).expect("Unable to load image from memory")
    } else {
        let mime_type = mime_type.unwrap();
        if mime_type == "image/png" {
            log::warn!("TODO: use png crate for faster image loading");
            image::load_from_memory_with_format(bytes, ImageFormat::Png)
                .expect("Unable to decode png image")
        } else if mime_type == "image/jpeg" {
            image::load_from_memory_with_format(bytes, ImageFormat::Jpeg)
                .expect("Unable to decode jpg image")
        } else {
            panic!("Unsupported mime_type provided: {}", mime_type);
        }
    }
}

pub async fn get_texture_bytes_info_from_gltf<'a>(
    texture: gltf::Texture<'a>,
    buffer_data: &[Vec<u8>],
) -> (Vec<u8>, String, Option<String>) {
    let texture_name = texture.name().unwrap_or("No texture name");
    let texture_source = texture.source().source();
    match texture_source {
        gltf::image::Source::View { view, mime_type } => {
            let parent_buffer_data = &buffer_data[view.buffer().index()];
            let begin = view.offset();
            let end = view.offset() + view.length();
            let buf_dat = &parent_buffer_data[begin..end];
            let mime_type = Some(mime_type.to_string());
            (
                buf_dat.to_vec(),
                String::from(texture_name),
                mime_type.clone(),
            )
        }
        gltf::image::Source::Uri { uri, mime_type } => {
            log::warn!(
                "TODO: get gltf texture from uri {} with mime type {}",
                uri,
                mime_type.unwrap_or("unknown")
            );
            let bin = dream_fs::fs::read_binary(std::path::PathBuf::from(uri), false)
                .await
                .expect("unable to load binary");
            let buf_dat: &[u8] = &bin;
            (buf_dat.to_vec(), String::from(texture_name), None)
        }
    }
}

impl Image {
    pub async fn load_from_bytes_threaded(
        &mut self,
        _bytes: &[u8],
        _label: &str,
        _mime_type: Option<String>,
    ) {
        //     self.dynamic_image = Some(dynamic_image_from_bytes(bytes, label, mime_type));
        //     self.update_rgba();

        get_task_pool().spawn(async {
            println!("Loaded texture in async task");
            log::warn!("Loaded texture in async task");
            // self.dynamic_image = Some(dynamic_image_from_bytes(bytes, label, mime_type));
            // self.update_rgba();
        });
    }

    pub async fn load_from_bytes(&mut self, bytes: &[u8], label: &str, mime_type: Option<String>) {
        self.dynamic_image = Some(dynamic_image_from_bytes(bytes, label, mime_type));
        self.update_rgba();
    }

    pub async fn load_from_gltf_texture<'a>(
        &mut self,
        texture: gltf::Texture<'a>,
        buffer_data: &[Vec<u8>],
    ) {
        let texture = texture.clone();
        let (bytes, label, mime_type) =
            get_texture_bytes_info_from_gltf(texture, buffer_data).await;
        self.load_from_bytes(&bytes, label.as_str(), mime_type)
            .await;
    }

    pub fn to_rgba8(&self) -> RgbaImage {
        self.rgba
            .as_ref()
            .expect("Image not loaded, so rgba does not exist")
            .clone()
    }

    pub fn loaded(&self) -> bool {
        self.dynamic_image.is_some()
    }

    fn update_rgba(&mut self) {
        self.rgba = Some(
            self.dynamic_image
                .as_ref()
                .expect("Image not loaded")
                .to_rgba8(),
        );
    }
}
