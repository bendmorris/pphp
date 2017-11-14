use ::std::collections::HashMap;
use ::std::fmt::Debug;
use ::std::sync::Arc;
use ::std::sync::Mutex;
use ast::{ZendAst, ZendAstPtr};
use context::PhpContext;

pub mod custom;

mod branch_elim;
mod cond_elim;
mod incr_decr;
mod instanceof;
mod loop_unroll;

pub trait AstOptimizationRule: Send + Debug {
    /**
     * This function should process the provided AST node, modifying it in
     * place if it's a match.
     */
    fn optimize(&self, ast: &mut ZendAstPtr, context: &mut PhpContext);
}

pub type RulesVec = Vec<Box<AstOptimizationRule>>;

lazy_static! {
    static ref OPTIMIZATIONS: Mutex<RulesVec> = {
        let mut map: RulesVec = Vec::new();
        map.push(Box::new(cond_elim::ConditionalElimination::new()));
        map.push(Box::new(branch_elim::BranchElimination::new()));
        map.push(Box::new(incr_decr::IncrDecr::new()));
        map.push(Box::new(instanceof::InstanceOf::new()));
        map.push(Box::new(loop_unroll::LoopUnroll::new()));
        Mutex::new(map)
    };
}

pub fn add_rule(rule: Box<AstOptimizationRule>) {
    let mut rules = OPTIMIZATIONS.lock().unwrap();
    rules.push(rule);
}

pub fn apply_all(ast: ZendAst) {
    let mut modified: bool = false;
    {
        let rules = OPTIMIZATIONS.lock().unwrap();
        let mut context = PhpContext::new();
        for rule in rules.iter() {
            ast_walk!(ast, |ast_ptr: &mut ZendAstPtr, ctx: &mut PhpContext| {
                rule.optimize(ast_ptr, ctx);
                if ast_ptr.is_dirty() {
                    ast_ptr.set_dirty(false);
                    modified = true;
                }
            }, &mut context);
        }
    }
    if modified {
        apply_all(ast)
    }
}
