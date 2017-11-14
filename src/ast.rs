use ::std::boxed::Box;
use ::std::collections::HashMap;
use ::std::ffi::CString;
use ::std::ops::Deref;
use ::std::os::raw::c_char;
use ::std::sync::Mutex;
use php;

pub struct Options {
    pub debug_trace: bool,
}

impl Options {
    pub fn new() -> Self {
        Options {
            debug_trace: false,
        }
    }
}

lazy_static! {
    pub static ref OPTIONS: Mutex<Options> = {
        Mutex::new(Options::new())
    };
}

pub fn set_debug_trace(enabled: bool) {
    let mut options = OPTIONS.lock().unwrap();
    options.debug_trace = enabled;
}

/**
 * Recursively walk an AST, calling a function on each child node. Zero or more
 * additional arguments can be specified which will be passed to $callable
 * after the pointer to the current node.
 */
macro_rules! ast_walk {
    ($ast:expr, $callable:expr) => {
        $crate::ast::node_walk($ast, &mut |node_ptr: &mut $crate::ast::ZendAstPtr| {
            $callable(node_ptr);
        })
    };

    ($ast:expr, $callable:expr, $($args:expr),*) => {
        $crate::ast::node_walk($ast, &mut |node_ptr: &mut $crate::ast::ZendAstPtr| {
            $callable(node_ptr, $($args),*);
        })
    };
}

pub type ZendAst = *mut php::zend_ast;
pub type ZendAstList = *mut php::zend_ast_list;
pub type ZendAstDecl = *mut php::zend_ast_decl;

#[derive(Clone, Debug)]
pub struct ZendAstPtr {
    ptr: *mut ZendAst,
    dirty: bool,
}
impl ZendAstPtr {
    pub fn new(ptr: *mut ZendAst) -> ZendAstPtr {
        ZendAstPtr {
            ptr: ptr,
            dirty: false,
        }
    }

