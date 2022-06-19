# actix-embed

[![GitHub](https://img.shields.io/github/license/unbyte/actix-embed)](https://github.com/unbyte/actix-embed)
[![Build](https://github.com/unbyte/actix-embed/workflows/CI/badge.svg)](https://github.com/unbyte/actix-embed/actions)
[![Crates.io](https://img.shields.io/crates/v/actix-embed)](https://crates.io/crates/actix-embed)
[![Docs.rs](https://docs.rs/actix-embed/badge.svg)](https://docs.rs/actix-embed)

Serve embedded file with actix.

```rust
use actix_web::App;
use actix_embed::Embed;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "testdata/"]
struct Assets;

let app = App::new()
    .service(Embed::new("/static", &Assets));
```