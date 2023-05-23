use std::{
    collections::VecDeque,
    env,
    ffi::CString,
    fs,
    io::{self, BufRead, BufReader, BufWriter, Read, Result, Write},
    path::Path,
    process::{Command, Stdio},
};

extern "C" {
    fn mkfifo(path: *const i8, mode: u32) -> i32;
    fn daemon(nochdir: i32, noclose: i32) -> i32;
}

struct TempFile {
    path: String,
}

impl From<&str> for TempFile {
    fn from(s: &str) -> Self {
        Self {
            path: s.to_string(),
        }
    }
}

impl TempFile {
    fn in_tempdir(name: &str) -> Self {
        let mut path = env::temp_dir().join(name).to_str().unwrap().to_string();
        while Path::new(&path).exists() {
            path.push('X');
        }

        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn daemonize() -> Result<()> {
    if unsafe { daemon(0, 0) != 0 } {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

fn temp_fifo() -> Result<TempFile> {
    let holder = TempFile::in_tempdir("glua_temp_fifo");
    create_fifo(holder.path.as_ref(), 0o777)?;

    Ok(holder)
}

fn create_fifo(path: &str, mode: u32) -> Result<()> {
    let path = CString::new(path).unwrap();
    if unsafe { mkfifo(path.as_ptr(), mode) != 0 } {
        return Err(io::Error::last_os_error());
    }

    Ok(())
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

fn run() -> Result<()> {
    let mut args = env::args().skip(1).collect::<VecDeque<String>>();
    let args_len = args.len();
    if args_len < 1 {
        return Ok(eprintln!("fail wrong argument count"));
    }

    match args.pop_front().unwrap().as_str() {
        "pipe" => {
            if args_len >= 1 {
                let fifo = temp_fifo()?;
                println!("{p}", p = &fifo.path);
                let mut cmd = Command::new(&args.pop_front().unwrap())
                    .args(args.make_contiguous())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;
                let fifo_path = Path::new(&fifo.path);

                let stdout = cmd.stdout.take().unwrap();
                let stderr = cmd.stderr.take().unwrap();
                let reader = BufReader::new(stdout.chain(stderr));

                daemonize()?;
                let fifo: fs::File = match fs::OpenOptions::new().write(true).open(fifo_path) {
                    Err(_) => return Ok(()),
                    Ok(f) => f,
                };

                let mut writer = BufWriter::new(fifo);
                for out in reader.split(b'\n') {
                    let mut out = match out {
                        Ok(o) => o,
                        Err(e) => e.to_string().into_bytes(),
                    };
                    out.push(b'\n');
                    let _ = writer.write_all(out.as_slice());
                }
            }
        }
        "eval" if args_len >= 2 => kak_send_msg(
            &args.pop_front().unwrap(),
            &args
                .into_iter()
                .map(|mut a| {
                    a.push(' ');
                    a
                })
                .collect::<String>(),
        )?,
        "evalall" if args_len >= 1 => {
            let glua_list = Command::new("kak")
                .args(["-l"])
                .stdout(Stdio::piped())
                .output()?;

            let sessions = String::from_utf8(glua_list.stdout).unwrap();

            let cmd = args
                .into_iter()
                .map(|mut a| {
                    a.push(' ');
                    a
                })
                .collect::<String>();

            for ses in sessions.trim().split('\n') {
                kak_send_msg(ses, &cmd)?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn main() {
    if let Err(some_er) = run() {
        eprintln!("fail {some_er}");
    }
}
