use ethabi;
use futures::{Async, Future, Poll};
use serde;
use std::mem;

use crate::contract;
use crate::contract::tokens::Detokenize;
use crate::helpers;
use crate::rpc;
use crate::types::Bytes;
use crate::Error as ApiError;

#[derive(Debug)]
enum ResultTokensType<F> {
    Decodable(helpers::CallFuture<Bytes, F>, ethabi::Function),
    Constant(Result<Vec<ethabi::Token>, contract::Error>),
    Done,
}

/// Function-specific bytes-decoder future.
/// Takes any type which is deserializable from `Vec<ethabi::Token>`,
/// a function definition and a future which yields that type.
#[derive(Debug)]
pub struct QueryTokensResult<F> {
    inner: ResultTokensType<F>,
}

impl<F, E> From<E> for QueryTokensResult<F>
where
    E: Into<contract::Error>,
{
    fn from(e: E) -> Self {
        QueryTokensResult {
            inner: ResultTokensType::Constant(Err(e.into())),
        }
    }
}

impl<F> QueryTokensResult<F> {
    /// Create a new `QueryTokensResult` wrapping the inner future.
    pub fn new(inner: helpers::CallFuture<Bytes, F>, function: ethabi::Function) -> Self {
        QueryTokensResult {
            inner: ResultTokensType::Decodable(inner, function),
        }
    }
}

impl<F> Future for QueryTokensResult<F>
where
    F: Future<Item = rpc::Value, Error = ApiError>,
{
    type Item = Vec<ethabi::Token>;
    type Error = contract::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let ResultTokensType::Decodable(ref mut inner, ref function) = self.inner {
            let bytes: Bytes = try_ready!(inner.poll());
            return Ok(Async::Ready(function.decode_output(&bytes.0)?));
        }

        match mem::replace(&mut self.inner, ResultTokensType::Done) {
            ResultTokensType::Constant(res) => res.map(Async::Ready),
            _ => panic!("Unsupported state"),
        }
    }
}
