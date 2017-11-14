PPHP is a PHP AST preprocessing extension written in Rust and C. It gives users the ability to transform PHP's abstract syntax tree at parse time.


Building
--------

Use the root directory Makefile to build, specifying the path to your PHP header files;

    make PHP_INCLUDE_DIR=/usr/include/php/20151012/

Ultimately this will generate ext/modules/pphp.so, which you can add to your php.ini:

    extension=/path/to/pphp.so


Project structure
-----------------

- `ext` contains the usual PHP extension skeleton code.
- `src` contains the Rust source, plus a C helper to expose inlined function and C macros to Rust.

The dependency chain:

- The extension requires libpphp which is built from Rust. The magic happens by inserting `pphp_optimize_ast` in the Zend framework's AST processing hook `zend_ast_process` when the extension activates.
- libpphp depends on bindings to PHP's zend_ast.h generated with rust-bindgen, as well as a C helper library, src/pphp_helper.c, which exposes some utility functions containing inlined functions and C macros which won't be available in the compiled library.


Substitution rules
------------------

The most fundamental type of AST transformation in PPHP is substitution, which replaces any AST node matching a given pattern with an alternative node. For convenience, substitution rules are written directly in PHP, without needing to interact with any of PHP's internal AST structures. AST optimization rules in Rust implement the AstOptimizationRule trait and define an `optimize` function which may apply one or more subsitutions which modify the AST in place. For example:

```rust
#[derive(Debug)]
pub struct SampleRule;

impl SampleRule {
	pub fn new() -> Self {
		SampleRule
	}
}

impl AstOptimizationRule for SampleRule {
	fn optimize(&self, ast: &mut ast::ZendAstPtr, ctx: &mut PhpContext) {
		map_sub!(
			"echo PPHP::$_something;",
			"echo 'I see you were trying to echo: ' . PPHP::$_something;",
			ast
		);
	}
}

```

Substitution rules compare against AST nodes in the PHP code being parsed. When the kind, value and attributes of the pattern node and the actual node are identical, the match succeeds. When the match pattern contains a variable in the format `PPHP::$myVar`, it will match *any* expression in the matched AST, binding that node to the variable `myVar` which can be referenced in the replacement pattern. If the same variable is repeated (as in `PPHP::$_1 = PPHP::$_1 + PPHP::$_2;`) the pattern will only match if all corresponding nodes are identical in the matched AST.

The above sample rule will match any echo statement in PHP, binding the expression after `echo` to `_something`. It will then replace matched AST nodes with an altered `echo` statement printing a prefixed version of the original.

Patterns written in PHP must parse as valid PHP statements, so a trailing semicolon is necessary, even for simple expressions like `1;`. Variable bindings are only accepted in places where PHP's parser would accept a variable.


Function mapping
----------------

In addition to simple substitution you can map a custom processing function to an AST, doing custom modification whenever a match is found:

```rust
map_fun!(
    "echo PPHP::$_something;", ast, {
        // add arbitrary code to modify matched AST nodes;
        // call `ast.set_dirty(true)` if you modify it
    }
)
```


Interacting with PPHP from PHP
------------------------------

PPHP exposes some functionality to PHP at runtime:

- `pphp_add_rule($searchPattern, $replacePattern)` - defines a new AST substitution rule. Returns `true` if the patterns were successfully parsed. Newly added rules will affect any PHP that is parsed after they're added (e.g. `eval`, `include`), but not anything that was already parsed.

```php
php > echo pphp_add_rule("2 + 2;", "5;");
1
php > echo 2 + 2;
5
```

- `pphp_set_debug_trace($enabled)` - enable or disable a debug tracing mode, which logs all pattern matches to stdout:

```php
php > pphp_set_debug_trace(true);
php > if (true) echo "hi";
** PPHP rule match **
==> matched pattern:
    IF (1 children)
      IF_ELEM (2 children)
        CONST (1 children)
          ZVAL 'true' (0 children)
        <$_1>
==> original AST:
    IF (1 children)
      IF_ELEM (2 children)
        CONST (1 children)
          ZVAL 'true' (0 children)
        ECHO (1 children)
          ZVAL 'hi' (0 children)
==> new AST:
    ECHO (1 children)
      ZVAL 'hi' (0 children)
hi
```


Why Rust?
---------

The real answer is: mostly for fun. While some of Rust's safety guarantees are lost when interoperating closely with C code, it's possible to quarantine any unsafe code into specific structures, creating a "safe" abstraction. Thus if there is any improper memory access from libpphp, it can generally be traced back to a small subset of the code.


TODO
----

- Memory leaking of Zend-allocated values.
- Support constraints on variable bindings.
- Track evaluation context for rules employing static analysis, constant values, types, etc.
    - Constant propagation.
    - Inline functions.
