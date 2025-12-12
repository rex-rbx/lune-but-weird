use mlua::prelude::*;

use crate::instance::Instance;

pub fn create_globals(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaValue)>> {
    let globals = vec![
        ("getgc", create_getgc(lua)?.into_lua(lua)?),
        ("filtergc", create_filtergc(lua)?.into_lua(lua)?),
        ("getreg", create_getreg(lua)?.into_lua(lua)?),
        ("getinstances", create_getinstances(lua)?.into_lua(lua)?),
        ("getnilinstances", create_getnilinstances(lua)?.into_lua(lua)?),
        ("hookfunction", create_hookfunction(lua)?.into_lua(lua)?),
        ("hookmetamethod", create_hookmetamethod(lua)?.into_lua(lua)?),
        ("checkcaller", create_checkcaller(lua)?.into_lua(lua)?),
        ("clonefunction", create_clonefunction(lua)?.into_lua(lua)?),
        ("islclosure", create_islclosure(lua)?.into_lua(lua)?),
        ("iscclosure", create_iscclosure(lua)?.into_lua(lua)?),
        ("newcclosure", create_newcclosure(lua)?.into_lua(lua)?),
        ("restorefunction", create_restorefunction(lua)?.into_lua(lua)?),
        ("cloneref", create_cloneref(lua)?.into_lua(lua)?),
        ("compareinstances", create_compareinstances(lua)?.into_lua(lua)?),
        ("getnamecallmethod", create_getnamecallmethod(lua)?.into_lua(lua)?),
        ("getrawmetatable", create_getrawmetatable(lua)?.into_lua(lua)?),
        ("isreadonly", create_isreadonly(lua)?.into_lua(lua)?),
        ("setrawmetatable", create_setrawmetatable(lua)?.into_lua(lua)?),
        ("setreadonly", create_setreadonly(lua)?.into_lua(lua)?),
    ];

    // Get or create debug table and add our extensions
    let debug_table: LuaTable = lua.globals().get("debug")
        .unwrap_or_else(|_| lua.create_table().unwrap());
    
    // Make the debug table writable temporarily
    use mlua::debug_api::LuaDebugExt;
    let was_readonly = lua.is_table_readonly(&debug_table).unwrap_or(false);
    if was_readonly {
        lua.set_table_readonly(&debug_table, false).ok();
    }
    
    debug_table.set("getconstant", create_debug_getconstant(lua)?)?;
    debug_table.set("getconstants", create_debug_getconstants(lua)?)?;
    debug_table.set("getproto", create_debug_getproto(lua)?)?;
    debug_table.set("getprotos", create_debug_getprotos(lua)?)?;
    debug_table.set("getstack", create_debug_getstack(lua)?)?;
    debug_table.set("getupvalue", create_debug_getupvalue(lua)?)?;
    debug_table.set("getupvalues", create_debug_getupvalues(lua)?)?;
    debug_table.set("setconstant", create_debug_setconstant(lua)?)?;
    debug_table.set("setstack", create_debug_setstack(lua)?)?;
    debug_table.set("setupvalue", create_debug_setupvalue(lua)?)?;
    
    // Restore readonly status
    if was_readonly {
        lua.set_table_readonly(&debug_table, true).ok();
    }

    let mut result = globals;
    result.push(("debug", debug_table.into_lua(lua)?));

    Ok(result)
}

fn create_getgc(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, include_tables: Option<bool>| {
        use mlua::gc_api::LuaGCExt;
        lua.get_gc_objects(include_tables.unwrap_or(false))
    })
}

fn create_filtergc(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, (type_filter, _options, _return_one): (String, Option<LuaTable>, Option<bool>)| {
        use mlua::gc_api::LuaGCExt;
        lua.filter_gc_objects(&type_filter)
    })
}

fn create_getreg(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, ()| {
        use mlua::debug_api::LuaDebugExt;
        lua.debug_get_registry_value()
    })
}

fn create_getinstances(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, ()| {
        let instances = Instance::get_all_instances();
        lua.create_table_from(instances.into_iter().enumerate().map(|(i, v)| (i + 1, v)))
    })
}

fn create_getnilinstances(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, ()| {
        let instances = Instance::get_nil_instances();
        lua.create_table_from(instances.into_iter().enumerate().map(|(i, v)| (i + 1, v)))
    })
}

fn create_hookfunction(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, (target, hook): (LuaValue, LuaFunction)| {
        use mlua::debug_api::LuaDebugExt;
        
        // Get the original function
        let target_func = match target {
            LuaValue::Function(f) => f,
            _ => return Err(LuaError::RuntimeError("First argument must be a function".into())),
        };
        
        // Clone the original function to return it
        let original = lua.clone_function(&target_func)?;
        
        // Store the hook mapping in registry
        let registry = lua.named_registry_value::<LuaTable>("__FUNCTION_HOOKS")
            .unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                lua.set_named_registry_value("__FUNCTION_HOOKS", t.clone()).unwrap();
                t
            });
        
        // Store original -> hook mapping
        registry.set(lua.create_registry_value(target_func.clone())?, hook)?;
        
        Ok(original)
    })
}

fn create_hookmetamethod(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, (obj, method, hook): (LuaValue, String, LuaFunction)| {
        use mlua::debug_api::LuaDebugExt;
        
        // Get the metatable
        let mt = lua.debug_get_metatable(&obj)?;
        let mt = match mt {
            Some(table) => table,
            None => return Err(LuaError::RuntimeError("Object has no metatable".into())),
        };
        
        // Get the original metamethod
        let original: LuaValue = mt.get(method.as_str())?;
        
        // Set the hook as the new metamethod
        mt.set(method, hook)?;
        
        // Return the original
        Ok(original)
    })
}

