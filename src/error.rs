// Copyright (C) 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: Apache-2.0

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum Error {
    ReplayError(async_std::sync::RecvError),
    ReceivError(async_std::sync::RecvError),
}
