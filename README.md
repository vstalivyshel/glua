Layer between the lua interpreter and the Kakoune text editor.
I'm just experimenting with language, sockets, and some APIs. Pretty much useless.

The only interesting thing that you may find here is this chunk of code,
with which you will be able to send Kakoune commands directly to the current
session's unix socket (which lives in "*$XDG_RUNTIME_DIR*" or in your "*/tmp*")
instead of using "*kak -p*":

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
