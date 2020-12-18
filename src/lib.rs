// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

pub use crate::error::Error;

use async_std::channel;
use async_std::prelude::FutureExt;
use async_std::sync::Mutex;
use std::any::Any;

mod error;

pub fn endpoints() -> (Requester, Replyer) {
    let (sndr, recv) = channel::bounded(10);
    (
        Requester { inner: sndr },
        Replyer {
            buffer: Mutex::default(),
            inner: recv,
        },
    )
}

#[derive(Debug, Clone)]
pub struct Requester {
    inner: channel::Sender<Box<dyn Any + Send>>,
}

#[derive(Debug)]
pub struct Replyer {
    buffer: Mutex<Vec<Box<dyn Any + Send>>>,
    inner: channel::Receiver<Box<dyn Any + Send>>,
}

#[must_use = "RespondeHandle should be used to respond to the received message"]
#[derive(Debug)]
pub struct ReplyHandle<T>(channel::Sender<T>);

struct MessageHandle<M: Message> {
    msg: M,
    sndr: ReplyHandle<M::Response>,
}

pub trait Message: 'static + Send {
    type Response: Send;
}

impl Requester {
    pub async fn send<M>(&self, msg: M) -> Result<M::Response, Error<M>>
    where
        M: Message,
    {
        let (sndr, recv) = channel::bounded::<M::Response>(1);
        let sndr = ReplyHandle(sndr);

        if let Err(e) = self.inner.send(Box::new(MessageHandle { msg, sndr })).await {
            // We need to convert it here as we need to unwrap the message
            // type so the error handling can use the message if need.
            return Err(Error::SendError(channel::SendError(
                *e.into_inner().downcast::<M>().unwrap(),
            )));
        }

        recv.recv().await.map_err(Error::ReplayError)
    }
}

impl Replyer {
    pub async fn recv<M>(&self) -> Result<(M, ReplyHandle<M::Response>), Error<M>>
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
    pub async fn respond(&self, r: T) -> Result<(), Error<T>> {
        Ok(self.0.send(r).await?)
    }
}

impl<M: Message> MessageHandle<M> {
    fn into_tuple(self) -> (M, ReplyHandle<M::Response>) {
        (self.msg, self.sndr)
    }
}
