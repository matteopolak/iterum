#![allow(unused)]

use iterum::versioned;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Email(String);

#[versioned(semver, serde, attrs(serde(tag = "version")))]
#[derive(Deserialize, Serialize)]
struct User<'a, A> {
	/// The name of the user.
	username: String,
	#[versioned(until = "1.0.0")]
	email: A,
	#[versioned(since = "1.0.0")]
	email: Email,
	created_at: String,
	#[versioned(since = "2.0.0")]
	a: &'a str,
}

fn main() {
	let content: User<'_, ()> = User::V2_0_0(UserLatest {
		a: "hello",
		created_at: "2023-10-01".into(),
		email: Email("email".into()),
		username: "username".into(),
	});

	println!("{}", serde_json::to_string_pretty(&content).unwrap());
}
