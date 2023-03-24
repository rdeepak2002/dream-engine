use boa_engine::class::{Class, ClassBuilder};
use boa_engine::{builtins::JsArgs, Context, JsResult, JsValue};
use boa_gc::{Finalize, Trace};

use crate::component::Transform;
use crate::entity::Entity;

#[derive(Debug, Trace, Finalize)]
pub struct EntityJS {
    pub transform: Transform,
}

impl EntityJS {
    pub fn new(transform: Transform) -> Self {
        return Self { transform };
    }

    // pub fn from_dream_entity(entity: &Entity) -> JsResult<Self> {
    //     let mut transform = None;
    //
    //     if entity.has_transform() {
    //         transform = Some(entity.get_transform().unwrap());
    //     }
    //
    //     let entity_js = EntityJS { Some(transform) };
    //
    //     Ok(entity_js)
    // }

    pub fn say_hello(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| context.construct_type_error("`this` is not a `EntityJS` object"))?;

        println!("Entity Transform {}", this.transform.to_string());
        log::warn!("Entity Transform {}", this.transform.to_string());

        Ok(JsValue::undefined())
    }
}

impl Class for EntityJS {
    const NAME: &'static str = "EntityJS";
    const LENGTH: usize = 2;

    fn constructor(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<Self> {
        let entity_js = EntityJS {
            transform: Transform::new(),
        };

        Ok(entity_js)
    }

    /// Here is where the class is initialized, to be inserted into the global object.
    fn init(class: &mut ClassBuilder) -> JsResult<()> {
        class.method("say_hello", 0, Self::say_hello);

        Ok(())
    }
}
