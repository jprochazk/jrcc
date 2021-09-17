use std::marker::PhantomData;
#[derive(Debug, Clone, PartialEq)]
pub struct Vec<'a, T> {
    v: std::vec::Vec<T>,
    _pd: PhantomData<&'a ()>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Box<'a, T> {
    b: std::boxed::Box<T>,
    _pd: PhantomData<&'a ()>,
}

impl<'a, T> std::ops::Deref for Box<'a, T> {
    type Target = std::boxed::Box<T>;

    fn deref(&self) -> &Self::Target {
        &self.b
    }
}
impl<'a, T> std::ops::DerefMut for Box<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.b
    }
}

impl<'a, T> std::ops::Deref for Vec<'a, T> {
    type Target = std::vec::Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.v
    }
}
impl<'a, T> std::ops::DerefMut for Vec<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.v
    }
}

pub fn alloc<T>(t: T, _ctx: &'_ ()) -> Box<'_, T> {
    Box {
        b: std::boxed::Box::new(t),
        _pd: PhantomData,
    }
}

pub fn alloc_vec<T>(_ctx: &'_ ()) -> Vec<'_, T> {
    Vec {
        v: std::vec::Vec::new(),
        _pd: PhantomData,
    }
}

crate::define_ast!(Box, Vec, (), alloc, alloc_vec);
