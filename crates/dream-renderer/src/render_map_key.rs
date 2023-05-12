#[derive(Hash, PartialEq, Eq, Clone)]
struct RenderMapKey {
    pub model_guid: String,
    pub mesh_index: i32,
}
