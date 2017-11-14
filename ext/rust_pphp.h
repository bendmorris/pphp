#ifndef RUST_PPHP_H
# define RUST_PPHP_H

void rust_pphp_optimize_ast(zend_ast *ast);
zend_bool rust_pphp_add_rule(char *from, char *to);
void rust_pphp_set_debug_trace(unsigned char enabled);

#endif
