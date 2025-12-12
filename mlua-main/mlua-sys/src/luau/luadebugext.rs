// Custom debug extensions for Luau
// These functions provide access to function internals for debugging
// Implementation is in mlua-sys/vendor/luau/VM/src/luadebugext.c

use std::os::raw::c_int;
use super::lua_State;

#[cfg(feature = "luau")]
unsafe extern "C-unwind" {
    // Function constant access
    /// Gets a constant from a function's proto at the given index
    /// Returns 1 if successful and pushes the constant on the stack, 0 otherwise
    pub fn luau_getconstant(L: *mut lua_State, funcindex: c_int, n: c_int) -> c_int;
    
    /// Gets the number of constants in a function
    pub fn luau_getconstantcount(L: *mut lua_State, funcindex: c_int) -> c_int;
    
    /// Sets a constant in a function's proto at the given index
    /// Value to set should be on top of stack
    /// Returns 1 if successful, 0 otherwise
    pub fn luau_setconstant(L: *mut lua_State, funcindex: c_int, n: c_int) -> c_int;
    
    // Function proto (nested function) access
    /// Gets a proto from a function at the given index
    /// If activated is true, creates a closure, otherwise pushes the proto as a function
    pub fn luau_getproto(L: *mut lua_State, funcindex: c_int, n: c_int, activated: c_int) -> c_int;
    
    /// Gets the number of protos in a function
    pub fn luau_getprotocount(L: *mut lua_State, funcindex: c_int) -> c_int;
    
    // Stack access at a specific level
    /// Gets a stack value at the given level and index
    /// Returns 1 if successful and pushes the value, 0 otherwise
    pub fn luau_getstack(L: *mut lua_State, level: c_int, n: c_int) -> c_int;
    
    /// Sets a stack value at the given level and index
    /// Value to set should be on top of stack
    pub fn luau_setstack(L: *mut lua_State, level: c_int, n: c_int) -> c_int;
}
