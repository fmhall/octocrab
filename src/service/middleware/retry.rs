use futures_util::future;
use http::{Request, Response};
use hyper_util::client::legacy::Error;
use tower::retry::Policy;

use crate::body::OctoBody;

#[derive(Clone)]
pub enum RetryConfig {
    None,
    Simple(usize),
}

impl<B> Policy<Request<OctoBody>, Response<B>, Error> for RetryConfig {
    type Future = futures_util::future::Ready<()>;

    fn retry(
        &mut self,
        _req: &mut Request<OctoBody>,
        result: &mut Result<Response<B>, Error>,
    ) -> Option<Self::Future> {
        match self {
            RetryConfig::None => None,
            RetryConfig::Simple(count) => match result {
                Ok(response) => {
                    if response.status().is_server_error() || response.status() == 429 {
                        if *count > 0 {
                            *count -= 1;
                            Some(future::ready(()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Err(_) => {
                    if *count > 0 {
                        *count -= 1;
                        Some(future::ready(()))
                    } else {
                        None
                    }
                }
            },
        }
    }

    fn clone_request(&mut self, req: &Request<OctoBody>) -> Option<Request<OctoBody>> {
        match self {
            RetryConfig::None => None,
            _ => {
                // `Request` can't be cloned
                let mut new_req = Request::builder()
                    .uri(req.uri())
                    .method(req.method())
                    .version(req.version());
                for (name, value) in req.headers() {
                    new_req = new_req.header(name, value);
                }

                let body = req.body().clone();
                let new_req = new_req.body(body).expect(
                    "This should never panic, as we are cloning a components from existing request",
                );
                Some(new_req)
            }
        }
    }
}
