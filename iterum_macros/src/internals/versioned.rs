use std::collections::{BTreeMap, BTreeSet, HashMap};

use darling::{
	FromDeriveInput, FromField, FromMeta, FromVariant, ast,
	usage::{GenericsExt, UsesLifetimes, UsesTypeParams},
};
use proc_macro2::TokenStream;
use quote::quote;
use semver::Version;
use syn::{Token, parse::Parser};

#[derive(Debug, FromMeta)]
struct Args {
	semver: darling::util::Flag,
	reverse: darling::util::Flag,
	serde: darling::util::Flag,
	#[darling(rename = "attrs")]
	attrs_raw: Option<syn::Meta>,

	#[darling(skip)]
	attrs: Vec<syn::Attribute>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(versioned), supports(struct_named), forward_attrs)]
struct Input {
	pub vis: syn::Visibility,
	pub ident: syn::Ident,
	pub generics: syn::Generics,
	pub data: ast::Data<Variant, Field>,
	pub attrs: Vec<syn::Attribute>,
}

#[allow(dead_code)]
#[derive(Debug, FromVariant)]
#[darling(attributes(versioned), forward_attrs)]
struct Variant {
	pub ident: syn::Ident,
	pub attrs: Vec<syn::Attribute>,
	pub fields: ast::Fields<Field>,
}

#[derive(Debug, FromField)]
#[darling(attributes(versioned), forward_attrs)]
struct Field {
	pub ident: Option<syn::Ident>,
	pub vis: syn::Visibility,
	pub ty: syn::Type,
	pub attrs: Vec<syn::Attribute>,

	pub since: Option<syn::LitStr>,
	pub until: Option<syn::LitStr>,
}

darling::uses_type_params!(&Field, ty);
darling::uses_lifetimes!(&Field, ty);

pub fn expand(args: TokenStream, input: syn::DeriveInput) -> syn::Result<TokenStream> {
	let args_ = args.clone();
	let args = ast::NestedMeta::parse_meta_list(args)?;
	let mut args = Args::from_list(&args)?;

	if let Some(attrs) = args.attrs_raw.take() {
		attrs.require_list()?;

		let syn::Meta::List(list) = attrs else {
			unreachable!("attrs should be a list");
		};

		args.attrs = syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated
			.parse2(list.tokens)?
			.into_iter()
			.map(|meta| syn::Attribute {
				meta,
				style: syn::AttrStyle::Outer,
				bracket_token: syn::token::Bracket::default(),
				pound_token: <Token![#]>::default(),
			})
			.collect();
	}

	if !args.semver.is_present() {
		return Err(syn::Error::new_spanned(
			&args_,
			"missing `semver` attribute, please add `#[versioned(semver)]`",
		));
	}

	let mut input = Input::from_derive_input(&input)?;

	match std::mem::replace(&mut input.data, ast::Data::Enum(Vec::new())) {
		ast::Data::Enum(data) => expand_enum(data, args, input),
		ast::Data::Struct(data) => expand_struct(data, args, input),
	}
}

const VERSION_ZERO: Version = Version::new(0, 0, 0);

#[allow(dead_code, unused_variables)]
fn expand_enum(data: Vec<Variant>, args: Args, input: Input) -> syn::Result<TokenStream> {
	todo!()
}

fn parse_version(lit: &Option<syn::LitStr>) -> syn::Result<Option<Version>> {
	let version = lit
		.as_ref()
		.map(|s| Version::parse(&s.value()).map_err(|e| syn::Error::new_spanned(s, e)))
		.transpose()?;

	if let (Some(version), Some(lit)) = (&version, lit.as_ref()) {
		if !version.pre.is_empty() {
			return Err(syn::Error::new_spanned(
				lit,
				"pre-release versions are not supported, open an issue if it would be useful to you",
			));
		}

		if !version.build.is_empty() {
			return Err(syn::Error::new_spanned(
				lit,
				"build metadata is not supported, open an issue if it would be useful to you",
			));
		}
	}

	Ok(version)
}

#[derive(Debug)]
struct VersionedField<'f> {
	pub field: &'f Field,
	pub since: Version,
	pub until: Option<Version>,
	pub ord: usize,
}

darling::uses_type_params!(&VersionedField<'_>, field);
darling::uses_lifetimes!(&VersionedField<'_>, field);

impl PartialOrd for VersionedField<'_> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for VersionedField<'_> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self
			.since
			.cmp(&other.since)
			.then_with(|| self.ord.cmp(&other.ord))
	}
}

impl PartialEq for VersionedField<'_> {
	fn eq(&self, other: &Self) -> bool {
		self.since == other.since && self.ord == other.ord
	}
}

impl Eq for VersionedField<'_> {}

fn version_to_ident(prefix: Option<&syn::Ident>, version: &Version) -> syn::Ident {
	let prefix = prefix.map(syn::Ident::to_string).unwrap_or_default();

	quote::format_ident!(
		"{}V{}_{}_{}",
		prefix,
		version.major,
		version.minor,
		version.patch
	)
}

