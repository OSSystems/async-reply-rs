[![Documentation](https://docs.rs/async-reply/badge.svg)](https://docs.rs/async-reply-derive)

# async-reply-derive

Derive macros for `async-reply` message.

## Usage

```rust
use async_reply::Message;

#[derive(Message)]
#[rtype(response = "Pong")]
struct Ping;

struct Pong;

fn main() {}
```

This code expands into following code:

```rust
use async_reply::Message;

struct Ping;

struct Pong;

impl Message for Ping {
    type Response = Pong;
}

fn main() {}
```


## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
