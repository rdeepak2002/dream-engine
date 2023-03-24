/// EXAMPLE ON HOW TO REGISTER CLASS WITH BOA SCRIPTING ENGINE
use boa_engine::class::{Class, ClassBuilder};
use boa_engine::property::Attribute;
use boa_engine::{builtins::JsArgs, Context, JsResult, JsValue};
use boa_gc::{Finalize, Trace};

#[derive(Debug, Trace, Finalize)]
pub struct Person {
    /// The name of the person.
    name: String,
    /// The age of the person.
    age: u8,
}

impl Person {
    /// Says "hello" using the name and the age of a `Person`.
    fn say_hello(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this = this
            .as_object()
            .and_then(|obj| obj.downcast_ref::<Self>())
            .ok_or_else(|| context.construct_type_error("`this` is not a `Person` object"))?;

        println!("Hello {}-year-old {}!", this.age, this.name);
        log::warn!("Hello {}-year-old {}!", this.age, this.name);

        Ok(JsValue::undefined())
    }
}

impl Class for Person {
    const NAME: &'static str = "Person";
    const LENGTH: usize = 2;

    // This is what is called when we construct a `Person` with the expression `new Person()`.
    fn constructor(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<Self> {
        let name = args.get_or_undefined(0).to_string(context)?;
        let age = args.get_or_undefined(1).to_u32(context)?;

        if !(0..=150).contains(&age) {
            context.throw_range_error(format!("invalid age `{age}`. Must be between 0 and 150"))?;
        }

        let age = u8::try_from(age).expect("we already checked that it was in range");

        let person = Person {
            name: name.to_string(),
            age,
        };

        Ok(person)
    }

    /// Here is where the class is initialized, to be inserted into the global object.
    fn init(class: &mut ClassBuilder) -> JsResult<()> {
        class.method("say_hello", 0, Self::say_hello);

        Ok(())
    }
}
