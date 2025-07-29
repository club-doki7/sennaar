use lalrpop_util::lalrpop_mod;

mod tok;
lalrpop_mod!(lalr, "/panspace/lalr.rs");

fn foo() {
    lalr::ExpressionParser::new()
        .parse("1 + 2 * 3")
        .unwrap();
}
