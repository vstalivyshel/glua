use crate::utils::*;
use crate::FromClient;
pub use mlua::Lua;
use mlua::{FromLua, MultiValue, Result, Table, ToLua};

const KAK: &str = "kak";
const SES: &str = "session";
const CLIENT: &str = "client";
const ROOT: &str = "root_dir";

pub trait LuaServer {
    fn prelude(&self, root: &str) -> Result<()>;
    fn session_data(&self) -> Result<Table>;
    fn set_data<A: for<'a> ToLua<'a>>(&self, field: &str, value: A) -> Result<()>;
    fn get_data<A: for<'a> FromLua<'a>>(&self, field: &str) -> Result<A>;
    fn call_chunk(&self, data: FromClient) -> Result<Vec<String>>;
    fn kak_eval(&self, cmd: &String) -> Result<()>;
}

impl LuaServer for Lua {
    fn prelude(&self, root: &str) -> Result<()> {
        let globals = self.globals();
        let kak = self.create_table()?;
        kak.set(ROOT, root.to_string())?;

        kak.set(
            "send_to",
            self.create_function(|_, (ses, cmd): (String, String)| {
                kak_send_msg(&ses, &cmd)?;
                Ok(())
            })?,
        )?;

        kak.set(
            "eval",
            self.create_function(|lua, cmd: String| lua.kak_eval(&cmd))?,
        )?;

        globals.set(KAK, kak)?;

        Ok(())
    }

    fn session_data(&self) -> Result<Table> {
        self.globals().get::<_, Table>(KAK)
    }

    fn set_data<A: for<'a> ToLua<'a>>(&self, field: &str, value: A) -> Result<()> {
        self.session_data()?.set(field, value)
    }

    fn get_data<A: for<'a> FromLua<'a>>(&self, field: &str) -> Result<A> {
        self.session_data()?.get::<_, A>(field)
    }

    fn call_chunk(&self, data: FromClient) -> Result<Vec<String>> {
        self.set_data::<String>(SES, data.session)?;
        self.set_data::<String>(CLIENT, data.client)?;
        self.globals().set::<_, Vec<String>>("arg", data.chunk_args)?;
        let vals: MultiValue = self
            .load(&data.chunk)
            .eval()?;
        let mut result = Vec::<String>::new();
        // TODO: how to deal with functions and tables in return values?
        for val in vals.into_iter() {
            if let Ok(v) = String::from_lua(val, self) {
                result.push(v);
            } else {
                result.push("Unconverable".to_string());
            }
        }

        Ok(result)
    }

    fn kak_eval(&self, cmd: &String) -> Result<()> {
        let cur_client = self.get_data::<String>(CLIENT)?;
        let cur_ses = self.get_data::<String>(SES)?;
        kak_send_client(&cur_ses, &cur_client, cmd)?;

        Ok(())
    }
}

