// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

//! Allow the sending and reciving of typed messages.
//!
//! # Example
//! ```
//! # async_std::task::block_on(async {
//! # use async_std::prelude::FutureExt;
//! use async_reply::Message;
//!
//! #[derive(Debug, Message)]
//! #[rtype(response = "Pong")]
//! struct Ping;
//!
//! #[derive(Debug)]
//! struct Pong;
//!
//! let (requester, replyer) = async_reply::endpoints();
//!
//! let ping_fut = async {
//!     println!("Sending Ping");
//!     let reply = requester.send(Ping).await.unwrap();
//!     println!("Received {:?}", reply);
//! };
//!
//! let pong_fut = async {
//!     let (msg, handler) = replyer.recv::<Ping>().await.unwrap();
//!     handler.respond(Pong).await.unwrap();
//!     println!("Replied {:?} with Pong", msg);
//! };
//!
//! ping_fut.join(pong_fut).await;
//! # });
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use async_std::{channel, prelude::FutureExt, sync::Mutex};
use std::any::Any;

#[doc(hidden)]
#[cfg(feature = "derive")]
pub use async_reply_derive::*;

/// Create a [`Requester`] and [`Replyer`] message endpoints which allow the
/// sending and receiving of typed messages.
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

/// The requester side of a endpoint.
#[derive(Debug, Clone)]
pub struct Requester {
    inner: channel::Sender<Box<dyn Any + Send>>,
}

/// The replyer side of a endpoint.
#[derive(Debug)]
pub struct Replyer {
    buffer: Mutex<Vec<Box<dyn Any + Send>>>,
    inner: channel::Receiver<Box<dyn Any + Send>>,
}

/// The reply handle to respond to the received message.
#[must_use = "ReplyHandle should be used to respond to the received message"]
#[derive(Debug)]
pub struct ReplyHandle<T>(channel::Sender<T>);

struct MessageHandle<M: Message> {
    msg: M,
    sndr: ReplyHandle<M::Response>,
}

/// A trait to bind the message and its respective response type.
pub trait Message: 'static + Send {
    /// The response type of the message.
    type Response: Send;
}

impl Requester {
    /// Send the message and wait its response.
    pub async fn send<M>(&self, msg: M) -> Result<M::Response, Error>
    where
        M: Message,
    {
        let (sndr, recv) = channel::bounded::<M::Response>(1);
        let sndr = ReplyHandle(sndr);

        self.inner
            .send(Box::new(MessageHandle { msg, sndr }))
            .await?;

        recv.recv().await.map_err(Error::ReplayError)
    }
}

impl Replyer {
    /// Receives the message and provide the handle to respond back.
    pub async fn recv<M>(&self) -> Result<(M, ReplyHandle<M::Response>), Error>
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
    /// Respond back to a received message.
    pub async fn respond(&self, r: T) -> Result<(), Error> {
        Ok(self.0.send(r).await?)
    }
}

impl<M: Message> MessageHandle<M> {
    fn into_tuple(self) -> (M, ReplyHandle<M::Response>) {
        (self.msg, self.sndr)
    }
}

/// Encapsulate the errors which can be triggered when sending or receiving a
/// message.
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum Error {
    /// Error while sending the message.
    SendError,

    /// Error to receive the response of sent message.
    #[from(ignore)]
    #[display(transparent)]
    ReplayError(async_std::channel::RecvError),

    /// Error while receiving the message.
    #[display(transparent)]
    ReceivError(async_std::channel::RecvError),
}

impl<T> From<channel::SendError<T>> for Error {
    fn from(_e: channel::SendError<T>) -> Self {
        // The original error from async_std::channel::Sender carries the undelivered
        // message for recovery. However here we want to avoid raising the arity of
        // the Error type, losing that ability but making the error type more
        // permissive
        Error::SendError
    }
}
