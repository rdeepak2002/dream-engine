use boa_engine::class::{Class, ClassBuilder};
use boa_engine::{Context, JsResult, JsValue};
use boa_gc::{Finalize, Trace};

use crate::component::Transform;

#[derive(Debug, Trace, Finalize)]
pub struct EntityJS {
    pub transform: Transform,
}

impl EntityJS {
    pub fn new(transform: Transform) -> Self {
        return Self { transform };
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

    fn init(_class: &mut ClassBuilder) -> JsResult<()> {
        Ok(())
    }
}
