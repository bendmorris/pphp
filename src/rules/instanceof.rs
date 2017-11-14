use ast;
use context::PhpContext;
use rules::AstOptimizationRule;

#[derive(Debug)]
pub struct InstanceOf;

impl InstanceOf {
    pub fn new() -> Self {
        InstanceOf
    }
}

impl AstOptimizationRule for InstanceOf {
    fn optimize(&self, ast: &mut ast::ZendAstPtr, ctx: &mut PhpContext) {
        // replaces is_a calls with instanceof constructs
        // TODO: constrain to string constant
        map_sub!(
            "is_a(PPHP::$_1, PPHP::$_2);",
            "PPHP::$_1 instanceof PPHP::$_2;",
            ast
        );
    }
}