fn expand_version(
	input: &Input,
	version: &Version,
	data: &VersionData<'_>,
) -> syn::Result<TokenStream> {
	let ident = version_to_ident(Some(&input.ident), version);
	let mut tokens = TokenStream::new();

	for versioned in &data.fields {
		let field = versioned.field;
		let attrs = &field.attrs;
		let vis = &field.vis;
		let ident = field.ident.as_ref().unwrap();
		let ty = &field.ty;

		tokens.extend(quote! {
			#(#attrs)*
			#vis #ident: #ty,
		});
	}

	let attrs = &input.attrs;
	let vis = &input.vis;
	let generics = &data.generics;

	Ok(quote! {
		#(#attrs)*
		#vis struct #ident #generics {
			#tokens
		}
	})
}

struct VersionData<'a> {
	fields: Vec<&'a VersionedField<'a>>,
	generics: syn::Generics,
}

fn expand_struct(data: ast::Fields<Field>, args: Args, input: Input) -> syn::Result<TokenStream> {
	let Input {
		vis,
		generics,
		ident,
		..
	} = &input;

	if data.fields.is_empty() {
		return Err(syn::Error::new(
			proc_macro2::Span::call_site(),
			"no fields found, please add at least one field",
		));
	}

	let mut fields = BTreeMap::new();

	let type_params = generics.declared_type_params();
	let lifetimes = generics.declared_lifetimes();

	for (idx, field) in data.fields.iter().enumerate() {
		let since = parse_version(&field.since)?.unwrap_or(VERSION_ZERO);
		let until = parse_version(&field.until)?;

		if let Some(until) = &until {
			if &since > until {
				return Err(syn::Error::new_spanned(
					&field.since,
					"`since` must be less than `until`",
				));
			}
		}

		fields
			.entry(since.clone())
			.or_insert_with(BTreeSet::new)
			.insert(VersionedField {
				field,
				since,
				until,
				ord: idx,
			});
	}

	let mut carry: HashMap<syn::Ident, &VersionedField<'_>> = HashMap::new();
	let mut versions: BTreeMap<Version, VersionData> = BTreeMap::new();

	for (version, fields) in &fields {
		carry.retain(|_, field| {
			if let Some(until) = &field.until {
				&field.since <= version && until > version
			} else {
				true
			}
		});

		for field in fields {
			let ident = field.field.ident.as_ref().unwrap();

			let old = if let Some(until) = &field.until {
				if until > version {
					carry.insert(ident.clone(), field)
				} else {
					None
				}
			} else {
				carry.insert(ident.clone(), field)
			};

			if let Some(old) = old {
				return Err(syn::Error::new_spanned(
					ident,
					format!(
						"duplicate field for version `{}`, other field declared since version `{}`",
						version, old.since
					),
				));
			}
		}

		let mut fields = carry.values().copied().collect::<Vec<_>>();
		let type_params =
			fields.uses_type_params(&darling::usage::Purpose::Declare.into(), &type_params);
		let lifetimes = fields.uses_lifetimes(&darling::usage::Purpose::Declare.into(), &lifetimes);

		let mut generics = input.generics.clone();

		generics.params = input
			.generics
			.params
			.iter()
			.filter(|gp| match gp {
				syn::GenericParam::Type(ty) => type_params.contains(&ty.ident),
				syn::GenericParam::Lifetime(lt) => lifetimes.contains(&lt.lifetime),
				_ => true,
			})
			.cloned()
			.collect();

		fields.sort_by(|a, b| a.ord.cmp(&b.ord));

		versions.insert(version.clone(), VersionData { fields, generics });
	}

	let structs = versions
		.iter()
		.map(|(version, data)| expand_version(&input, version, data))
		.collect::<syn::Result<Vec<_>>>()?;

	let mut variants = versions
		.iter()
		.map(|(version, data)| {
			let variant = version_to_ident(None, version);
			let struct_ = version_to_ident(Some(&input.ident), version);
			let generics = &data.generics;

			let serde_rename = if args.serde.is_present() {
				let version = version.to_string();

				quote! {
					#[serde(rename = #version)]
				}
			} else {
				quote! {}
			};

			let serde_borrow = (args.serde.is_present() && generics.lifetimes().count() > 0).then(|| {
				quote! {
					#[serde(borrow)]
				}
			});

			quote! {
				#serde_rename
				#variant(#serde_borrow #struct_ #generics)
			}
		})
		.collect::<Vec<_>>();

	if args.reverse.is_present() {
		variants.reverse();
	}

	let (latest_version, latest) = versions.last_key_value().unwrap();
	let latest_ident = version_to_ident(Some(ident), latest_version);
	let latest_type_ident = quote::format_ident!("{}Latest", ident);
	let latest_generics = &latest.generics;
	let base_attrs = &args.attrs;
	let attrs = &input.attrs;

	Ok(quote! {
		#(#structs)*

		#(#attrs)*
		#(#base_attrs)*
		#vis enum #ident #generics {
			#(#variants),*
		}

		#vis type #latest_type_ident #latest_generics = #latest_ident #latest_generics;
	})
}
