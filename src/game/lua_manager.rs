use std::{collections::HashMap, ffi::OsStr, sync::Arc};

use mlua::Lua;
use tokio::sync::RwLock;

use crate::utils::errors::Error;

pub struct LuaManager {
    pub lua: Arc<Lua>,
    pub functions: Arc<RwLock<HashMap<String, mlua::Function>>>,
}

impl LuaManager {
    pub async fn new() -> Result<Self, Error> {
        let me = Self {
            lua: Arc::new(Lua::new()),
            functions: Arc::new(RwLock::new(HashMap::default())),
        };

        me.load_scripts()?;
        me.preload_functions().await;

        return Ok(me);
    }

    pub fn load_scripts(&self) -> Result<(), Error> {
        let scripts = std::fs::read_dir("./scripts").map_err(|_| Error::ScriptingFailed(0))?;
        for entry in scripts {
            let entry = entry.map_err(|_| Error::ScriptingFailed(0))?.path();
            if entry.extension() == Some(OsStr::new("lua")) {
                let chunk =
                    std::fs::read_to_string(&entry).map_err(|_| Error::ScriptingFailed(0))?;
                self.lua
                    .load(&chunk)
                    .exec()
                    .map_err(|_| Error::ScriptingFailed(0))?;
            }
        }

        Ok(())
    }

    pub async fn preload_functions(&self) {
        let globals = self.lua.globals();
        if let Ok(function) = globals.get::<mlua::Function>("check_calls") {
            let mut guard = self.functions.write().await;
            guard.insert("check_calls".to_string(), function);
        }
    }

    pub async fn check_calls(&self, arg: mlua::Table) -> Result<mlua::Table, Error> {
        let guard = self.functions.read().await;
        let function = guard.get("check_calls").ok_or(Error::InternalError)?;
        let response: mlua::Table = function.call(arg).map_err(|_| Error::InternalError)?;
        return Ok(response);
    }

    pub fn vec_to_luatable(&self, v: &Vec<i8>) -> Result<mlua::Table, Error> {
        let tbl = self.lua.create_table().map_err(|_| Error::InternalError)?;
        for (i, val) in v.iter().enumerate() {
            tbl.set(i + 1, *val).map_err(|_| Error::InternalError)?; // Lua is 1-indexed
        }
        Ok(tbl)
    }
}
