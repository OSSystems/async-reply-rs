// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use async_std::prelude::FutureExt;

#[derive(Debug)]
struct Ping(usize);

#[derive(Debug)]
struct Pong(usize);

impl async_reply::Message for Ping {
    type Response = Pong;
}

#[async_std::main]
async fn main() {
    let (req, mut rep) = async_reply::endpoints();
    let fut1 = async move {
        let mut x = 0;
        loop {
            let res = req.send(Ping(x)).await.unwrap();
            println!("Pong: {}", res.0);
            x = res.0 + 1;
            if res.0 >= 5 {
                break;
            }
        }
    };

    let fut2 = async move {
        loop {
            match rep.recv::<Ping>().await {
                Ok((msg, handle)) => {
                    println!("Ping: {}", msg.0);
                    handle.respond(Pong(msg.0 + 1)).await;
                }
                Err(_) => break,
            }
        }
    };

    fut1.join(fut2).await;
}
