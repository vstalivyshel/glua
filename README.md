I'm just experimenting with language, sockets, and some APIs. Pretty much useless.

The only interesting thing that you may find here is this chunk of code,
with which you will be able to send Kakoune commands directly to the current
session's unix socket (which lives in "$XDG_RUNTIME_DIR" or in your "/tmp")
instead of using "kak -p":

```rust
fn encode(msg: &str) -> Vec<u8> {
    let mut result = Vec::<u8>::with_capacity(msg.len() + 9);
    result.splice(..0, (msg.len() as u32).to_ne_bytes());
    msg.bytes().for_each(|b| result.push(b));
    result.splice(..0, (result.len() as u32 + 5).to_ne_bytes());
    result.insert(0, b'\x02');

    result
}
```
'Notes' section. Not mature enough to turn it into a useful description.  
Ok, no Lua script anymore. More focus on cli functionality.

the same as `kak -p`
``` bash
glua eval <session> <cmd> 
```

the same as `kak -p` but for all existing sessions listed in `kak -l`   
TODO: This spits a useless error about a failed connection in the debug buffer but still works fine. fix it.
``` bash
glua evalall <cmd>
```

Run the shell command as a daemon, throwing output into `glua_temp_fifo` in your /tmp.
TODO: What if someone decides to manually remove the fifo file? so there will be a zombie process. fix it.
``` bash
glua pipe <shell-cmd>
```

This subcommand will output the path to temp fifo after invoking, so the fallowing will work just fine:

```
edit -fifo %sh{ glua pipe ls -Ahl } *somebufname*
```

TODO: Make this readme more readable, please. Why?
