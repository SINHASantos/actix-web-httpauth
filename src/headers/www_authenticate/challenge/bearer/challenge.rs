use std::borrow::Cow;
use std::fmt;
use std::str;

use actix_web::http::header::{HeaderValue, IntoHeaderValue, InvalidHeaderValueBytes};
use bytes::{BufMut, Bytes, BytesMut};

use super::super::Challenge;
use super::{BearerBuilder, Error};
use crate::utils;

/// Challenge for [`WWW-Authenticate`] header with HTTP Bearer auth scheme,
/// described in [RFC 6750](https://tools.ietf.org/html/rfc6750#section-3)
///
/// ## Example
///
/// ```rust
/// # use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
/// use actix_web_httpauth::headers::www_authenticate::bearer::{Bearer, Error};
/// use actix_web_httpauth::headers::www_authenticate::WwwAuthenticate;
///
/// fn index(_req: HttpRequest) -> HttpResponse {
///     let challenge = Bearer::build()
///         .realm("example")
///         .scope("openid profile email")
///         .error(Error::InvalidToken)
///         .error_description("The access token expired")
///         .error_uri("http://example.org")
///         .finish();
///
///     HttpResponse::Unauthorized().set(WwwAuthenticate(challenge)).finish()
/// }
/// ```
///
/// [`WWW-Authenticate`]: ../struct.WwwAuthenticate.html
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Clone)]
pub struct Bearer {
    pub(crate) scope: Option<Cow<'static, str>>,
    pub(crate) realm: Option<Cow<'static, str>>,
    pub(crate) error: Option<Error>,
    pub(crate) error_description: Option<Cow<'static, str>>,
    pub(crate) error_uri: Option<Cow<'static, str>>,
}

impl Bearer {
    /// Creates the builder for `Bearer` challenge.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use actix_web_httpauth::headers::www_authenticate::bearer::{Bearer};
    /// let challenge = Bearer::build()
    ///     .realm("Restricted area")
    ///     .scope("openid profile email")
    ///     .finish();
    /// ```
    pub fn build() -> BearerBuilder {
        BearerBuilder::default()
    }
}

#[doc(hidden)]
impl Challenge for Bearer {
    fn to_bytes(&self) -> Bytes {
        let desc_uri_required = self.error_description.as_ref().map_or(0, |desc| desc.len() + 20)
            + self.error_uri.as_ref().map_or(0, |url| url.len() + 12);
        let capacity = 6
            + self.realm.as_ref().map_or(0, |realm| realm.len() + 9)
            + self.scope.as_ref().map_or(0, |scope| scope.len() + 9)
            + desc_uri_required;
        let mut buffer = BytesMut::with_capacity(capacity);
        buffer.put("Bearer");

        if let Some(ref realm) = self.realm {
            buffer.put(" realm=\"");
            utils::put_cow(&mut buffer, realm);
            buffer.put("\"");
        }

        if let Some(ref scope) = self.scope {
            buffer.put(" scope=\"");
            utils::put_cow(&mut buffer, scope);
            buffer.put("\"");
        }

        if let Some(ref error) = self.error {
            let error_repr = error.as_str();
            let remaining = buffer.remaining_mut();
            let required = desc_uri_required + error_repr.len() + 9; // 9 is for `" error=\"\""`
            if remaining < required {
                buffer.reserve(required);
            }
            buffer.put(" error=\"");
            buffer.put(error_repr);
            buffer.put("\"")
        }

        if let Some(ref error_description) = self.error_description {
            buffer.put(" error_description=\"");
            utils::put_cow(&mut buffer, error_description);
            buffer.put("\"");
        }

        if let Some(ref error_uri) = self.error_uri {
            buffer.put(" error_uri=\"");
            utils::put_cow(&mut buffer, error_uri);
            buffer.put("\"");
        }

        buffer.freeze()
    }
}

impl fmt::Display for Bearer {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let bytes = self.to_bytes();
        let repr = str::from_utf8(&bytes)
            // Should not happen since challenges are crafted manually
            // from `&'static str`'s and Strings
            .map_err(|_| fmt::Error)?;

        f.write_str(repr)
    }
}

impl IntoHeaderValue for Bearer {
    type Error = InvalidHeaderValueBytes;

    fn try_into(self) -> Result<HeaderValue, <Self as IntoHeaderValue>::Error> {
        HeaderValue::from_shared(self.to_bytes())
    }
}