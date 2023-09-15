use proc_macro2::TokenStream;
use quote::*;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::{braced, Expr, Ident, ItemStruct, LitStr, Token, Type};

pub struct RequestBuilderInput {
	/// Name of request
	name: Ident,
	/// An optional field of the struct to serialize
	serialize: Option<Expr>,
	/// The equals token
	_eq: Token!(=),

	/// The request method (e.g. Method::POST)
	request_method: Ident,

	/// How the request is formated
	format: LitStr,
	/// The named fields in the earlier formatted string
	fields: Vec<Ident>,

	/// (if not a get request) the return type of the request
	return_ty: Option<Type>,
}

impl Parse for RequestBuilderInput {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(RequestBuilderInput {
			name: input.parse()?,
			serialize: {
				if input.peek(Token!(as)) {
					let _as: Token!(as) = input.parse()?;
					let content;
					let _ = braced!(content in input);
					let x = content.parse::<Expr>()?;

					Some(x)
				} else {
					None
				}
			},
			return_ty: {
				if input.peek(Token!(return)) {
					let _return: Token!(return) = input.parse()?;

					Some(input.parse()?)
				} else {
					None
				}
			},
			_eq: input.parse()?,
			request_method: input.parse()?,

			format: input.parse()?,
			fields: if input.peek(Token!(as)) {
				let _as: Token!(as) = input.parse()?;

				let fields = input.parse_terminated::<Ident, Token!(,)>(Ident::parse)?;
				fields.into_iter().collect()
			} else {
				Vec::new()
			},
		})
	}
}
impl RequestBuilderInput {
	pub fn process(self, input: ItemStruct) -> TokenStream {
		let fn_name = format_ident!("{}_{}", self.request_method.to_string().to_lowercase(), self.name);
		let fields = self.fields;
		let format = self.format;

		let decleration = quote! {pub async fn #fn_name<'a>};
		let endpoint = quote! {
			let mut endpoint = DiscordClient::API.to_string();
			endpoint.push_str(&format!(#format, #(#fields = #fields),*));
		};

		let stream = match self.request_method.to_string().as_str() {
			"GET" => {
				let return_value = if let Some(return_ty) = &self.return_ty {
					quote!(#return_ty)
				} else {
					quote!(Self)
				};
				quote! {
					#decleration (client: &'a mut DiscordClient, #(#fields: impl std::fmt::Display),*) -> Result<#return_value, NetError>{
						#endpoint
						let response = client.request(&endpoint, String::new(), hyper::Method::GET).await?;
						serde_json::from_str(&response).map_err(|e| NetError::DeJson(e, response))
					}
				}
			}
			request => {
				let request_name = format_ident!("{}", request);

				let serialize = self.serialize.map_or(quote! {self}, |f| quote! {#f});

				// We automatically return a string if there is no return type
				let return_stmt = if let Some(return_ty) = &self.return_ty {
					quote! {serde_json::from_str::<#return_ty>(&response).map_err(|e| NetError::DeJson(e, response.to_string()))}
				} else {
					quote! { Ok(response) }
				};

				// Handle the above for the function signiture
				let return_ty = if let Some(return_ty) = &self.return_ty {
					quote_spanned! {return_ty.span() => #return_ty}
				} else {
					quote! {String}
				};

				// Add the function
				quote! {
					#decleration (&self, client: &'a mut DiscordClient, #(#fields: impl std::fmt::Display),*) -> Result<#return_ty, NetError>{
						#endpoint
						let response = client.request(&endpoint, serde_json::to_string(#serialize).map_err(|e|NetError::DeJson(e, format!("{:?}", #serialize)))?, hyper::Method::#request_name).await?;
						#return_stmt
					}
				}
			}
		};

		let ident = &input.ident;
		let generics = &input.generics;
		quote! {
			#input
			impl #ident #generics{
				#stream
			}
		}
	}
}
