use ast;
use context::PhpContext;
use rules::AstOptimizationRule;

#[derive(Debug)]
pub struct BranchElimination;

impl BranchElimination {
    pub fn new() -> Self {
        BranchElimination
    }
}

impl AstOptimizationRule for BranchElimination {
    fn optimize(&self, ast: &mut ast::ZendAstPtr, ctx: &mut PhpContext) {
        map_sub!(
            "if (true) { PPHP::$_1; }",
            "PPHP::$_1;",
            ast
        );
        map_sub!(
            "if (false) { PPHP::$_1; }",
            "{}",
            ast
        );
    }
}
