# serde-querystring for actix-web

This crate provides an extractor for `serde-querystring` which can be used in place of the `actix-web::Query` extractor.

```rust
use serde::Deserialize;
use serde_querystring_actix::QueryString;

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