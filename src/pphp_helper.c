#include "zend_ast.h"
#include "zend_string.h"
#include "zend_globals.h"
#include "zend_language_parser.h"
#include "zend_language_scanner.h"
#include "zend_language_scanner_defs.h"
#include "zend_smart_str_public.h"
#include "zend_smart_str.h"
#include "php.h"
#include "ext/standard/php_var.h"

zend_bool ast_is_list(zend_ast *ast) {
    return zend_ast_is_list(ast);
}

zend_bool ast_is_decl(zend_ast *ast) {
    zend_ast_kind kind = ast->kind;
    return kind == ZEND_AST_FUNC_DECL || kind == ZEND_AST_CLOSURE
        || kind == ZEND_AST_METHOD || kind == ZEND_AST_CLASS;
}

uint32_t ast_num_children(zend_ast *ast) {
    return zend_ast_get_num_children(ast);
}

uint32_t ast_size(zend_ast *ast) {
    if (ast_is_decl(ast)) {
        return sizeof(zend_ast_decl);
    } else if (ast_is_list(ast)) {
        zend_ast_list *list = zend_ast_get_list(ast);
        return sizeof(zend_ast_list) - sizeof(zend_ast *) + sizeof(zend_ast *) * list->children;
    } else {
        return sizeof(zend_ast) - sizeof(zend_ast *) + sizeof(zend_ast *) * ast_num_children(ast);
    }
}

zval *ast_zval(zend_ast *ast) {
    return &((zend_ast_zval *) ast)->val;
}

zend_string *zval_string(zval *zval) {
    return zval_get_string(zval);
}

zend_ast *get_ast(char *code_str) {
    zval code_zv;
    zend_bool original_in_compilation;
    zend_lex_state original_lex_state;
    zend_ast *ast;

    zend_string *code;
    code = zend_string_init(code_str, strlen(code_str), 0);

    char *filename;
    filename = "<pphp>";

    ZVAL_STR_COPY(&code_zv, code);

    original_in_compilation = CG(in_compilation);
    CG(in_compilation) = 1;

    zend_save_lexical_state(&original_lex_state);
    if (zend_prepare_string_for_scanning(&code_zv, filename) == SUCCESS) {
        CG(ast) = NULL;
        CG(ast_arena) = zend_arena_create(1024 * 32);
        LANG_SCNG(yy_state) = yycINITIAL;

        if (zendparse() != 0) {
            zend_ast_destroy(CG(ast));
            zend_arena_destroy(CG(ast_arena));
            CG(ast) = NULL;
        }
    }

    /* restore_lexical_state changes CG(ast) and CG(ast_arena) */
    ast = CG(ast);

    zend_restore_lexical_state(&original_lex_state);
    CG(in_compilation) = original_in_compilation;

    zval_dtor(&code_zv);

    return ast;
}

void set_ast(zend_ast *ast) {
    CG(ast) = ast;
}

static zend_always_inline zend_string *smart_str_extract(smart_str *str) {
    if (str->s) {
        zend_string *res;
        smart_str_0(str);
        res = str->s;
        str->s = NULL;
        return res;
    } else {
        return ZSTR_EMPTY_ALLOC();
    }
}

zend_string *var_export(zval *v) {
    smart_str buf = {0};
    php_var_export_ex(v, 1, &buf);
    smart_str_0(&buf);
    return smart_str_extract(&buf);
}
