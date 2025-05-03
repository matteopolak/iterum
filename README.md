# Iterum

[![<https://img.shields.io/crates/v/iterum>](https://img.shields.io/crates/v/iterum)](https://crates.io/crates/iterum)
[![<https://img.shields.io/docsrs/iterum>](https://img.shields.io/docsrs/iterum)](https://docs.rs/iterum/latest/iterum/)
[![ci status](https://github.com/matteopolak/iterum/workflows/ci/badge.svg)](https://github.com/matteopolak/iterum/actions)

A utility macro for supporting multiple versions of a struct.

```rust
use iterum::versioned;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Email(String);

#[versioned(semver, serde, attrs(serde(tag = "version")))]
#[derive(Deserialize, Serialize)]
struct User<'a, A> {
  username: String,
  #[versioned(until = "1.0.0")]
  email: A,
  #[versioned(since = "1.0.0")]
  email: Email,
  created_at: String,
  #[versioned(since = "2.0.0")]
  a: &'a str,
}
```

`iterum` will expand this to the following code:

```rust
use iterum::versioned;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Email(String);

#[derive(Deserialize, Serialize)]
struct UserV0_0_0<A> {
  username: String,
  email: A,
  created_at: String,
}

#[derive(Deserialize, Serialize)]
struct UserV1_0_0 {
  username: String,
  email: Email,
  created_at: String,
}

#[derive(Deserialize, Serialize)]
struct UserV2_0_0<'a> {
  username: String,
  email: Email,
  created_at: String,
  a: &'a str,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "version")]
enum User<'a, A> {
  #[serde(rename = "0.0.0")]
  V0_0_0(UserV0_0_0<A>),
  #[serde(rename = "1.0.0")]
  V1_0_0(UserV1_0_0),
  #[serde(rename = "2.0.0")]
  V2_0_0(#[serde(borrow)] UserV2_0_0<'a>),
}

type UserLatest<'a> = UserV2_0_0<'a>;
```
