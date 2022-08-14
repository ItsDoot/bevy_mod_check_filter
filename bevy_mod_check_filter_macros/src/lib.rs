use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
    DeriveInput, Ident, Path,
};

#[proc_macro_derive(IsVariant)]
pub fn is_variant_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = input.ident;
    let vis = input.vis.to_token_stream();

    let expanded: TokenStream = if let syn::Data::Enum(e) = input.data {
        e.variants
            .into_iter()
            .map(|v| {
                let ident = v.ident;
                let name = Ident::new(&format!("Is{ident}"), ident.span());
                let m = match v.fields {
                    syn::Fields::Unit => quote! { #enum_name::#ident },
                    _ => quote! { #enum_name::#ident { .. } },
                };

                quote! {
                    #vis struct #name;

                    impl<T> bevy_mod_check_filter::Predicate<T> for #name
                    where
                        T: bevy_mod_check_filter::Checkable<Checked = #enum_name>,
                    {
                        fn test(test: &T::Checked) -> bool {
                            match *test {
                              #m => true,
                              _ => false,
                            }
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

#[proc_macro_derive(FieldCheckable)]
pub fn field_checkable_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let vis = input.vis.to_token_stream();

    let expanded: TokenStream = if let syn::Data::Struct(s) = input.data {
        s.fields
            .into_iter()
            .filter_map(|f| {
                let ident = f.ident?;
                let ty = f.ty;
                let span = ident.span();
                let name = lens_name(struct_name.clone(), ident.clone(), span);

                Some(quote! {
                    #vis struct #name;
                    impl bevy_mod_check_filter::Checkable for #name {
                        type Component = #struct_name;
                        type Checked = #ty;

                        fn get(v: &Self::Component) -> &Self::Checked {
                            &v.#ident
                        }
                    }
                })
            })
            .collect()
    } else {
        panic!("IsVariant derive only works on Enums");
    };

    // Hand the output tokens back to the compiler
    proc_macro::TokenStream::from(expanded)
}

fn lens_name(struct_name: Ident, field_name: Ident, span: Span) -> Ident {
    Ident::new(&format!("{struct_name}Lens{field_name}"), span)
}

#[proc_macro]
pub fn lens(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let Lens { paths } = parse_macro_input!(input as Lens);

    assert!(!paths.is_empty(), "lens! macro requires at least one lens");

    let mut i = paths.into_iter();
    let first = path_to_lens_name(i.next().unwrap());
    let first = quote! { #first };

    let expanded = i.fold(first, |acc, x| {
        let x = path_to_lens_name(x);
        quote! {
             bevy_mod_check_filter::Compose<#acc, #x>
        }
    });
    proc_macro::TokenStream::from(expanded)
}

fn path_to_lens_name(p: Path) -> Ident {
    let mut v: Vec<Ident> = p.segments.into_iter().map(|s| s.ident).collect();
    assert!(v.len() == 2, "lenses must be of the form Type::field");
    let field = v.pop().unwrap();
    let ty = v.pop().unwrap();
    let span = field.span();

    lens_name(ty, field, span)
}

struct Lens {
    paths: Vec<Path>,
}
impl Parse for Lens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // input.parse::<Punctuated<Path, Comma>>()?;
        let lenses = Punctuated::<Path, Comma>::parse_terminated(input)?;
        Ok(Lens {
            paths: lenses.into_iter().collect(),
        })
    }
}
