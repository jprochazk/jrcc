pub mod ast_boxed;
pub mod ast_bumped;

pub type Span = std::ops::Range<usize>;

#[macro_export]
macro_rules! define_ast {
    ($Ptr:tt, $Vec:tt, $Ctx:ty, $alloc:ident, $alloc_vec:ident) => {
        use crate::Span;
        use rand::Rng;

        impl Default for BinOp {
            fn default() -> Self {
                BinOp::Add
            }
        }
        impl Default for ExprKind<'_> {
            fn default() -> Self {
                Self::Variable("")
            }
        }
        impl Default for StmtKind<'_> {
            fn default() -> Self {
                Self::NullNode
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Hash)]
        pub enum BinOp {
            Add,
            Sub,
            Mul,
            Div,
        }

        #[derive(Debug, Default, PartialEq)]
        pub struct Binary<'a> {
            left: Expr<'a>,
            op: BinOp,
            right: Expr<'a>,
        }

        #[derive(Debug, PartialEq)]
        pub enum ExprKind<'a> {
            Variable(&'a str),
            Binary($Ptr<'a, Binary<'a>>),
            Grouping($Ptr<'a, Expr<'a>>),
            Comma($Vec<'a, Expr<'a>>),
        }

        #[derive(Debug, Default, PartialEq)]
        pub struct Expr<'a> {
            kind: ExprKind<'a>,
            span: Span,
        }

        #[derive(Debug, Default, PartialEq)]
        pub struct VarDecl<'a> {
            name: &'a str,
            initializer: Option<Expr<'a>>,
        }

        #[derive(Debug, PartialEq)]
        pub enum StmtKind<'a> {
            VarDecl($Ptr<'a, VarDecl<'a>>),
            List($Vec<'a, Stmt<'a>>),
            ExprStmt($Ptr<'a, Expr<'a>>),
            NullNode,
        }

        #[derive(Debug, Default, PartialEq)]
        pub struct Stmt<'a> {
            kind: StmtKind<'a>,
            span: Span,
        }

        pub fn gen_slice<'a>(slice: &'a str, rng: &mut rand::prelude::StdRng) -> &'a str {
            let start = rng.gen_range(0..slice.len());
            let end = rng.gen_range(start..slice.len());
            &slice[start..end]
        }

        impl<'a> Expr<'a> {
            pub fn wrangle<H: std::hash::Hasher>(&mut self, ctx: &'a $Ctx, state: &mut H) {
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

                        let left = std::mem::replace(
                            &mut v.left,
                            Expr {
                                span: Default::default(),
                                kind: ExprKind::Variable(""),
                            },
                        );
                        let right = std::mem::replace(
                            &mut v.right,
                            Expr {
                                span: Default::default(),
                                kind: ExprKind::Variable(""),
                            },
                        );

                        if matches!(v.op, BinOp::Add | BinOp::Mul) {
                            v.right = left;
                            v.left = right;
                            ExprKind::Binary(v)
                        } else {
                            let mut c = $alloc_vec(ctx);
                            c.push(left);
                            c.push(right);
                            ExprKind::Comma(c)
                        }
                    }
                    ExprKind::Grouping(mut g) => {
                        g.wrangle(ctx, state);
                        std::mem::replace(&mut g.kind, ExprKind::Variable(""))
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
                ctx: &'a $Ctx,
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
                            ExprKind::Binary($alloc(
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
                            let mut exprs = $alloc_vec(ctx);
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
                        _ => ExprKind::Grouping($alloc(
                            Expr::build_random_tree_with_rng(
                                source,
                                depth - 1,
                                max_width,
                                rng,
                                ctx,
                            ),
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
            pub fn wrangle<H: std::hash::Hasher>(&mut self, ctx: &'a $Ctx, state: &mut H) {
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
                ctx: &'a $Ctx,
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
                ctx: &'a $Ctx,
            ) -> Self {
                let kind = if depth > 0 {
                    match rng.gen::<bool>() {
                        false => {
                            let count = rng.gen_range(0..max_width);
                            let mut stmts = $alloc_vec(ctx);
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
                        true => StmtKind::ExprStmt($alloc(
                            Expr::build_random_tree_with_rng(
                                source,
                                depth - 1,
                                max_width,
                                rng,
                                ctx,
                            ),
                            ctx,
                        )),
                    }
                } else {
                    StmtKind::VarDecl($alloc(
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
    };
}
