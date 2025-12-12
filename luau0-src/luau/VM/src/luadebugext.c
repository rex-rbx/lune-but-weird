// luadebugext.c
// Custom debug extensions for Luau that expose function internals
// These functions access Luau's internal Proto structures to provide
// debugging capabilities similar to Roblox script debuggers

#include "lualib.h"
#include "lstate.h"
#include "lfunc.h"
#include "lobject.h"
#include "ldo.h"

// Get a constant from a function at the given index
// Returns 1 if successful and pushes the constant, 0 otherwise  
LUA_API int luau_getconstant(lua_State* L, int funcindex, int n)
{
    const TValue* f = luaA_toobject(L, funcindex);
    if (!ttisfunction(f))
        return 0;
    
    Closure* cl = clvalue(f);
    if (!cl->isC)
    {
        Proto* p = cl->l.p;
        if (n >= 0 && n < p->sizek)
        {
            setobj2s(L, L->top, &p->k[n]);
            api_incr_top(L);
            return 1;
        }
    }
    return 0;
}

// Get the number of constants in a function
LUA_API int luau_getconstantcount(lua_State* L, int funcindex)
{
    const TValue* f = luaA_toobject(L, funcindex);
    if (!ttisfunction(f))
        return 0;
    
    Closure* cl = clvalue(f);
    if (!cl->isC)
        return cl->l.p->sizek;
    
    return 0;
}

// Set a constant in a function (value should be on top of stack)
LUA_API int luau_setconstant(lua_State* L, int funcindex, int n)
{
    const TValue* f = luaA_toobject(L, funcindex);
    if (!ttisfunction(f) || api_gettop(L) < 1)
        return 0;
    
    Closure* cl = clvalue(f);
    if (!cl->isC)
    {
        Proto* p = cl->l.p;
        if (n >= 0 && n < p->sizek)
        {
            setobj(L, &p->k[n], L->top - 1);
            L->top--;
            return 1;
        }
    }
    L->top--;
    return 0;
}

// Get a proto (nested function) from a function
// If activated != 0, creates a closure, otherwise pushes raw function
LUA_API int luau_getproto(lua_State* L, int funcindex, int n, int activated)
{
    const TValue* f = luaA_toobject(L, funcindex);
    if (!ttisfunction(f))
        return 0;
    
    Closure* cl = clvalue(f);
    if (!cl->isC)
    {
        Proto* p = cl->l.p;
        if (n >= 0 && n < p->sizep)
        {
            Proto* np = p->p[n];
            if (activated)
            {
                // Create a closure
                Closure* ncl = luaF_newLclosure(L, np->nups, cl->env, np);
                setclvalue(L, L->top, ncl);
            }
            else
            {
                // Push as a new closure anyway (can't push raw Proto)
                Closure* ncl = luaF_newLclosure(L, np->nups, cl->env, np);
                setclvalue(L, L->top, ncl);
            }
            api_incr_top(L);
            return 1;
        }
    }
    return 0;
}

// Get the number of protos in a function
LUA_API int luau_getprotocount(lua_State* L, int funcindex)
{
    const TValue* f = luaA_toobject(L, funcindex);
    if (!ttisfunction(f))
        return 0;
    
    Closure* cl = clvalue(f);
    if (!cl->isC)
        return cl->l.p->sizep;
    
    return 0;
}

// Get a stack value at a specific level and index
LUA_API int luau_getstack(lua_State* L, int level, int n)
{
    CallInfo* ci = L->ci;
    
    // Find the call frame at the specified level
    for (int i = 0; ci && i < level; i++)
    {
        ci = ci->previous;
        if (!ci) return 0;
    }
    
    if (!ci) return 0;
    
    // Get the nth stack value
    if (n >= 0 && ci->base + n < ci->top)
    {
        setobj2s(L, L->top, ci->base + n);
        api_incr_top(L);
        return 1;
    }
    
    return 0;
}

// Set a stack value at a specific level and index (value on top of stack)
LUA_API int luau_setstack(lua_State* L, int level, int n)
{
    if (api_gettop(L) < 1)
        return 0;
        
    CallInfo* ci = L->ci;
    
    // Find the call frame
    for (int i = 0; ci && i < level; i++)
    {
        ci = ci->previous;
        if (!ci) return 0;
    }
    
    if (!ci) return 0;
    
    // Set the stack value
    if (n >= 0 && ci->base + n < ci->top)
    {
        setobj(L, ci->base + n, L->top - 1);
        L->top--;
        return 1;
    }
    
    L->top--;
    return 0;
}
