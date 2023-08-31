use crossbeam_channel::{unbounded, Receiver};
use image::{DynamicImage, ImageFormat, RgbaImage};

// use dream_tasks::task_pool::get_async_task_pool;

#[derive(Clone, Default)]
pub struct Image {
    dynamic_image: Option<DynamicImage>,
    rgba8: Option<RgbaImage>,
    receiver: Option<Receiver<(DynamicImage, RgbaImage)>>,
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

pub fn get_texture_bytes_info_from_gltf<'a>(
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
        gltf::image::Source::Uri {
            uri,
            mime_type: _mime_type,
        } => {
            let bin = dream_fs::fs::read_binary(std::path::PathBuf::from(uri), false)
                .expect("unable to load binary");
            let buf_dat: &[u8] = &bin;
            (buf_dat.to_vec(), String::from(texture_name), None)
        }
    }
}

impl Image {
    pub fn load_from_bytes_threaded(
        &mut self,
        bytes: &[u8],
        label: &str,
        mime_type: Option<String>,
    ) {
        let bytes = bytes.to_owned();
        let label = label.to_owned();
        let mime_type = mime_type;
        let (sx, rx) = unbounded();

        rayon::spawn(move || {
            let dynamic_image = dynamic_image_from_bytes(&bytes, label.as_str(), mime_type);
            let rgba8 = dynamic_image.to_rgba8();
            sx.send((dynamic_image, rgba8))
                .expect("Unable to send dynamic image contents");
        });

        self.receiver = Some(rx);
    }

    pub fn load_from_gltf_texture_threaded(
        &mut self,
        texture: gltf::Texture,
        buffer_data: &[Vec<u8>],
    ) {
        let texture = texture.clone();
        let (bytes, label, mime_type) = get_texture_bytes_info_from_gltf(texture, buffer_data);
        self.load_from_bytes_threaded(&bytes, label.as_str(), mime_type);
    }

    pub fn update(&mut self) {
        if self.receiver.is_some() {
            if let Some((dynamic_image, rgba8)) = self.receiver.clone().unwrap().try_iter().last() {
                self.dynamic_image = Some(dynamic_image);
                self.rgba8 = Some(rgba8);
                self.receiver = None;
            }
        }
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
        let (bytes, label, mime_type) = get_texture_bytes_info_from_gltf(texture, buffer_data);
        self.load_from_bytes(&bytes, label.as_str(), mime_type)
            .await;
    }

    pub fn to_rgba8(&self) -> RgbaImage {
        self.rgba8
            .as_ref()
            .expect("Image not loaded, so rgba does not exist")
            .clone()
    }

    pub fn loaded(&self) -> bool {
        self.dynamic_image.is_some()
    }

    fn update_rgba(&mut self) {
        self.rgba8 = Some(
            self.dynamic_image
                .as_ref()
                .expect("Image not loaded")
                .to_rgba8(),
        );
    }
}
