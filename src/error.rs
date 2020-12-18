// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum Error<T> {
    SendError(async_std::channel::SendError<T>),
    #[from(ignore)]
    ReplayError(async_std::channel::RecvError),
    ReceivError(async_std::channel::RecvError),
}
