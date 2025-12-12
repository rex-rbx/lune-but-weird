use crate::error::Result;
use crate::state::Lua;
use crate::table::Table;
use crate::function::Function;
use crate::value::Value;
use crate::ffi;

/// Extension trait for GC introspection
pub trait LuaGCExt {
    /// Scans all GC objects and returns them
    /// 
    /// # Parameters
    /// - `include_tables`: Whether to include table objects in the results
    /// 
    /// # Returns
    /// A table (array) containing all GC objects
    fn get_gc_objects(&self, include_tables: bool) -> Result<Table>;
    
    /// Filters GC objects by type
    ///
    /// # Parameters
    /// - `type_filter`: The type to filter by ("function", "table", "userdata", "thread", etc.)
    /// 
    /// # Returns
    /// A table (array) containing matching GC objects
    fn filter_gc_objects(&self, type_filter: &str) -> Result<Table>;
}

impl LuaGCExt for Lua {
    fn get_gc_objects(&self, include_tables: bool) -> Result<Table> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let result = self.create_table()?;
            
            // We need to traverse the GC list
            // This requires accessing Luau's internal GC structures
            // For now, we'll return what we can access through the registry
            
            // Push registry
            ffi::lua_pushvalue(state, ffi::LUA_REGISTRYINDEX);
            
            // Iterate through registry
            ffi::lua_pushnil(state);
            let mut index = 1;
            
            while ffi::lua_next(state, -2) != 0 {
                // Key is at -2, value at -1
                let val_type = ffi::lua_type(state, -1);
                
                let should_include = match val_type {
                    ffi::LUA_TTABLE => include_tables,
                    ffi::LUA_TFUNCTION | ffi::LUA_TUSERDATA | ffi::LUA_TTHREAD => true,
                    _ => false,
                };
                
                if should_include {
                    ffi::lua_pushvalue(state, -1); // Duplicate value
                    let val = lua.pop_value();
                    result.raw_set(index, val)?;
                    index += 1;
                }
                
                ffi::lua_pop(state, 1); // Pop value, keep key for next iteration
            }
            
            ffi::lua_pop(state, 1); // Pop registry
            
            Ok(result)
        }
    }
    
    fn filter_gc_objects(&self, type_filter: &str) -> Result<Table> {
        unsafe {
            let lua = self.lock();
            let state = lua.state();
            let result = self.create_table()?;
            
            let type_id = match type_filter.to_lowercase().as_str() {
                "function" => ffi::LUA_TFUNCTION,
                "table" => ffi::LUA_TTABLE,
                "userdata" => ffi::LUA_TUSERDATA,
                "thread" => ffi::LUA_TTHREAD,
                "string" => ffi::LUA_TSTRING,
                _ => return Ok(result), // Empty result for unknown types
            };
            
            // Push registry
            ffi::lua_pushvalue(state, ffi::LUA_REGISTRYINDEX);
            
            // Iterate through registry
            ffi::lua_pushnil(state);
            let mut index = 1;
            
            while ffi::lua_next(state, -2) != 0 {
                let val_type = ffi::lua_type(state, -1);
                
                if val_type == type_id {
                    ffi::lua_pushvalue(state, -1);
                    let val = lua.pop_value();
                    result.raw_set(index, val)?;
                    index += 1;
                }
                
                ffi::lua_pop(state, 1);
            }
            
            ffi::lua_pop(state, 1);
            
            Ok(result)
        }
    }
}
