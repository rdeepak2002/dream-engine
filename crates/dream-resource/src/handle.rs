use std::marker::PhantomData;

pub struct Handle<T> {
    guid: String,
    marker: PhantomData<fn() -> T>, // this gives Handle safe Send/Sync impls
}
