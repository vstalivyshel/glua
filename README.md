The only interesting thing that you may find here is this chunk of code,
with which you will be able to send Kakoune commands directly to the current
session's unix socket (which lives in "$XDG_RUNTIME_DIR" or in your "/tmp")
instead of using "kak -p":

```rust
fn encode(msg: &str) -> Vec<u8> {
    let mut result = Vec::<u8>::with_capacity(msg.len() + 9);
    result.splice(..0, (msg.len() as u32).to_ne_bytes()); // Length of a msg string
    msg.bytes().for_each(|b| result.push(b));
    result.splice(..0, (result.len() as u32 + 5).to_ne_bytes()); // Length of the whole encoded message
                                                                 // and + 5 for some reason
                                                                
    result.insert(0, b'\x02'); // Tells to the editor that the message is a command
                               // This is basically index of enum's variant from source code

    // For example:
    //
    // magic_byte = 2 <- constant
    // msg = 'halo'
    // msg_bytes = [ 104, 97, 108, 111 ]
    // My machine uses little endian to represent numbers:
    // msg_len = 4
    // msg_len_bytes = [ 4, 0, 0, 0 ]
    // msg_len_and_msg = msg_len_bytes + msg_bytes
    // whole_msg_len = len(msg_len_and_msg) + 5 = 13
    // whole_msg_len_bytes = [ 13, 0, 0, 0 ]
    // The order matters
    // result = magic_byte + whole_msg_len_bytes + msg_len_bytes + msg_bytes
    // result = [ 2, 13, 0, 0, 0, 4, 0, 0, 0, 104, 97, 108, 111 ]
    //
    // I am not sure about all that, but it worked .
    result
}
```

You may find more answers how it works here (why would you):
- [initial idea](https://github.com/caksoylar/kakoune-smooth-scroll/blob/master/smooth-scroll.py)
- [source code](https://github.com/mawww/kakoune/blob/master/src/remote.cc)
