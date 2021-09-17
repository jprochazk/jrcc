use std::marker::PhantomData;

use rand::Rng;

use crate::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Vec<'a, T> {
    v: std::vec::Vec<T>,
    _pd: PhantomData<&'a ()>,
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

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binary<'a> {
    left: Expr<'a>,
    op: BinOp,
    right: Expr<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind<'a> {
    Variable(&'a str),
    Binary(Box<Binary<'a>>),
    Grouping(Box<Expr<'a>>),
    Comma(Vec<'a, Expr<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr<'a> {
    kind: ExprKind<'a>,
    span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDecl<'a> {
    name: &'a str,
    initializer: Option<Expr<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind<'a> {
    VarDecl(Box<VarDecl<'a>>),
    List(Vec<'a, Stmt<'a>>),
    ExprStmt(Box<Expr<'a>>),
    NullNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt<'a> {
    kind: StmtKind<'a>,
    span: Span,
}

pub fn alloc<T>(t: T, _ctx: &()) -> Box<T> {
    Box::new(t)
}

pub fn alloc_vec<T>(_ctx: &()) -> Vec<T> {
    Vec {
        v: std::vec::Vec::new(),
        _pd: PhantomData,
    }
}

pub fn gen_slice<'a>(slice: &'a str, rng: &mut rand::prelude::StdRng) -> &'a str {
    let start = rng.gen_range(0..slice.len());
    let end = rng.gen_range(start..slice.len());
    &slice[start..end]
}

impl<'a> Expr<'a> {
    pub fn wrangle<H: std::hash::Hasher>(&mut self, ctx: &'a (), state: &mut H) {
        let kind = std::mem::replace(&mut self.kind, ExprKind::Variable(""));
        self.kind = match kind {
            ExprKind::Variable(v) => {
                std::hash::Hash::hash(v, state);
                ExprKind::Variable(v)
            }
            ExprKind::Binary(mut v) => {
                std::hash::Hash::hash(&v.op, state);
                v.left.wrangle(ctx, state);
                v.right.wrangle(ctx, state);

                if matches!(v.op, BinOp::Add | BinOp::Mul) {
                    std::mem::swap(&mut v.left, &mut v.right);
                    ExprKind::Binary(v)
                } else {
                    let mut c = alloc_vec(ctx);
                    c.push(v.left);
                    c.push(v.right);
                    ExprKind::Comma(c)
                }
            }
            ExprKind::Grouping(mut g) => {
                g.wrangle(ctx, state);
                g.kind
            }
            ExprKind::Comma(mut c) => {
                c.iter_mut().for_each(|c| c.wrangle(ctx, state));
                c.reverse();
                ExprKind::Comma(c)
            }
        };
    }

    pub fn build_random_tree_with_rng(
        source: &'a str,
        depth: usize,
        max_width: usize,
        rng: &mut rand::prelude::StdRng,
        ctx: &'a (),
    ) -> Self {
        let kind = if depth > 0 {
            match rng.gen_range(0..3) {
                0 if max_width >= 2 => {
                    let left = Expr::build_random_tree_with_rng(
                        source,
                        depth - 1,
                        max_width.saturating_sub(2),
                        rng,
                        ctx,
                    );
                    let right = Expr::build_random_tree_with_rng(
                        source,
                        depth - 1,
                        max_width.saturating_sub(2),
                        rng,
                        ctx,
                    );
                    ExprKind::Binary(alloc(
                        Binary {
                            left,
                            right,
                            op: match rng.gen_range(0..4) {
                                0 => BinOp::Add,
                                1 => BinOp::Sub,
                                2 => BinOp::Mul,
                                3 => BinOp::Div,
                                _ => unreachable!(),
                            },
                        },
                        ctx,
                    ))
                }
                1 if max_width >= 2 => {
                    let count = rng.gen_range(0..max_width);
                    let mut exprs = alloc_vec(ctx);
                    for _ in 0..count {
                        exprs.push(Expr::build_random_tree_with_rng(
                            source,
                            depth - 1,
                            max_width - count,
                            rng,
                            ctx,
                        ));
                    }
                    ExprKind::Comma(exprs)
                }
                _ => ExprKind::Grouping(alloc(
                    Expr::build_random_tree_with_rng(source, depth - 1, max_width, rng, ctx),
                    ctx,
                )),
            }
        } else {
            ExprKind::Variable(gen_slice(source, rng))
        };

        Expr {
            kind,
            span: rng.gen()..rng.gen(),
        }
    }
}

impl<'a> Stmt<'a> {
    pub fn wrangle<H: std::hash::Hasher>(&mut self, ctx: &'a (), state: &mut H) {
        let kind = std::mem::replace(&mut self.kind, StmtKind::NullNode);
        self.kind = match kind {
            StmtKind::VarDecl(mut v) => {
                std::hash::Hash::hash(&v.name, state);
                if let Some(e) = v.initializer.as_mut() {
                    e.wrangle(ctx, state)
                }
                StmtKind::VarDecl(v)
            }
            StmtKind::ExprStmt(mut e) => {
                e.wrangle(ctx, state);
                StmtKind::ExprStmt(e)
            }
            StmtKind::List(mut c) => {
                c.iter_mut().for_each(|c| c.wrangle(ctx, state));
                c.reverse();
                StmtKind::List(c)
            }
            StmtKind::NullNode => StmtKind::NullNode,
        };
    }

    pub fn build_random_tree(
        source: &'a str,
        seed: u64,
        depth: usize,
        max_width: usize,
        ctx: &'a (),
    ) -> Self {
        use rand::{prelude::StdRng, SeedableRng};
        let mut rng = StdRng::seed_from_u64(seed);
        Self::build_random_tree_with_rng(source, depth, max_width, &mut rng, ctx)
    }

    pub fn build_random_tree_with_rng(
        source: &'a str,
        depth: usize,
        max_width: usize,
        rng: &mut rand::prelude::StdRng,
        ctx: &'a (),
    ) -> Self {
        let kind = if depth > 0 {
            match rng.gen::<bool>() {
                false => {
                    let count = rng.gen_range(0..max_width);
                    let mut stmts = alloc_vec(ctx);
                    for _ in 0..count {
                        stmts.push(Stmt::build_random_tree_with_rng(
                            source,
                            depth - 1,
                            max_width - count,
                            rng,
                            ctx,
                        ));
                    }
                    StmtKind::List(stmts)
                }
                true => StmtKind::ExprStmt(alloc(
                    Expr::build_random_tree_with_rng(source, depth - 1, max_width, rng, ctx),
                    ctx,
                )),
            }
        } else {
            StmtKind::VarDecl(alloc(
                VarDecl {
                    name: gen_slice(source, rng),
                    initializer: if rng.gen::<bool>() {
                        Some(Expr::build_random_tree_with_rng(
                            source,
                            depth - 1,
                            max_width,
                            rng,
                            ctx,
                        ))
                    } else {
                        None
                    },
                },
                ctx,
            ))
        };

        Stmt {
            kind,
            span: rng.gen()..rng.gen(),
        }
    }
}
