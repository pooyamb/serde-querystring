# serde-querystring for axum

This crate provides an extractor for `serde-querystring` which can be used in place of the `axum::extract::Query` extractor.

```rust
use serde::Deserialize;
use serde_querystring_axum::QueryString;

#[derive(Deserialize)]
pub struct AuthRequest {
   id: u64,
   scopes: Vec<u64>,
}

// In your handler
async fn index(QueryString(info): QueryString<AuthRequest>) -> String {
    format!("Authorization request for client with id={} and type={:?}!", info.id, info.scopes)
}
```