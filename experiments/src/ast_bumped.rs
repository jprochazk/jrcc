pub type Vec<'a, T> = bumpalo::collections::Vec<'a, T>;
pub type Box<'a, T> = bumpalo::boxed::Box<'a, T>;

pub fn alloc<T>(t: T, ctx: &'_ bumpalo::Bump) -> Box<'_, T> {
    Box::new_in(t, ctx)
}

pub fn alloc_vec<T>(ctx: &'_ bumpalo::Bump) -> Vec<'_, T> {
    Vec::new_in(ctx)
}

crate::define_ast!(Box, Vec, bumpalo::Bump, alloc, alloc_vec);
