[![Documentation](https://docs.rs/async-reply/badge.svg)](https://docs.rs/async-reply)

# async-reply

Allow the sending and reciving of typed messages.

## Example
```rust
use async_reply::Message;

#[derive(Debug, Message)]
#[rtype(response = "Pong")]
struct Ping;

#[derive(Debug)]
struct Pong;

let (requester, replyer) = async_reply::endpoints();

let ping_fut = async {
    println!("Sending Ping");
    let reply = requester.send(Ping).await.unwrap();
    println!("Received {:?}", reply);
};

let pong_fut = async {
    let (msg, handler) = replyer.recv::<Ping>().await.unwrap();
    handler.respond(Pong).await.unwrap();
    println!("Replied {:?} with Pong", msg);
};

ping_fut.join(pong_fut).await;
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
