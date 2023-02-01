#![doc = include_str!("../README.md")]

use std::future::{ready, Ready};
use std::sync::Arc;
use std::{fmt, ops};

use actix_web::dev::Payload;
use actix_web::http::StatusCode;
use actix_web::{Error, FromRequest, HttpRequest, ResponseError};
use derive_more::{Display, From};
use serde::de;

pub use serde_querystring::de::ParseMode;

/// Actix-web's web::Query modified to work with serde-querystring
///
/// [**QueryStringConfig**](struct.QueryStringConfig.html) allows to configure extraction process.
///
/// # Example
///
/// ```rust
/// use actix_web::{web, App};
/// use serde::Deserialize;
/// use serde_querystring_actix::QueryString;
///
/// #[derive(Debug, Deserialize)]
/// pub enum ResponseType {
///    Token,
///    Code
/// }
///
/// #[derive(Deserialize)]
/// pub struct AuthRequest {
///    id: u64,
///    response_type: ResponseType,
/// }
///
/// // Use `QueryString` extractor for query information (and destructure it within the signature).
/// // This handler gets called only if the request's query string contains a `username` field.
/// // The correct request for this handler would be `/index.html?id=64&response_type=Code"`.
/// // For more example visit the serde-querystring crate itself.
/// async fn index(QueryString(info): QueryString<AuthRequest>) -> String {
///     format!("Authorization request for client with id={} and type={:?}!", info.id, info.response_type)
/// }
///
/// fn main() {
///     let app = App::new().service(
///        web::resource("/index.html").route(web::get().to(index))); // <- use `Query` extractor
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryString<T>(pub T);

impl<T> QueryString<T> {
    /// Deconstruct to a inner value
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get query parameters from the path
    pub fn from_query(
        query_str: &str,
        parse_mode: serde_querystring::de::ParseMode,
    ) -> Result<Self, QueryStringPayloadError>
    where
        T: de::DeserializeOwned,
    {
        serde_querystring::de::from_str::<T>(query_str, parse_mode)
            .map(Self)
            .map_err(QueryStringPayloadError::Deserialize)
    }
}

impl<T> ops::Deref for QueryString<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> ops::DerefMut for QueryString<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: fmt::Display> fmt::Display for QueryString<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> FromRequest for QueryString<T>
where
    T: de::DeserializeOwned,
{
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let config = req
            .app_data::<QueryStringConfig>()
            .cloned()
            .unwrap_or_default();

        serde_querystring::de::from_str::<T>(req.query_string(), config.mode)
            .map(|val| ready(Ok(QueryString(val))))
            .unwrap_or_else(move |e| {
                let e = QueryStringPayloadError::Deserialize(e);

                log::debug!(
                    "Failed during QueryString extractor deserialization. \
                     Request path: {:?}",
                    req.path()
                );

                let e = if let Some(error_handler) = config.ehandler {
                    (error_handler)(e, req)
                } else {
                    e.into()
                };

                ready(Err(e))
            })
    }
}

/// QueryString extractor configuration
///
/// # Example
///
/// ```rust
/// use actix_web::{error, web, App, FromRequest, HttpResponse};
/// use serde::Deserialize;
/// use serde_querystring_actix::{QueryString, QueryStringConfig, ParseMode};
///
/// #[derive(Deserialize)]
/// struct Info {
///     username: String,
/// }
///
/// /// deserialize `Info` from request's querystring
/// async fn index(info: QueryString<Info>) -> String {
///     format!("Welcome {}!", info.username)
/// }
///
/// fn main() {
///     let app = App::new().service(
///         web::resource("/index.html").app_data(
///             // change query extractor configuration
///             QueryStringConfig::default()
///                 .parse_mode(ParseMode::Brackets) // <- choose the parsing mode
///                 .error_handler(|err, req| {  // <- create custom error response
///                     error::InternalError::from_response(
///                         err, HttpResponse::Conflict().finish()).into()
///                 })
///             )
///             .route(web::post().to(index))
///     );
/// }
/// ```
#[derive(Clone)]
pub struct QueryStringConfig {
    mode: serde_querystring::de::ParseMode,
    ehandler: Option<Arc<dyn Fn(QueryStringPayloadError, &HttpRequest) -> Error + Send + Sync>>,
}

impl QueryStringConfig {
    /// Set custom error handler
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(QueryStringPayloadError, &HttpRequest) -> Error + Send + Sync + 'static,
    {
        self.ehandler = Some(Arc::new(f));
        self
    }

    pub fn parse_mode(mut self, mode: serde_querystring::de::ParseMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Default for QueryStringConfig {
    fn default() -> Self {
        QueryStringConfig {
            mode: serde_querystring::de::ParseMode::Duplicate,
            ehandler: None,
        }
    }
}

/// A set of errors that can occur during parsing query strings
#[derive(Debug, Display, From)]
pub enum QueryStringPayloadError {
    /// Deserialize error
    #[display(fmt = "Query deserialize error: {}", _0)]
    Deserialize(serde_querystring::de::Error),
}

impl std::error::Error for QueryStringPayloadError {}

/// Return `BadRequest` for `QueryStringPayloadError`
impl ResponseError for QueryStringPayloadError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

#[cfg(test)]
mod tests {
    use actix_web::error::InternalError;
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use actix_web::HttpResponse;
    use derive_more::Display;
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize, Debug, Display)]
    struct Id {
        id: String,
    }

    #[actix_rt::test]
    async fn test_service_request_extract() {
        let req = TestRequest::with_uri("/name/user1/").to_srv_request();
        assert!(QueryString::<Id>::from_query(
            &req.query_string(),
            serde_querystring::de::ParseMode::UrlEncoded
        )
        .is_err());

        let req = TestRequest::with_uri("/name/user1/?id=test").to_srv_request();
        let mut s = QueryString::<Id>::from_query(
            &req.query_string(),
            serde_querystring::de::ParseMode::UrlEncoded,
        )
        .unwrap();

        assert_eq!(s.id, "test");
        assert_eq!(
            format!("{}, {:?}", s, s),
            "test, QueryString(Id { id: \"test\" })"
        );

        s.id = "test1".to_string();
        let s = s.into_inner();
        assert_eq!(s.id, "test1");
    }

    #[actix_rt::test]
    async fn test_request_extract() {
        let req = TestRequest::with_uri("/name/user1/").to_srv_request();
        let (req, mut pl) = req.into_parts();
        assert!(QueryString::<Id>::from_request(&req, &mut pl)
            .await
            .is_err());

        let req = TestRequest::with_uri("/name/user1/?id=test").to_srv_request();
        let (req, mut pl) = req.into_parts();

        let mut s = QueryString::<Id>::from_request(&req, &mut pl)
            .await
            .unwrap();
        assert_eq!(s.id, "test");
        assert_eq!(
            format!("{}, {:?}", s, s),
            "test, QueryString(Id { id: \"test\" })"
        );

        s.id = "test1".to_string();
        let s = s.into_inner();
        assert_eq!(s.id, "test1");
    }

    #[actix_rt::test]
    async fn test_custom_error_responder() {
        let req = TestRequest::with_uri("/name/user1/")
            .app_data(QueryStringConfig::default().error_handler(|e, _| {
                let resp = HttpResponse::UnprocessableEntity().finish();
                InternalError::from_response(e, resp).into()
            }))
            .to_srv_request();

        let (req, mut pl) = req.into_parts();
        let query = QueryString::<Id>::from_request(&req, &mut pl).await;

        assert!(query.is_err());
        assert_eq!(
            query
                .unwrap_err()
                .as_response_error()
                .error_response()
                .status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
}