    pub fn deref(&self) -> ZendAst {
        unsafe {
            *(self.ptr)
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /**
     * Replace the AST node pointed to by `old` with `new`.
     */
    pub fn replace(&mut self, new: ZendAst) {
        self.dirty = true;
        unsafe {
            ::std::ptr::replace(self.ptr, new);
        };
    }

    pub fn bind_sub_replace(&mut self, bindings: &Bindings) {
        match is_bind_param(unsafe {*(self.ptr)}) {
            Some(var) => {
                self.replace(bindings.get(&var).unwrap().clone());
            }
            _ => {}
        }
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
}
impl From<ZendAstPtr> for ZendAst {
    fn from(ptr: ZendAstPtr) -> Self {
        ptr.deref()
    }
}

pub type Bindings = HashMap<String, ZendAst>;

extern "C" {
    fn get_ast(code: *const c_char) -> ZendAst;
    fn ast_is_list(zast: ZendAst) -> bool;
    fn ast_is_decl(zast: ZendAst) -> bool;
    fn ast_num_children(zast: ZendAst) -> u32;
    fn ast_size(zast: ZendAst) -> u32;
    fn ast_zval(zast: ZendAst) -> *mut php::zval;
    fn zval_string(zval: *mut php::zval) -> *mut php::zend_string;
    fn var_export(zval: *mut php::zval) -> *mut php::zend_string;
}

/**
 * Parse a string of PHP code into a ZendAst struct. The string should begin
 * with a <?php tag; otherwise it'll parse as an echo statement.
 *
 * Returns None if the parse failed.
 */
pub fn parse(code: &str) -> Option<ZendAst> {
    let cstr = CString::new(code).unwrap();
    unsafe {
        let cp = cstr.as_ptr();
        let ast = get_ast(cp);
        if ast.is_null() {
            None
        } else {
            Some(ast)
        }
    }
}

/**
 * A convenience function for internal PHP which is known to parse
 * successfully. Adds the opening <?php tag automatically.
 */
pub fn parse_pattern(code: &str) -> Option<ZendAst> {
    let node = parse(&("<?php ".to_string() + code));
    node
}

/**
 * Given an AST node, returns a pointer to a pointer for each of the node's
 * children. These pointers can be modified to modify the AST in place.
 */
pub fn get_children(zast: ZendAst) -> Vec<ZendAstPtr> {
    let mut children = Vec::new();
    let mut child: *mut ZendAst;
    let mut count: u32 = 0;
    unsafe {
        if ast_is_decl(zast) {
            let decl = zast as ZendAstDecl;
            count = if (*decl).kind == (php::_zend_ast_kind::ZEND_AST_CLASS as u16) {
                3_u32
            } else {
                4_u32
            };
            child = &mut ((*decl).child[0]) as *mut ZendAst;
        } else if ast_is_list(zast) {
            let list = zast as ZendAstList;
            count = (*list).children;
            child = &mut ((*list).child[0]) as *mut ZendAst;
        } else {
            count = ast_num_children(zast);
            child = &mut ((*zast).child[0]) as *mut ZendAst;
        }
        for i in 0 .. count {
            let child_ptr = child.offset(i as isize);
            if !(*child_ptr).is_null() {
                children.push(ZendAstPtr::new(child_ptr));
            }
        }
    }
    children
}

/**
 * Return an owned String from a zend_string.
 */
fn zend_str_val<'a>(mut zstr: php::zend_string) -> String {
    let len = zstr.len;
    let char_ptr = &mut (zstr.val[0]) as *mut _;
    unsafe {
        let cstr = ::std::ffi::CStr::from_ptr(char_ptr);
        cstr.to_str().unwrap().to_string()
    }
}

/**
 * If this AST node matches the form PPHP::$variableName, returns
 * Some(variableName); otherwise returns None.
 *
 * TODO: implement constraints
 */
pub fn is_bind_param(zast: ZendAst) -> Option<String> {
    let deref: php::zend_ast = unsafe { *zast };
    if deref.kind == php::_zend_ast_kind::ZEND_AST_STATIC_PROP as u16 {
        let children = get_children(zast);
        if children.len() != 2 {
            return None;
        }
        let clsNode = &children[0];
        let varNameNode = &children[1];
        unsafe {
            if (*clsNode.deref()).kind != php::_zend_ast_kind::ZEND_AST_ZVAL as u16 {
                return None;
            }
            if (*varNameNode.deref()).kind != php::_zend_ast_kind::ZEND_AST_ZVAL as u16 {
                return None;
            }
            let cls = zval_string(ast_zval(clsNode.deref()));
            let varName = zval_string(ast_zval(varNameNode.deref()));
            if zend_str_val(*cls) == "PPHP" {
                // found it!
                return Some(zend_str_val(*varName));
            }
        }
    }
    None
}

/**
 * Recursively walk an AST, calling the provided callback on every child.
 *
 * This is generally not called directly; use ast_walk! for a variadic
 * interface.
 */
pub fn node_walk<F>(zast: ZendAst, f: &mut F) where F: FnMut(&mut ZendAstPtr) -> () {
    // FIXME: if a node was modified, you should stop and re-call get_children?
    for mut child in get_children(zast) {
        f(&mut child);
        unsafe {
            node_walk(ZendAst::from(child), f);
        }
    }
}

/**
 * Unwraps a ZEND_AST_STMT_LIST containing only a single AST node, returning
 * the inner node.
 */
pub fn unwrap(zast: ZendAst) -> Option<ZendAst> {
    if (unsafe {*zast}).kind == php::_zend_ast_kind::ZEND_AST_STMT_LIST as u16 {
        let l = unsafe {
            (*(zast as ZendAstList))
        };
        if l.children == 1 {
            Some(l.child[0])
        } else {
            None
        }
    }
    else {
        None
    }
}

pub fn unwrap_all(zast: ZendAst) -> ZendAst {
    let mut current: ZendAst = zast;
    loop {
        match unwrap(current) {
            Some(z) => {
                current = z;
            }
            None => {
                break;
            }
        }
    }
    current
}

/**
 * Returns true if `zast` matches the provided `pattern`. If variable names are
 * encountered, the corresponding AST node will be bound to the variable.
 */
pub fn pattern_match(mut pattern: ZendAst, mut zast: ZendAst, mut bindings: &mut Bindings) -> bool {
    // unwrap statement lists with exactly 1 child
    pattern = unwrap_all(pattern);
    zast = unwrap_all(zast);

    match is_bind_param(pattern) {
        Some(var) => {
            if bindings.contains_key(&var) {
                return pattern_match(bindings.get(&var).unwrap().clone(), zast, &mut bindings);
            } else {
                bindings.insert(var, zast);
                return true;
            }
        }
        None => unsafe {
            if (*pattern).kind == (*zast).kind {
                // TODO: should attr mismatch matter for all kinds?
                if (*pattern).attr != (*zast).attr {
                    return false;
                }
                // TODO: decl name binding?
                // node-specific checks
                if (*pattern).kind == php::_zend_ast_kind::ZEND_AST_ZVAL as u16 {
                    let identical = php::zend_is_identical(ast_zval(pattern), ast_zval(zast));
                    if identical == 0 {
                        return false;
                    }
                }
                let children1 = get_children(pattern);
                let children2 = get_children(zast);
                if children1.len() != children2.len() {
                    return false;
                }
                for i in 0 .. children1.len() {
                    if !pattern_match(children1[i].deref(), children2[i].deref(), &mut bindings) {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
    }
    true
}

/**
 * Replace all instances of PPHP::$variable nodes with the bound pattern.
 */
pub fn bind_sub(ast: ZendAst, bindings: &Bindings) {
    ast_walk!(ast, |node_ptr: &mut ZendAstPtr| {
        node_ptr.bind_sub_replace(&bindings);
    });
}

pub fn print_node(zast: ZendAst, indentation: usize) {
    match unwrap(zast) {
        Some(inner) => {
            return print_node(inner, indentation);
        }
        None => {}
    }
    let kind: php::_zend_ast_kind = unsafe {
        ::std::mem::transmute((*zast).kind as u32)
    };
    match is_bind_param(zast) {
        Some(var) => {
            println!("{:width$}<${}>", " ", var, width=indentation * 2);
        }
        None => {
            let children = get_children(zast);
            if kind == php::_zend_ast_kind::ZEND_AST_ZVAL {
                let zstr = unsafe {
                    *var_export(ast_zval(zast))
                };
                let dump = zend_str_val(zstr);
                println!("{:width$}ZVAL {} ({} children)", " ", dump, children.len(), width=indentation * 2);
            } else {
                let name = unsafe {
                    if ast_is_decl(zast) {
                        let decl = *(zast as ZendAstDecl);
                        format!("{} ", zend_str_val(*decl.name))
                    } else {
                        format!("")
                    }
                };
                let attr = unsafe {
                    let a = (*zast).attr;
                    if a == 0 {
                        format!("")
                    } else {
                        format!("0x{:x} ", a)
                    }
                };
                let kind_name = {
                    let debug_fmt = format!("{:?}", kind);
                    if &debug_fmt[..9] == "ZEND_AST_" {
                        (&(format!("{:?}", kind))[9..]).to_string()
                    } else {
                        debug_fmt
                    }
                };
                println!("{:width$}{} {}{}({} children)", " ", kind_name, name, attr, children.len(), width=indentation * 2);
            }
            for child in children {
                print_node(child.deref(), indentation + 1);
            }
        }
    }
}

/**
 * Map an AST substitution rule over an AST tree, replacing any instances in
 * $ast which match.
 *
 * The following special patterns are supported:
 *
 * Static variables in the form PPHP::$myVariable will bind to any expression
 * and can be referenced in the replacement pattern.
 */
#[macro_export]
macro_rules! map_sub {
    ($patternSearch:expr, $patternReplace:expr, $ast:expr) => {
        let mut bindings = ast::Bindings::new();
        map_fun!(__impl, bindings, $patternSearch, $ast, {
            let replace = ast::parse_pattern($patternReplace).unwrap();
            ast::bind_sub(replace, &bindings);
            $ast.replace($crate::ast::unwrap_all(replace));
        });
    };
}

/**
 * Like map_sub, but instead of a simple substitution, calls the provided
 * function on a match. The function should modify the AST in place and return
 * true if it was modified and false otherwise.
 *
 * $searchPattern can contain variables to be bound, as in map_sub.
 */
#[macro_export]
macro_rules! map_fun {
    ($patternSearch:expr, $ast:expr, $fun:tt) => {
        {
            let mut bindings = ast::Bindings::new();
            map_fun!(__impl, bindings, $patternSearch, $ast, $fun);
        }
    };

    (__impl, $bindings:ident, $patternSearch:expr, $ast:expr, $fun:tt) => {
        {
            let pattern = ast::parse_pattern($patternSearch).unwrap();
            if ast::pattern_match(pattern, $ast.deref(), &mut $bindings) {
                let options = ast::OPTIONS.lock().unwrap();
                if options.debug_trace {
                    println!("** PPHP rule match **");
                    println!("==> matched pattern:");
                    $crate::ast::print_node(pattern, 2);
                    println!("==> original AST:");
                    $crate::ast::print_node($ast.deref(), 2);
                }
                $fun
                if options.debug_trace {
                    println!("==> new AST:");
                    $crate::ast::print_node($ast.deref(), 2);
                }
            }
        }
    };
}
