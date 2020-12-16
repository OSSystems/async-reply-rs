// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

use async_std::io;

enum Command {
    Inc,
    Dec,
    Set,
}

impl async_reply::Message for Command {
    type Response = i64;
}

async fn io_process(req: async_reply::Requester) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut line;
    loop {
        print!(
            r#"
Use the following commands to change internal state:
INC - Increment internal state by 1
DEC - Decrement internal state by 1
SET - Set internal state to 0
"#
        );
        line = String::default();
        stdin.read_line(&mut line).await?;
        let command = match line.trim().to_lowercase().as_str() {
            "inc" => Command::Inc,
            "dec" => Command::Dec,
            "set" => Command::Set,
            _ => {
                println!("Invalid command!");
                continue;
            }
        };
        let val = req.send(command).await.unwrap();
        println!("State: {}", val);
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (req, rep) = async_reply::endpoints();
    async_std::task::spawn_local(io_process(req));

    let mut state = 0;
    loop {
        let (command, handle) = rep.recv().await?;
        match command {
            Command::Inc => state += 1,
            Command::Dec => state -= 1,
            Command::Set => state = 0,
        }
        handle.respond(state).await;
    }
}
