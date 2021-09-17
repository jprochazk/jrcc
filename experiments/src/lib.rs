pub mod ast_boxed;

pub type Span = std::ops::Range<usize>;

#[test]
fn test() {
    use ast_boxed::Stmt;

    // assert!(false, "{:#?}", Stmt::build_random_tree("source", 0, 7, 100, &()));
    let mut stmt = Stmt::build_random_tree("source", 1, 5, 7, &());
    eprintln!("{:#?}", stmt);

    let mut state = std::collections::hash_map::DefaultHasher::new();
    stmt.wrangle(&(), &mut state);

    panic!("{:#?}", stmt);
}
