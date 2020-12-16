// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

pub use crate::error::{Error, Result};

use async_std::prelude::FutureExt;
use async_std::sync::{self, Mutex};
use std::any::Any;

mod error;

pub fn endpoints() -> (Requester, Replyer) {
    let (sndr, recv) = sync::channel(10);
    (
        Requester { inner: sndr },
        Replyer {
            buffer: Mutex::default(),
            inner: recv,
        },
    )
}

#[derive(Clone)]
pub struct Requester {
    inner: sync::Sender<Box<dyn Any + Send>>,
}

pub struct Replyer {
    buffer: Mutex<Vec<Box<dyn Any + Send>>>,
    inner: sync::Receiver<Box<dyn Any + Send>>,
}

#[must_use = "RespondeHandle should be used to respond to the received message"]
pub struct ReplyHandle<T>(sync::Sender<T>);

struct MessageHandle<M: Message> {
    msg: M,
    sndr: ReplyHandle<M::Response>,
}

pub trait Message: 'static + Send {
    type Response: Send;
}

impl Requester {
    pub async fn send<M>(&self, msg: M) -> Result<M::Response>
    where
        M: Message,
    {
        let (sndr, recv) = sync::channel::<M::Response>(1);
        let sndr = ReplyHandle(sndr);
        self.inner.send(Box::new(MessageHandle { msg, sndr })).await;
        recv.recv().await.map_err(Error::ReplayError)
    }
}

impl Replyer {
    pub async fn recv<M>(&self) -> Result<(M, ReplyHandle<M::Response>)>
    where
        M: Message,
    {
        let is_message_type = |any: &Box<dyn Any + Send>| any.is::<MessageHandle<M>>();

        loop {
            let buffer_search_fut = async {
                loop {
                    let mut buffer = self.buffer.lock().await;
                    let msg_index = buffer
                        .iter()
                        .enumerate()
                        .find(|(_, elem)| is_message_type(elem))
                        .map(|(index, _)| index);
                    if let Some(index) = msg_index {
                        // We have a buffereda message of this type, so we pop
                        // and return it
                        return Ok(buffer.remove(index));
                    }
                    async_std::task::yield_now().await;
                }
            };
            let channel_search_fut = async { self.inner.recv().await.map_err(Error::ReceivError) };

            let msg = buffer_search_fut.race(channel_search_fut).await?;
            if is_message_type(&msg) {
                return Ok(msg.downcast::<MessageHandle<M>>().unwrap().into_tuple());
            }
            self.buffer.lock().await.push(msg);
        }
    }
}

impl<T> ReplyHandle<T> {
    pub async fn respond(&self, r: T) {
        self.0.send(r).await;
    }
}

impl<M: Message> MessageHandle<M> {
    fn into_tuple(self) -> (M, ReplyHandle<M::Response>) {
        (self.msg, self.sndr)
    }
}
