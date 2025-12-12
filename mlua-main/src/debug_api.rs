use std::ffi::CStr;

use crate::function::Function;
use crate::prelude::LuaResult;
use crate::state::Lua;
use crate::table::Table;
use crate::util::StackGuard;
use crate::value::Value;
use crate::ffi;

/// Extension trait for Lua to provide debug capabilities
pub trait LuaDebugExt {
    /// Gets the registry table
    fn debug_get_registry_value(&self) -> LuaResult<Table>;

    /// Gets an upvalue by index
    fn debug_get_upvalue(&self, func: &Function, index: i32) -> LuaResult<(Option<String>, Value)>;

    /// Sets an upvalue by index
    fn debug_set_upvalue(&self, func: &Function, index: i32, value: Value) -> LuaResult<Option<String>>;
    
    /// Gets the metatable of a value, bypassing __metatable check
    fn debug_get_metatable(&self, value: &Value) -> LuaResult<Option<Table>>;

    /// Sets the metatable of a value, bypassing __metatable check
    fn debug_set_metatable(&self, value: &Value, metatable: Option<Table>) -> LuaResult<bool>;
    
    /// Clones a Lua function
    fn clone_function(&self, func: &Function) -> LuaResult<Function>;
    
    /// Checks if a table is readonly
    fn is_table_readonly(&self, table: &Table) -> LuaResult<bool>;
    
    /// Sets whether a table is readonly
    fn set_table_readonly(&self, table: &Table, readonly: bool) -> LuaResult<()>;
}

impl LuaDebugExt for Lua {
    fn debug_get_registry_value(&self) -> LuaResult<Table> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            ffi::lua_pushvalue(state, ffi::LUA_REGISTRYINDEX);
            match lua.pop_value() {
                Value::Table(t) => Ok(t),
                _ => Err(crate::error::Error::RuntimeError("Registry is not a table".into())),
            }
        }
    }

    fn debug_get_upvalue(&self, func: &Function, index: i32) -> LuaResult<(Option<String>, Value)> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);

            lua.push_ref(&func.0);
            let name_ptr = ffi::lua_getupvalue(state, -1, index);
            if name_ptr.is_null() {
                ffi::lua_pop(state, 1);
                return Err(crate::error::Error::RuntimeError("Invalid upvalue index".into()));
            }
            
            let val = lua.pop_value();
            ffi::lua_pop(state, 1);

            let name = if name_ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(name_ptr).to_string_lossy().into_owned())
            };
            
            Ok((name, val))
        }
    }

    fn debug_set_upvalue(&self, func: &Function, index: i32, value: Value) -> LuaResult<Option<String>> {
         unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);

            lua.push_ref(&func.0);
            lua.push_value(&value)?;
            
            let name_ptr = ffi::lua_setupvalue(state, -2, index);
            ffi::lua_pop(state, 1);

            if name_ptr.is_null() {
                 return Err(crate::error::Error::RuntimeError("Invalid upvalue index".into()));
            }

            let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
            Ok(Some(name))
         }
    }
    
    fn debug_get_metatable(&self, value: &Value) -> LuaResult<Option<Table>> {
        unsafe {
             let lua = self.lock();
             let state = lua.state();
             let _sg = StackGuard::new(state);

             lua.push_value(value)?;
             if ffi::lua_getmetatable(state, -1) == 0 {
                 ffi::lua_pop(state, 1);
                 return Ok(None);
             }
             
             let mt = lua.pop_value();
             ffi::lua_pop(state, 1);
             
             match mt {
                 Value::Table(t) => Ok(Some(t)),
                 _ => Ok(None),
             }
        }
    }

    fn debug_set_metatable(&self, value: &Value, metatable: Option<Table>) -> LuaResult<bool> {
        unsafe {
             let lua = self.lock();
             let state = lua.state();
             let _sg = StackGuard::new(state);
             
             lua.push_value(value)?;
             
             if let Some(mt) = metatable {
                 lua.push_ref(&mt.0);
             } else {
                 ffi::lua_pushnil(state);
             }
             
             ffi::lua_setmetatable(state, -2);
             ffi::lua_pop(state, 1);
             
             Ok(true)
        }
    }
    
    fn clone_function(&self, func: &Function) -> LuaResult<Function> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            lua.push_ref(&func.0);
            ffi::lua_clonefunction(state, -1);
            let cloned = lua.pop_ref();
            ffi::lua_pop(state, 1);
            Ok(Function(cloned))
        }
    }
    
    fn is_table_readonly(&self, table: &Table) -> LuaResult<bool> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            lua.push_ref(&table.0);
            let is_readonly = ffi::lua_getreadonly(state, -1) != 0;
            ffi::lua_pop(state, 1);
            Ok(is_readonly)
        }
    }
    
    fn set_table_readonly(&self, table: &Table, readonly: bool) -> LuaResult<()> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let _sg = StackGuard::new(state);
            
            lua.push_ref(&table.0);
            ffi::lua_setreadonly(state, -1, readonly as i32);
            ffi::lua_pop(state, 1);
            Ok(())
        }
    }
}
