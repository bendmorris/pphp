use ast;
use context::PhpContext;
use rules::AstOptimizationRule;

#[derive(Debug)]
pub struct IncrDecr;

impl IncrDecr {
    pub fn new() -> Self {
        IncrDecr
    }
}

impl AstOptimizationRule for IncrDecr {
    fn optimize(&self, ast: &mut ast::ZendAstPtr, ctx: &mut PhpContext) {
        // replaces postfix with prefix increment/decrement in for loops
        map_sub!(
            "for (PPHP::$_1; PPHP::$_2; PPHP::$iter++) PPHP::$_3;",
            "for (PPHP::$_1; PPHP::$_2; ++PPHP::$iter) PPHP::$_3;",
            ast
        );
        map_sub!(
            "for (PPHP::$_1; PPHP::$_2; PPHP::$iter--) PPHP::$_3;",
            "for (PPHP::$_1; PPHP::$_2; --PPHP::$iter) PPHP::$_3;",
            ast
        );
        // there's a dedicated operator for this, use it!
        map_sub!(
            "PPHP::$_1 += 1;",
            "++PPHP::$_1;",
            ast
        );
        map_sub!(
            "PPHP::$_1 -= 1;",
            "--PPHP::$_1;",
            ast
        );
        // use dedicated in place modification ops
        for op in vec!["*", "/", "+", "-", "**", "%", "<<", ">>", ".", "|", "&"] {
            map_sub!(
                &format!("PPHP::$_1 = PPHP::$_1 {} PPHP::$_2;", op),
                &format!("PPHP::$_1 {}= PPHP::$_2;", op),
                ast
            );
        }
    }
}
