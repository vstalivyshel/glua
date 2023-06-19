use std::{
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
    let path = CString::new(holder.path.as_str()).unwrap();
    if unsafe { mkfifo(path.as_ptr(), 0o777) != 0 } {
        return Err(io::Error::last_os_error());
    }

    Ok(holder)
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
    let mut args = env::args().skip(1);
    let sub = if let Some(cmd) = args.next() {
        cmd
    } else {
        return Ok(eprintln!("fail wrong argument count"));
    };

    match sub.as_str() {
        "pipe" if args.len() >= 1 => {
            let cmd = args.next().unwrap();
            if cmd.is_empty() {
                return Ok(eprintln!("fail invalid argument to `pipe` subcommand"));
            }

            let mut child = Command::new(&cmd)
                .args(args.collect::<Vec<String>>().as_slice())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();
            let reader = BufReader::new(stdout.chain(stderr));

            let fifo = temp_fifo()?;
            println!("{p}", p = &fifo.path);

            daemonize()?;

            let fifo: fs::File = match fs::OpenOptions::new()
                .write(true)
                .open(Path::new(&fifo.path))
            {
                Ok(f) => f,
                Err(_) => return Ok(()),
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
        _ => eprintln!(
            "fail failed to run \"{cmd}\"",
            cmd = {
                let mut cmd = sub;
                cmd.push(' ');
                cmd.push_str(
                    args.map(|mut a| {
                        a.push(' ');
                        a
                    })
                    .collect::<String>()
                    .trim(),
                );
                cmd
            }
        ),
    }

    Ok(())
}

fn main() {
    let n = "halo";
    let b = encode(n);
    println!("{b:?}");
    // if let Err(some_er) = run() {
    //     eprintln!("fail {some_er}");
    // }
}
