use mlua::{FromLua, Lua, MultiValue, Result};
use std::{
    io::{self, Write},
    process::{Command, Stdio},
    ffi::CString,
};

extern "C" {
    fn mkfifo(path: *const i8,  mode: u32) -> i32;
}

fn create_fifo(path: &str, mode: u32) -> Result<()> {
    let path = CString::new(path).unwrap();
    let ok = unsafe { mkfifo(path.as_ptr(), mode) == 0 };
    if ok {
        Ok(())
    } else {
        Err(io::Error::last_os_error().into())
    }
}

fn lua_exec(chunk: String, args: Vec<String>) -> Result<Vec<String>> {
    let lua = Lua::new();
    let globals = lua.globals();
    let kak = lua.create_table()?;

    kak.set(
        "send_to",
        lua.create_function(|_, (ses, cmd): (String, String)| Ok(kak_send_msg(&ses, &cmd)?))?,
    )?;

    kak.set(
        "send_all",
        lua.create_function(|_, cmd: String| {
            let glua_list = Command::new("glua")
                .args(["-l"])
                .stdout(Stdio::piped())
                .output()?;

            let sessions = String::from_utf8(glua_list.stdout)
                .unwrap();

                
            for ses in sessions.trim().split('\n') {
                kak_send_msg(ses, &cmd)?;
            }

            Ok(())
        })?,
    )?;

    kak.set(
        "mkfifo",
        lua.create_function(|_, path: String| create_fifo(&path, 0o777))?
    )?;

    globals.set("kak", kak)?;

    lua.globals().set::<_, Vec<String>>("arg", args)?;
    let vals: MultiValue = lua.load(&chunk).eval()?;

    let mut result = Vec::<String>::new();
    for val in vals.into_iter() {
        if let Ok(v) = String::from_lua(val, &lua) {
            result.push(v);
        }
    }

    Ok(result)
}

fn encode(msg: &str) -> Vec<u8> {
    let mut result = Vec::<u8>::with_capacity(msg.len() + 9);
    result.splice(..0, (msg.len() as u32).to_ne_bytes());
    msg.bytes().for_each(|b| result.push(b));
    result.splice(..0, (result.len() as u32 + 5).to_ne_bytes());
    result.insert(0, b'\x02');

    result
}

fn kak_send_msg(session: &str, msg: &str) -> Result<()> {
    let rntmd = std::env::var("XDG_RUNTIME_DIR").expect("runtimedir");
    let socket = std::path::Path::new(&rntmd).join("kakoune").join(session);
    let mut stream = std::os::unix::net::UnixStream::connect(socket)?;
    let _ = stream.write(&encode(msg))?;
    stream.flush()?;

    Ok(())
}

fn run() {
    let mut args = std::env::args().skip(1).collect::<Vec<String>>();

    if args.len() < 1 {
        return println!("fail Wrong argument count");
    }

    let chunk = args.pop().unwrap();
    match lua_exec(chunk, args) {
        Err(lua_err) => return println!("fail {lua_err}"),
        Ok(ret_vals) => {
            for val in ret_vals {
                println!("{val}");
            }
        }
    }
}

fn main() {
    run();
}
