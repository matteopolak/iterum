use proc_macro::TokenStream;
use syn::parse_macro_input;

mod internals;

/// A macro to generate versioned structs.
///
/// ## Examples
///
/// The following code will result in three versions of the `User` struct:
///
/// - `UserV0_0_0`
/// - `UserV1_0_0`
/// - `UserV2_0_0`
///
/// ```rust
/// use iterum_macros::versioned;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize)]
/// struct Email(String);
///
/// #[versioned(
///   // use semantic versioning
///   semver,
///   // add #[serde(rename = "...")] on the enum variants
///   // and #[serde(borrow)] where appropriate
///   serde,
///   // add attributes to the generated enum only
///   attrs(serde(tag = "version"))
/// )]
/// // add attributes to all generated structs
/// #[derive(Deserialize, Serialize)]
/// struct User<'a, A> {
///   // present on all versions
///   username: String,
///   // present on all versions until 1.0.0 (exclusive)
///   #[versioned(until = "1.0.0")]
///   email: A,
///   // present on all versions since 1.0.0 (inclusive)
///   #[versioned(since = "1.0.0")]
///   email: Email,
///   created_at: String,
///   // present on all versions since 2.0.0 (inclusive)
///   #[versioned(since = "2.0.0")]
///   a: &'a str,
/// }
/// ```
#[proc_macro_attribute]
pub fn versioned(attr: TokenStream, input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as syn::DeriveInput);
	internals::versioned::expand(attr.into(), input)
		.unwrap_or_else(syn::Error::into_compile_error)
		.into()
}
