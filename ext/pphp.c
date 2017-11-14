/* pphp extension for PHP */

#ifdef HAVE_CONFIG_H
# include "config.h"
#endif

#include "php.h"
#include "ext/standard/info.h"
#include "php_pphp.h"
#include "zend_extensions.h"
#include "zend_ast.h"
#include "zend_globals.h"
#include "zend_language_scanner.h"
#include "zend_language_scanner_defs.h"
#include "rust_pphp.h"

ZEND_EXTENSION();

static zend_ast_process_t zend_orig_ast_process;

void pphp_ast_process(zend_ast *ast) {
    // call the original processor first
    if (zend_orig_ast_process)
        zend_orig_ast_process(ast);
    rust_pphp_optimize_ast(ast);
}

void pphp_enable() {
    // override zend_ast_process
    zend_orig_ast_process = zend_ast_process;
    zend_ast_process = pphp_ast_process;
}

void pphp_disable() {
    // replace original zend_ast_process
    zend_ast_process = zend_orig_ast_process;
}

PHP_FUNCTION(pphp_add_rule) {
    char *from;
    size_t from_len;
    char *to;
    size_t to_len;

    ZEND_PARSE_PARAMETERS_START(2, 2)
        Z_PARAM_STRING(from, from_len)
        Z_PARAM_STRING(to, to_len)
    ZEND_PARSE_PARAMETERS_END();

    RETURN_BOOL(rust_pphp_add_rule(from, to));
}

PHP_FUNCTION(pphp_set_debug_trace) {
    zend_bool enabled;

    ZEND_PARSE_PARAMETERS_START(1, 1)
        Z_PARAM_BOOL(enabled)
    ZEND_PARSE_PARAMETERS_END();

    rust_pphp_set_debug_trace(enabled);
}

/* {{{ PHP_RINIT_FUNCTION
 */
PHP_RINIT_FUNCTION(pphp)
{
#if defined(ZTS) && defined(COMPILE_DL_PPHP)
    ZEND_TSRMLS_CACHE_UPDATE();
#endif

    pphp_enable();

    return SUCCESS;
}
/* }}} */

/* {{{ PHP_MINFO_FUNCTION
 */
PHP_MINFO_FUNCTION(pphp)
{
    php_info_print_table_start();
    php_info_print_table_header(2, "pphp support", "enabled");
    php_info_print_table_end();
}
/* }}} */

/* {{{ arginfo
 */
ZEND_BEGIN_ARG_INFO(arginfo_pphp_add_rule, 0)
    ZEND_ARG_INFO(0, fromPattern)
    ZEND_ARG_INFO(0, toPattern)
ZEND_END_ARG_INFO()

ZEND_BEGIN_ARG_INFO(arginfo_pphp_set_debug_trace, 0)
    ZEND_ARG_INFO(0, enabled)
ZEND_END_ARG_INFO()
/* }}} */

/* {{{ pphp_functions[]
 */
const zend_function_entry pphp_functions[] = {
    PHP_FE(pphp_add_rule, arginfo_pphp_add_rule)
    PHP_FE(pphp_set_debug_trace, arginfo_pphp_set_debug_trace)
    PHP_FE_END
};
/* }}} */

/* {{{ pphp_module_entry
 */
zend_module_entry pphp_module_entry = {
    STANDARD_MODULE_HEADER,
    "pphp",                    /* Extension name */
    pphp_functions,            /* zend_function_entry */
    NULL,                            /* PHP_MINIT - Module initialization */
    NULL,                            /* PHP_MSHUTDOWN - Module shutdown */
    PHP_RINIT(pphp),            /* PHP_RINIT - Request initialization */
    NULL,                            /* PHP_RSHUTDOWN - Request shutdown */
    PHP_MINFO(pphp),            /* PHP_MINFO - Module info */
    PHP_PPHP_VERSION,        /* Version */
    STANDARD_MODULE_PROPERTIES
};
/* }}} */

#ifdef COMPILE_DL_PPHP
# ifdef ZTS
ZEND_TSRMLS_CACHE_DEFINE()
# endif
ZEND_GET_MODULE(pphp)
#endif
