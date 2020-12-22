#![recursion_limit = "128"]

//! Derive macros for `async-reply` message.
//!
//! ## Usage
//!
//! ```rust
//! use async_reply::Message;
//!
//! #[derive(Message)]
//! #[rtype(response = "Pong")]
//! struct Ping;
//!
//! struct Pong;
//!
//! fn main() {}
//! ```
//!
//! This code expands into following code:
//!
//! ```rust
//! use async_reply::Message;
//!
//! struct Ping;
//!
//! struct Pong;
//!
//! impl Message for Ping {
//!     type Response = Pong;
//! }
//!
//! fn main() {}
//! ```

extern crate proc_macro;

use proc_macro::TokenStream;
use syn::DeriveInput;

mod message;

#[proc_macro_derive(Message, attributes(rtype))]
pub fn message_derive_rtype(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    message::expand(&ast).into()
}
