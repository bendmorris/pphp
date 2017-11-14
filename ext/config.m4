PHP_ARG_ENABLE(pphp, whether to enable Zend OPcache support,
	[--disable-pphp		Disable Zend OPcache support], yes)

PHP_ARG_ENABLE(pphp_rust_target, rust target to link against,
	[--pphp-rust-target		Rust target (debug, release...)], debug)

if test "$PHP_PPHP" != "no"; then
	PHP_SUBST(PPHP_SHARED_LIBADD)
	PHP_ADD_LIBPATH("../target/$PHP_PPHP_RUST_TARGET")
	PHP_ADD_LIBRARY(pphp, 1, PPHP_SHARED_LIBADD)
	PHP_NEW_EXTENSION(pphp, pphp.c, $ext_shared)
fi
