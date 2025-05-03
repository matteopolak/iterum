pub use iterum_macros::*;

#[cfg(doctest)]
mod doctests {
	/// ```compile_fail
	/// #[iterum::versioned]
	/// struct User {
	///   field: String,
	/// }
	fn test_no_versioning_set() {}

	/// ```compile_fail
	/// #[iterum::versioned(semver)]
	/// struct User {}
	/// ```
	fn test_no_fields() {}
}

#[cfg(test)]
#[allow(unused)]
mod tests {
	use serde::{Deserialize, Serialize};

	use super::*;

	#[test]
	fn test_serde() {
		#[versioned(semver, serde)]
		#[derive(Deserialize, Serialize)]
		struct User {
			field: String,
		}

		let user = User::V0_0_0(UserV0_0_0 {
			field: "hello".into(),
		});
	}

	#[test]
	fn test_serde_borrow() {
		#[versioned(semver, serde)]
		#[derive(Deserialize, Serialize)]
		struct User<'a> {
			field: &'a str,
		}

		let user = User::V0_0_0(UserV0_0_0 { field: "hello" });
	}

	#[test]
	fn test_overwrite_field() {
		#[versioned(semver)]
		struct User {
			#[versioned(until = "1.0.0")]
			field: String,
			#[versioned(since = "1.0.0")]
			field: u32,
		}

		let v0 = User::V0_0_0(UserV0_0_0 {
			field: "hello".into(),
		});

		let v1 = User::V1_0_0(UserV1_0_0 { field: 42 });
	}

	fn test_generics() {
		#[versioned(semver)]
		struct User<'a, A> {
			#[versioned(until = "1.0.0")]
			field: &'a str,
			field2: A,
			#[versioned(since = "1.0.0")]
			field: String,
		}

		let v0 = User::V0_0_0(UserV0_0_0 {
			field: "hello",
			field2: 42,
		});

		let v1 = User::V1_0_0(UserV1_0_0 {
			field: "hello".into(),
			field2: (),
		});
	}
}
