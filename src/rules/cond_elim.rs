use ast;
use context::PhpContext;
use rules::AstOptimizationRule;

#[derive(Debug)]
pub struct ConditionalElimination;

impl ConditionalElimination {
    pub fn new() -> Self {
        ConditionalElimination
    }
}

impl AstOptimizationRule for ConditionalElimination {
    fn optimize(&self, ast: &mut ast::ZendAstPtr, ctx: &mut PhpContext) {
        map_sub!(
            "true && PPHP::$_1;",
            "PPHP::$_1;",
            ast
        );
        map_sub!(
            "PPHP::$_1 && true;",
            "PPHP::$_1;",
            ast
        );
        map_sub!(
            "true || PPHP::$_1;",
            "true;",
            ast
        );
        map_sub!(
            "PPHP::$_1 || true;",
            "true;",
            ast
        );
        map_sub!(
            "false && PPHP::$_1;",
            "false;",
            ast
        );
        map_sub!(
            "PPHP::$_1 && false;",
            "false;",
            ast
        );
        map_sub!(
            "false || PPHP::$_1;",
            "PPHP::$_1;",
            ast
        );
        map_sub!(
            "PPHP::$_1 || false;",
            "PPHP::$_1;",
            ast
        );
    }
}
