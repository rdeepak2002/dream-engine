use boa_engine::class::{Class, ClassBuilder};
use boa_engine::object::{JsObject, ObjectInitializer};
use boa_engine::property::Attribute;
use boa_engine::{Context, JsResult, JsValue};
use boa_gc::{Finalize, Trace};
use cgmath::num_traits::ToPrimitive;

use dream_ecs::component::Transform;
use dream_ecs::entity::Entity;
use dream_math::Vector3;

#[derive(Debug, Trace, Finalize)]
pub struct Vector3JS {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3JS {
    pub fn new(vector: Vector3) -> Self {
        Self {
            x: vector.x,
            y: vector.y,
            z: vector.z,
        }
    }

    pub fn js_object(&self, context: &mut Context) -> JsObject {
        let js_obj = ObjectInitializer::new(context)
            .property("x", self.x, Attribute::all())
            .property("y", self.y, Attribute::all())
            .property("z", self.z, Attribute::all())
            .build();
        return js_obj;
    }

    pub fn get_vector3(&self) -> Vector3 {
        return Vector3::from(self.x, self.y, self.z);
    }
}

impl Class for Vector3JS {
    const NAME: &'static str = "Vector3";
    const LENGTH: usize = 2;

    fn constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
        let x = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?
            .to_f32()
            .unwrap();

        let y = args
            .get(1)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?
            .to_f32()
            .unwrap();

        let z = args
            .get(2)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?
            .to_f32()
            .unwrap();

        let vector3_js = Vector3JS::new(Vector3::from(x, y, z));

        Ok(vector3_js)
    }

    fn init(_class: &mut ClassBuilder) -> JsResult<()> {
        Ok(())
    }
}

#[derive(Debug, Trace, Finalize)]
pub struct EntityJS {
    pub handle: u64,
}

impl EntityJS {
    fn set_position(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| context.construct_type_error("`this` is not a `EntityJS` object"))?;

        let entity_inner_id = this.handle;
        let entity: Option<Entity> = Some(Entity::from_handle(entity_inner_id));

        if entity.is_some() {
            let entity = entity.unwrap();
            let transform: Option<Transform> = entity.get_component();
            if transform.is_some() {
                let mut transform = transform.unwrap();
                let new_position = args
                    .get(0)
                    .ok_or_else(|| context.construct_type_error("No first argument provided"))?
                    .as_object()
                    .and_then(|obj| obj.downcast_ref::<Vector3JS>())
                    .ok_or_else(|| context.construct_type_error("Not a `Vector3` object"))?;
                transform.position = new_position.get_vector3();
                entity.add_component(transform);
            }
        }

        Ok(JsValue::undefined())
    }

    fn get_position(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| context.construct_type_error("`this` is not a `EntityJS` object"))?;

        let entity_inner_id = this.handle;
        let entity: Option<Entity> = Some(Entity::from_handle(entity_inner_id));
        if entity.is_some() {
            let entity = entity.unwrap();
            let transform: Option<Transform> = entity.get_component();
            return if let Some(transform) = transform {
                let position = transform.position;
                let position_js = Vector3JS::new(position);
                let position_js_obj = position_js.js_object(context);
                Ok(JsValue::Object(position_js_obj))
            } else {
                Ok(JsValue::undefined())
            };
        }

        Ok(JsValue::undefined())
    }
}

impl Class for EntityJS {
    const NAME: &'static str = "Entity";
    const LENGTH: usize = 2;

    fn constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
        let handle = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_u32(context)?
            .to_u64()
            .unwrap();

        let entity_js = EntityJS { handle };

        Ok(entity_js)
    }

    fn init(class: &mut ClassBuilder) -> JsResult<()> {
        class.method("getPosition", 0, Self::get_position);
        class.method("setPosition", 0, Self::set_position);
        Ok(())
    }
}
