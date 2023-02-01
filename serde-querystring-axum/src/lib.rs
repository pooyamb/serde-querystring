use std::ops::Deref;
use std::sync::Arc;

use async_trait::async_trait;
use axum_core::{
    extract::FromRequestParts,
    response::{IntoResponse, Response},
};
use http::{request::Parts, StatusCode};
use serde::de::DeserializeOwned;
use serde_querystring::de::Error;

pub use serde_querystring::de::ParseMode;

/// Axum's Query extractor, modified to use serde-querystring.
///
/// `T` is expected to implement [`serde::Deserialize`].
///
/// # Example
///
/// ```rust,no_run
/// use axum::{
///     routing::get,
///     Router,
/// };
/// use serde::Deserialize;
/// use serde_querystring_axum::QueryString;
///
/// #[derive(Deserialize)]
/// struct Pagination {
///     page: usize,
///     per_page: usize,
/// }
///
/// // This will parse query strings like `?page=2&per_page=30` into `Pagination`
/// // structs.
/// async fn list_things(pagination: QueryString<Pagination>) {
///     let pagination: Pagination = pagination.0;
///
///     // ...
/// }
///
/// let app = Router::new().route("/list_things", get(list_things));
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
///
/// If the query string cannot be parsed it will reject the request with a `422
/// Unprocessable Entity` response.
///
/// To change the default error and the parsing mode, add `QueryStringConfig` to your extensions.
///
/// ```rust,no_run
/// use axum::{Router, Extension, http::StatusCode};
/// use serde_querystring_axum::{ParseMode, QueryStringConfig};
///
/// let app = Router::new().layer(Extension(
///     QueryStringConfig::new(ParseMode::Brackets).ehandler(|err| {
///         (StatusCode::BAD_REQUEST, err.to_string()) // return type should impl IntoResponse
///     }),
/// ));
/// # async {
/// # axum::Server::bind(&"".parse().unwrap()).serve(app.into_make_service()).await.unwrap();
/// # };
/// ```
///
#[derive(Debug, Clone, Copy, Default)]
pub struct QueryString<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for QueryString<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let QueryStringConfig { mode, ehandler } = parts
            .extensions
            .get::<QueryStringConfig>()
            .cloned()
            .unwrap_or_default();

        let query = parts.uri.query().unwrap_or_default();
        let value = serde_querystring::from_str(query, mode).map_err(|e| {
            if let Some(ehandler) = ehandler {
                ehandler(e)
            } else {
                QueryStringError::default().into_response()
            }
        })?;
        Ok(QueryString(value))
    }
}

impl<T> Deref for QueryString<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct QueryStringConfig {
    mode: ParseMode,
    ehandler: Option<Arc<dyn Fn(Error) -> Response + Send + Sync>>,
}

impl Default for QueryStringConfig {
    fn default() -> Self {
        Self {
            mode: ParseMode::Duplicate,
            ehandler: None,
        }
    }
}

impl QueryStringConfig {
    pub fn new(mode: ParseMode) -> Self {
        Self {
            mode,
            ehandler: None,
        }
    }

    pub fn mode(mut self, mode: ParseMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn ehandler<F, R>(mut self, ehandler: F) -> Self
    where
        F: Fn(Error) -> R + Send + Sync + 'static,
        R: IntoResponse,
    {
        self.ehandler = Some(Arc::new(move |e| ehandler(e).into_response()));
        self
    }
}

#[derive(Debug)]
struct QueryStringError {
    status: StatusCode,
    body: String,
}

impl Default for QueryStringError {
    fn default() -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: String::from("Failed to deserialize query string"),
        }
    }
}

impl IntoResponse for QueryStringError {
    fn into_response(self) -> Response {
        (self.status, self.body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use axum::{
        body::{Body, HttpBody},
        extract::FromRequest,
        routing::get,
        Extension, Router,
    };
    use http::{Request, StatusCode};
    use serde::Deserialize;
    use tower::ServiceExt;

    use super::*;

    async fn check<T>(uri: impl AsRef<str>, value: T)
    where
        T: DeserializeOwned + PartialEq + Debug,
    {
        let req = Request::builder().uri(uri.as_ref()).body(()).unwrap();
        assert_eq!(
            QueryString::<T>::from_request(req, &()).await.unwrap().0,
            value
        );
    }

    #[tokio::test]
    async fn test_query() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Pagination {
            size: Option<u64>,
            pages: Option<Vec<u64>>,
        }

        check(
            "http://example.com/test",
            Pagination {
                size: None,
                pages: None,
            },
        )
        .await;

        check(
            "http://example.com/test?size=10",
            Pagination {
                size: Some(10),
                pages: None,
            },
        )
        .await;

        check(
            "http://example.com/test?size=10&pages=20",
            Pagination {
                size: Some(10),
                pages: Some(vec![20]),
            },
        )
        .await;

        check(
            "http://example.com/test?size=10&pages=20&pages=21&pages=22",
            Pagination {
                size: Some(10),
                pages: Some(vec![20, 21, 22]),
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_config_mode() {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Params {
            n: Vec<i32>,
        }

        async fn handler(q: QueryString<Params>) -> String {
            format!("{}-{}", q.n.get(0).unwrap(), q.n.get(2).unwrap())
        }

        let app = Router::new()
            .route("/", get(handler))
            .layer(Extension(QueryStringConfig::new(ParseMode::Brackets)));
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/?n[3]=300&n[2]=200&n[1]=100")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let (parts, mut body) = res.into_parts();

        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(body.data().await.unwrap().unwrap(), "100-300")
    }

    #[tokio::test]
    async fn correct_rejection_default() {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Params {
            n: i32,
        }

        async fn handler(_: QueryString<Params>) {}

        let app = Router::new().route("/", get(handler));
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/?n=string")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let (parts, mut body) = res.into_parts();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(
            body.data().await.unwrap().unwrap(),
            "Failed to deserialize query string"
        );
    }

    #[tokio::test]
    async fn correct_rejection_custom() {
        #[derive(Deserialize)]
        #[allow(dead_code)]
        struct Params {
            n: i32,
        }

        async fn handler(_: QueryString<Params>) {}

        let app = Router::new().route("/", get(handler)).layer(Extension(
            QueryStringConfig::default().ehandler(|_err| {
                (
                    StatusCode::BAD_GATEWAY,
                    String::from("Something went wrong"),
                )
            }),
        ));

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/?n=string")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let (parts, mut body) = res.into_parts();

        assert_eq!(parts.status, StatusCode::BAD_GATEWAY);
        assert_eq!(body.data().await.unwrap().unwrap(), "Something went wrong");
    }
}
