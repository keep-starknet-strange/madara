//! An implementation of [`Transport`] that uses the [`hyper`] HTTP client.

use std::future::Future;
use std::pin::Pin;

use hyper::body::HttpBody;
use hyper::client::connect::Connect;
use hyper::Uri;

/// The configuration for an [`HttpTransport`].
#[derive(Debug, Clone)]
pub struct Config {
    /// The URI to send requests to.
    pub uri: Uri,
}

/// An implementation of [`Transport`] that uses the [`hyper`] HTTP client.
pub struct Transport<C = hyper_rustls::HttpsConnector<hyper::client::HttpConnector>> {
    client: hyper::Client<C>,
    uri: hyper::Uri,
}

impl Transport {
    /// Creates a new [`HttpTransport`] with the given configuration.
    pub fn new(config: Config) -> Self {
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .wrap_connector(hyper::client::HttpConnector::new());

        Self::with_connector(config.uri, connector)
    }
}

impl<C> Transport<C> {
    /// Creates a new [`HttpTransport`] with the given connector.
    pub fn with_connector(uri: Uri, connector: C) -> Self
    where
        C: Connect + Clone,
    {
        Self { uri, client: hyper::Client::builder().build(connector) }
    }
}

impl<C> super::Transport for Transport<C>
where
    C: Connect + Clone + Send + Sync + 'static,
{
    type Error = hyper::Error;

    type Future<'a> = Pin<Box<dyn 'a + Future<Output = Result<Vec<u8>, Self::Error>> + Send + Sync>>
    where
        Self: 'a;

    fn request(&self, body: &[u8]) -> Self::Future<'_> {
        let body = hyper::Body::from(body.to_vec());

        let fut = async move {
            let mut response = self
                .client
                .request(
                    hyper::Request::builder()
                        .method(hyper::Method::POST)
                        .uri(self.uri.clone())
                        .header("content-type", "application/json")
                        .body(body)
                        .unwrap(),
                )
                .await?;

            let mut v = Vec::new();
            if let Some(chunk) = response.body_mut().data().await {
                v.extend_from_slice(&chunk?);
            }

            Ok(v)
        };

        Box::pin(fut)
    }
}
