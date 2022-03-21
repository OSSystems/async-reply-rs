// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use async_std::prelude::FutureExt;

#[derive(Debug, async_reply::Message)]
#[rtype(response = "Pong")]
struct Ping(usize);

#[derive(Debug)]
struct Pong(usize);

#[async_std::main]
async fn main() {
    let (req, rep) = async_reply::endpoints();
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
        while let Ok((msg, handle)) = rep.recv::<Ping>().await {
            println!("Ping: {}", msg.0);
            handle.respond(Pong(msg.0 + 1)).await.unwrap();
        }
    };

    fut1.join(fut2).await;
}
