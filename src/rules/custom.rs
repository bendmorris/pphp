use ast;
use context::PhpContext;
use rules::AstOptimizationRule;

#[derive(Debug)]
pub struct CustomSubstitution {
    from_pattern: String,
    to_pattern: String,
}

impl CustomSubstitution {
    pub fn try_create(from: String, to: String) -> Option<Self> {
        if ast::parse_pattern(&from).is_some() && ast::parse_pattern(&to).is_some() {
            Some(CustomSubstitution::new(from, to))
        } else {
            None
        }
    }

    pub fn new(from: String, to: String) -> Self {
        CustomSubstitution {from_pattern: from, to_pattern: to}
    }
}

impl AstOptimizationRule for CustomSubstitution {
    fn optimize(&self, ast: &mut ast::ZendAstPtr, ctx: &mut PhpContext) {
        map_sub!(
            &self.from_pattern,
            &self.to_pattern,
            ast
        );
    }
}
