use crate::error::Result;
use crate::function::Function;
use crate::state::Lua;
use crate::table::Table;
use crate::util::StackGuard;
use crate::value::Value;
use crate::ffi;

/// Extension trait for function introspection (constants, protos, etc.)
pub trait LuaFunctionExt {
    /// Gets all constants from a function
    fn get_function_constants(&self, func: &Function) -> Result<Table>;
    
    /// Gets a specific constant from a function by index
    fn get_function_constant(&self, func: &Function, index: usize) -> Result<Value>;
    
    /// Sets a specific constant in a function by index
    fn set_function_constant(&self, func: &Function, index: usize, value: Value) -> Result<()>;
    
    /// Gets all prototypes (nested functions) from a function
    fn get_function_protos(&self, func: &Function) -> Result<Table>;
    
    /// Gets a specific prototype from a function by index
    fn get_function_proto(&self, func: &Function, index: usize, activated: bool) -> Result<Option<Function>>;
    
    /// Gets a stack value at a specific level and index
    fn get_stack_value(&self, level: usize, index: usize) -> Result<Value>;
    
    /// Sets a stack value at a specific level and index
    fn set_stack_value(&self, level: usize, index: usize, value: Value) -> Result<()>;
}

impl LuaFunctionExt for Lua {
    fn get_function_constants(&self, func: &Function) -> Result<Table> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            let result = self.create_table()?;
            lua.push_ref(&func.0);
            
            let count = ffi::luau_getconstantcount(state, -1);
            for i in 0..count {
                if ffi::luau_getconstant(state, -2, i) != 0 {
                    let val = lua.pop_value();
                    result.raw_set(i + 1, val)?;
                }
            }
            
            ffi::lua_pop(state, 1); // pop function
            Ok(result)
        }
    }
    
    fn get_function_constant(&self, func: &Function, index: usize) -> Result<Value> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            lua.push_ref(&func.0);
            
            if ffi::luau_getconstant(state, -1, index as i32) != 0 {
                let val = lua.pop_value();
                ffi::lua_pop(state, 1); // pop function
                Ok(val)
            } else {
                ffi::lua_pop(state, 1); // pop function
                Ok(Value::Nil)
            }
        }
    }
    
    fn set_function_constant(&self, func: &Function, index: usize, value: Value) -> Result<()> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            lua.push_ref(&func.0);
            lua.push_value(&value)?;
            
            ffi::luau_setconstant(state, -2, index as i32);
            ffi::lua_pop(state, 1); // pop function
            
            Ok(())
        }
    }
    
    fn get_function_protos(&self, func: &Function) -> Result<Table> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            let result = self.create_table()?;
            lua.push_ref(&func.0);
            
            let count = ffi::luau_getprotocount(state, -1);
            for i in 0..count {
                if ffi::luau_getproto(state, -2, i, 0) != 0 {
                    let proto_ref = lua.pop_ref();
                    result.raw_set(i + 1, Function(proto_ref))?;
                }
            }
            
            ffi::lua_pop(state, 1); // pop function
            Ok(result)
        }
    }
    
    fn get_function_proto(&self, func: &Function, index: usize, activated: bool) -> Result<Option<Function>> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            lua.push_ref(&func.0);
            
            if ffi::luau_getproto(state, -1, index as i32, activated as i32) != 0 {
                let proto_ref = lua.pop_ref();
                ffi::lua_pop(state, 1); // pop function
                Ok(Some(Function(proto_ref)))
            } else {
                ffi::lua_pop(state, 1); // pop function
                Ok(None)
            }
        }
    }
    
    fn get_stack_value(&self, level: usize, index: usize) -> Result<Value> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            if ffi::luau_getstack(state, level as i32, index as i32) != 0 {
                Ok(lua.pop_value())
            } else {
                Ok(Value::Nil)
            }
        }
    }
    
    fn set_stack_value(&self, level: usize, index: usize, value: Value) -> Result<()> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            lua.push_value(&value)?;
            ffi::luau_setstack(state, level as i32, index as i32);
            
            Ok(())
        }
    }
}
