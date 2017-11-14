/* pphp extension for PHP */

#ifndef PHP_PPHP_H
# define PHP_PPHP_H

#ifndef ZEND_EXT_API
# if WIN32|WINNT
#  define ZEND_EXT_API __declspec(dllexport)
# elif defined(__GNUC__) && __GNUC__ >= 4
#  define ZEND_EXT_API __attribute__ ((visibility("default")))
# else
#  define ZEND_EXT_API
# endif
#endif

# define PHP_PPHP_VERSION "0.1.0"

# if defined(ZTS) && defined(COMPILE_DL_PPHP)
ZEND_TSRMLS_CACHE_EXTERN()
# endif

zend_op_array *pphp_compile_string(zval *source_string, char *filename);

#endif    /* PHP_PPHP_H */
