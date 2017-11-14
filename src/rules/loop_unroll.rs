use ast;
use context::PhpContext;
use rules::AstOptimizationRule;

#[derive(Debug)]
pub struct LoopUnroll;

impl LoopUnroll {
    pub fn new() -> Self {
        LoopUnroll
    }
}

impl AstOptimizationRule for LoopUnroll {
    fn optimize(&self, ast: &mut ast::ZendAstPtr, ctx: &mut PhpContext) {
        // TODO
    }
}
