use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

/// Derives [`secrets_rs::Bindable`] for a struct.
///
/// All fields whose type is `Secret<T>` (or any path ending in `Secret`) will
/// have `.bind(registry)` called on them. All other fields are ignored.
///
/// # Example
///
/// ```rust,ignore
/// use secrets_rs::{Secret, bind_all, EnvSource, SourceRegistry};
///
/// #[derive(secrets_rs::Bindable)]
/// struct Config {
///     api_key:  Secret<String>,
///     timeout:  u32,           // ignored — not a Secret
/// }
/// ```
#[proc_macro_derive(Bindable)]
pub fn derive_bindable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_bindable(input).unwrap_or_else(|e| e.to_compile_error().into())
}

fn impl_bindable(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;

    let named_fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    name,
                    "#[derive(Bindable)] requires a struct with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "#[derive(Bindable)] can only be applied to structs",
            ));
        }
    };

    let bind_calls = named_fields
        .iter()
        .filter(|f| is_secret_type(&f.ty))
        .map(|f| {
            let ident = f.ident.as_ref().unwrap();
            quote! {
                if let Err(__e) = self.#ident.bind(__registry) {
                    __errors.push(__e);
                }
            }
        });

    let expanded = quote! {
        impl ::secrets_rs::Bindable for #name {
            fn bind_secrets(
                &mut self,
                __registry: &::secrets_rs::SourceRegistry,
            ) -> ::std::result::Result<(), ::std::vec::Vec<::secrets_rs::BindError>> {
                let mut __errors: ::std::vec::Vec<::secrets_rs::BindError> =
                    ::std::vec::Vec::new();
                #(#bind_calls)*
                if __errors.is_empty() {
                    ::std::result::Result::Ok(())
                } else {
                    ::std::result::Result::Err(__errors)
                }
            }
        }
    };

    Ok(expanded.into())
}

/// Returns `true` if the type's final path segment is `Secret`.
fn is_secret_type(ty: &Type) -> bool {
    match ty {
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .map(|s| s.ident == "Secret")
            .unwrap_or(false),
        _ => false,
    }
}
