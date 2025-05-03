use iterum_macros::versioned;

#[test]
#[should_panic(expected = "missing `semver` attribute, please add `#[versioned(semver)]`")]
fn test_no_versioning_set() {
	#[versioned]
	struct User {
		field: String,
	}
}

#[test]
#[should_panic(expected = "no fields found, please add at least one field")]
fn test_no_fields() {
	#[versioned(semver)]
	struct User {}
}
