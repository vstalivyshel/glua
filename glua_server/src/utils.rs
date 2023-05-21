use crate::{f, SELF};
use std::{
    path::{PathBuf,Path},
    io::{self, Write},
    os::unix::net::UnixStream,
};

#[macro_export]
macro_rules! f {
    ($($a:expr)*) => {{
        let mut nice = String::new();
        $(
        nice.push_str($a.as_ref());
        nice.push(' ');
        )*
        nice
    }}
}

pub fn kak_send_msg(session: &str, msg: &str) -> Result<(), io::Error> {
    let rntm = std::env::var("XDG_RUNTIME_DIR").expect("runtimedir");
    let socket = std::path::PathBuf::from(rntm).join("kakoune").join(session);
    let mut stream = UnixStream::connect(socket)?;
    let _  = stream.write(&encode(msg))?;
    stream.flush()?;

    Ok(())
}

pub fn kak_send_client(session: &str, client: &str, msg: &str) -> Result<(), io::Error> {
    kak_send_msg(session, &f!("evaluate-commands -client" client msg.kakqt()))
}

pub struct TempFile {
    pub path: PathBuf,
}

impl<P: AsRef<Path>> From<P> for TempFile {
    fn from(path: P) -> Self {
        Self {
            path: path.as_ref().into(),
        }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn encode(msg: &str) -> Vec<u8> {
    let mut result = Vec::<u8>::with_capacity(msg.len() + 9);
    result.splice(..0, (msg.len() as u32).to_ne_bytes());
    msg.bytes().for_each(|b| result.push(b));
    result.splice(..0, (result.len() as u32 + 5).to_ne_bytes());
    result.insert(0, b'\x02');

    result
}

pub trait StringExt: AsRef<str> + Sized {
    fn sur_with(self, op: &str, cl: &str) -> String {
        let mut val = String::new();
        val.push_str(op);
        val.push_str(self.as_ref());
        val.push_str(cl);

        val
    }

    fn and<S: StringExt>(self, more: S) -> String {
        let mut new = String::new();
        new.push_str(self.as_ref());
        new.push_str(more.as_ref());

        new
    }

    fn for_sh(self) -> String {
        self.sur_with("$kak_", "").dqt()
    }

    fn kakarg(self) -> String {
        self.sur_with("%arg¿", "¿")
    }

    fn kakreg(self) -> String {
        self.sur_with("%reg£", "£")
    }

    fn kakval(self) -> String {
        self.sur_with("%val®", "®")
    }

    fn kakopt(self) -> String {
        self.sur_with("%opt¶", "¶")
    }

    fn kakfile(self) -> String {
        self.sur_with("%file{", "}")
    }

    fn sh_cmd(self) -> String {
        self.sur_with("%sh{", "}")
    }

    fn qt(self) -> String {
        self.sur_with("'", "'")
    }

    fn dqt(self) -> String {
        self.sur_with("\"", "\"")
    }

    fn kakqt(self) -> String {
        self.sur_with("%[", "]")
    }

    fn and_sh_cmd<S: StringExt>(self, cmd: S) -> String {
        self.and(" ").and(cmd.sh_cmd())
    }

    fn and_kakqt<S: StringExt>(self, cmd: S) -> String {
        self.and(" ").and(cmd.kakqt())
    }

    fn and_sh_args<S: StringExtChain>(self, args: S) -> String {
        self.and(" ").and(args.kaksh_cmd())
    }

    fn block<S: StringExtChain>(self, block: S) -> String {
        self.and(" ").and(block.kakqt())
    }
}

impl StringExt for String {}
impl StringExt for &String {}
impl StringExt for &str {}

pub trait StringExtChain: IntoIterator {
    fn kakcmd(self) -> String;
    fn kakeval(self) -> String;
    fn kaksh_cmd(self) -> String;
    fn kakqt(self) -> String;
}

impl<T: StringExt, I: IntoIterator<Item = T>> StringExtChain for I {
    fn kakcmd(self) -> String {
        let mut cmd = String::new();
        for c in self.into_iter() {
            cmd.push_str(c.as_ref());
            cmd.push_str("; ");
        }

        cmd
    }

    fn kaksh_cmd(self) -> String {
        let mut sh_cmd = String::new();
        for c in self.into_iter() {
            sh_cmd.push_str(c.as_ref());
            sh_cmd.push(' ');
        }

        sh_cmd.sh_cmd()
    }

    fn kakqt(self) -> String {
        self.kakcmd().kakqt()
    }

    fn kakeval(self) -> String {
        self.kakcmd().kakqt().sur_with("eval ", "")
    }
}

