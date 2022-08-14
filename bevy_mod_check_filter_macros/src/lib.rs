use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(IsVariant)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = input.ident;
    let vis = input.vis.to_token_stream();

    let expanded: TokenStream = if let syn::Data::Enum(e) = input.data {
        e.variants
            .into_iter()
            .map(|v| v.ident)
            .map(|v| {
                let name = Ident::new(&format!("Is{v}"), v.span());
                quote! {
                    #vis struct #name;

                    impl<T> Predicate<T> for #name
                    where
                        T: std::ops::Deref<Target = #enum_name>,
                    {
                        fn test(test: &T) -> bool {
                            matches!(**test, #enum_name::#v)
                        }
                    }
                }
            })
            .collect()
    } else {
        panic!("IsVariant derive only works on Enums");
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}
