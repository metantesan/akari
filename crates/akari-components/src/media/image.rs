use akari_core::{Element, Image as ImageData, Key};
use std::marker::PhantomData;

/// Local raster image displayed by Akari's renderer.
pub struct Image<M = ()> {
    key: Key,
    image: ImageData,
    message: PhantomData<fn() -> M>,
}

impl<M> Image<M> {
    pub fn new(key: Key, image: ImageData) -> Self {
        Self {
            key,
            image,
            message: PhantomData,
        }
    }

    pub fn build(self) -> Element<M>
    where
        M: Clone,
    {
        Element::image(self.key, self.image)
    }
}
