#[derive(shipyard::Component, Debug)]
pub struct Transform {
    x: f32,
    y: f32,
    z: f32,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

pub struct Scene {
    pub name: &'static str,
    pub handle: shipyard::World,
}

impl Scene {
    pub fn new() -> Self {
        let name = "scene";
        let handle = shipyard::World::new();
        return Self { name, handle };
    }

    pub fn create_entity(mut self) -> Entity {
        let handle = self.handle.add_entity(());
        let scene: std::rc::Rc<Scene> = std::rc::Rc::new(self);
        let entity = Entity::new(scene, handle);
        return entity;
    }
}

pub struct Entity {
    pub scene: std::rc::Rc<Scene>,
    pub handle: shipyard::EntityId,
}

impl Entity {
    pub fn new(scene: std::rc::Rc<Scene>, handle: shipyard::EntityId) -> Self {
        Self { scene, handle }
    }
}
