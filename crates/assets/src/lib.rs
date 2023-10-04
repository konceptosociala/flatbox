use std::marker::PhantomData;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AssetHandle<A> {
    index: usize,
    __phantom_data: PhantomData<A>
}

impl<A> AssetHandle<A> {
    pub fn invalid() -> AssetHandle<A> {
        AssetHandle { 
            index: usize::MAX, 
            __phantom_data: PhantomData
        }
    }
}