fn create_checkcaller(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| {
        Ok(true) // Always true for now as we are essentially the executor
    })
}

fn create_clonefunction(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, f: LuaFunction| {
        use mlua::debug_api::LuaDebugExt;
        lua.clone_function(&f)
    })
}

fn create_islclosure(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, f: LuaFunction| {
        let info = f.info();
        Ok(info.what == "Lua")
    })
}

fn create_iscclosure(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, f: LuaFunction| {
        let info = f.info();
        Ok(info.what == "C")
    })
}

fn create_newcclosure(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, f: LuaFunction| {
        // Wrapping a Lua function in a C closure is standard behavior for create_function but 
        // passing a Lua function to it? 
        // We essentially want to return a Rust function that calls the Lua function.
        Ok(f) // TODO: Wrap in native callback
    })
}

fn create_restorefunction(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, _f: LuaFunction| {
        Ok(())
    })
}

fn create_cloneref(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, o: LuaValue| {
        // TODO: Implement safe reference cloning
        Ok(o)
    })
}

fn create_compareinstances(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, (a, b): (LuaValue, LuaValue)| {
        match (a.as_userdata(), b.as_userdata()) {
            (Some(a_ud), Some(b_ud)) => {
                let a_inst = a_ud.borrow::<Instance>()?;
                let b_inst = b_ud.borrow::<Instance>()?;
                Ok(a_inst.dom_ref == b_inst.dom_ref)
            }
            _ => Ok(false)
        }
    })
}

fn create_getnamecallmethod(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|_, ()| {
        Ok(LuaValue::Nil) // TODO
    })
}

fn create_getrawmetatable(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, o: LuaValue| {
        use mlua::debug_api::LuaDebugExt;
        lua.debug_get_metatable(&o)
    })
}

fn create_isreadonly(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, t: LuaTable| {
        use mlua::debug_api::LuaDebugExt;
        lua.is_table_readonly(&t)
    })
}

fn create_setrawmetatable(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, (obj, mt): (LuaValue, Option<LuaTable>)| {
         use mlua::debug_api::LuaDebugExt;
         lua.debug_set_metatable(&obj, mt)?;
         Ok(obj)
    })
}

fn create_setreadonly(lua: &Lua) -> LuaResult<LuaFunction> {
    lua.create_function(|lua, (t, readonly): (LuaTable, bool)| {
        use mlua::debug_api::LuaDebugExt;
        lua.set_table_readonly(&t, readonly)
    })
}

// Debug functions placeholders
fn create_debug_getconstant(lua: &Lua) -> LuaResult<LuaFunction> {
     lua.create_function(|lua, (f, idx): (LuaFunction, usize)| {
         use mlua::function_api::LuaFunctionExt;
         lua.get_function_constant(&f, idx)
     })
}
fn create_debug_getconstants(lua: &Lua) -> LuaResult<LuaFunction> {
     lua.create_function(|lua, f: LuaFunction| {
         use mlua::function_api::LuaFunctionExt;
         lua.get_function_constants(&f)
     })
}
fn create_debug_getproto(lua: &Lua) -> LuaResult<LuaFunction> {
     lua.create_function(|lua, (f, idx, act): (LuaFunction, usize, Option<bool>)| {
         use mlua::function_api::LuaFunctionExt;
         lua.get_function_proto(&f, idx, act.unwrap_or(false))
     })
}
fn create_debug_getprotos(lua: &Lua) -> LuaResult<LuaFunction> {
     lua.create_function(|lua, f: LuaFunction| {
         use mlua::function_api::LuaFunctionExt;
         lua.get_function_protos(&f)
     })
}
fn create_debug_getstack(lua: &Lua) -> LuaResult<LuaFunction> {
     lua.create_function(|lua, (lvl, idx): (usize, Option<usize>)| {
         use mlua::function_api::LuaFunctionExt;
         lua.get_stack_value(lvl, idx.unwrap_or(1))
     })
}
fn create_debug_getupvalue(lua: &Lua) -> LuaResult<LuaFunction> {
     lua.create_function(|lua, (f, idx): (LuaFunction, i32)| { 
         use mlua::debug_api::LuaDebugExt;
         let (name, val) = lua.debug_get_upvalue(&f, idx)?;
         Ok((name, val))
     })
}
fn create_debug_getupvalues(lua: &Lua) -> LuaResult<LuaFunction> {
     lua.create_function(|lua, f: LuaFunction| {
         use mlua::debug_api::LuaDebugExt;
         let result = lua.create_table()?;
         let mut idx = 1i32;
         loop {
             match lua.debug_get_upvalue(&f, idx) {
                 Ok((name, val)) => {
                     if let Some(n) = name {
                         result.set(n, val)?;
                     }
                     idx += 1;
                 }
                 Err(_) => break,
             }
         }
         Ok(result)
     })
}
fn create_debug_setconstant(lua: &Lua) -> LuaResult<LuaFunction> {
     lua.create_function(|lua, (f, idx, val): (LuaFunction, usize, LuaValue)| {
         use mlua::function_api::LuaFunctionExt;
         lua.set_function_constant(&f, idx, val)
     })
}
fn create_debug_setstack(lua: &Lua) -> LuaResult<LuaFunction> {
     lua.create_function(|lua, (lvl, idx, val): (usize, usize, LuaValue)| {
         use mlua::function_api::LuaFunctionExt;
         lua.set_stack_value(lvl, idx, val)
     })
}
fn create_debug_setupvalue(lua: &Lua) -> LuaResult<LuaFunction> {
     lua.create_function(|lua, (f, idx, val): (LuaFunction, i32, LuaValue)| { 
         use mlua::debug_api::LuaDebugExt;
         let name = lua.debug_set_upvalue(&f, idx, val)?;
         Ok(name)
     })
}
