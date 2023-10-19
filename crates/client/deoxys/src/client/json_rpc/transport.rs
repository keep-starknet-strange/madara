//! Defines the [`Transport`] trait.

use std::future::Future;

/// Represents the transport layer used to communicate over the network.
pub trait Transport {
    /// An error that might occur whilst communicating over the network.
    type Error;

    /// The future type returned by [`Transport::request`].
    type Future: Future<Output = Result<Vec<u8>, Self::Error>>;

    /// Sends a request over the network and returns the response.
    fn request(&self, body: &[u8]) -> Self::Future;
}

impl<T: Transport> Transport for &'_ T {
    type Error = T::Error;
    type Future = T::Future;

    fn request(&self, body: &[u8]) -> Self::Future {
        (**self).request(body)
    }
}

impl<T: Transport> Transport for Box<T> {
    type Error = T::Error;
    type Future = T::Future;

    fn request(&self, body: &[u8]) -> Self::Future {
        (**self).request(body)
    }
}

impl<T: Transport> Transport for std::sync::Arc<T> {
    type Error = T::Error;
    type Future = T::Future;

    fn request(&self, body: &[u8]) -> Self::Future {
        (**self).request(body)
    }
}
