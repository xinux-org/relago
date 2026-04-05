#![allow(clippy::nonminimal_bool)]

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Attribute, DeriveInput, Expr, Field, Fields, Ident, Lit, Token, parse_macro_input,
    punctuated::Punctuated, spanned::Spanned,
};

#[proc_macro_derive(Persist, attributes(persist))]
pub fn persist_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    TokenStream::from(implement_members(&input).unwrap_or_else(|e| e.to_compile_error()))
}

fn implement_members(ast: &DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &ast.ident;
    let mut_struct_ident = &Ident::new(&format!("{}Mut", ident), Span::call_site());
    let syn::Data::Struct(ref data) = ast.data else {
        return Err(syn::Error::new(
            ast.ident.span(),
            "Only structs with named fields can derive Persist",
        ));
    };

    let path = &parse_path(&ast.attrs)?;
    let fields = &parse_fields(&data.fields)?;

    let mut output = quote!();

    output.extend(define_persist_mut_struct(mut_struct_ident, fields));
    output.extend(define_traits(mut_struct_ident));
    output.extend(implement_persist_load(ident, path));
    output.extend(implement_persist_edit(ident, mut_struct_ident, fields));
    output.extend(implement_persist_save(
        ident,
        mut_struct_ident,
        path,
        fields,
    ));

    Ok(output)
}

fn define_traits(mut_ident: &Ident) -> TokenStream2 {
    quote! {
        trait PersistLoad {
            fn load() -> String;
            fn parse(contents: &str) -> anyhow::Result<Self> where Self: Sized;
        }

        pub trait PersistEdit {
            fn edit(&self, f: impl FnOnce(&mut #mut_ident)) -> #mut_ident;
        }

        pub trait PersistSave {
            fn save(self) -> anyhow::Result<()>;
        }
    }
}

fn define_persist_mut_struct(ident: &Ident, fields: &Punctuated<Field, Token![,]>) -> TokenStream2 {
    let field_defs = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();
        let type_def = &field.ty;

        quote!(pub #ident: #type_def)
    });

    quote! {
        pub struct #ident {
            #(#field_defs),*
        }
    }
}

fn implement_persist_edit(
    ident: &Ident,
    mut_ident: &Ident,
    fields: &Punctuated<Field, Token![,]>,
) -> TokenStream2 {
    let field_initers = fields.iter().map(|field| {
        let ident = field.ident.as_ref().unwrap();

        quote!(#ident: cloned.#ident)
    });

    quote! {
        #[automatically_derived]
        impl PersistEdit for #ident {
            fn edit(&self, f: impl FnOnce(&mut #mut_ident)) -> #mut_ident
            {
                let cloned = self.clone();
                let mut config_mut = #mut_ident {
                    #(#field_initers),*
                };

                f(&mut config_mut);

                config_mut
            }
        }

        // FIXME: hardcoded to fix public visibility
        #[automatically_derived]
        impl #ident {
            pub fn edit(&self, f: impl FnOnce(&mut #mut_ident)) -> #mut_ident {
                <Self as PersistEdit>::edit(self, f)
            }
        }
    }
}

fn implement_persist_load(ident: &Ident, path: &String) -> TokenStream2 {
    quote! {
        #[automatically_derived]
        impl PersistLoad for #ident {
            fn parse(contents: &str) -> anyhow::Result<Self> {
                toml::from_str(contents).map_err(|e| anyhow!("{}", e))
            }

            fn load() -> String {
                std::fs::read_to_string(#path).expect("Should have been able to read the file")
            }
        }
    }
}

fn implement_persist_save(
    ident: &Ident,
    mut_ident: &Ident,
    path: &String,
    fields: &Punctuated<Field, Token![,]>,
) -> TokenStream2 {
    let field_expressions = fields.iter().map(|field| {
        let name = field.ident.as_ref().expect("msg");
        let name_quoted = format!("{}", name);

        quote! {
            doc[#name_quoted] = toml_edit::value(serde::Serialize::serialize(&self.#name, toml_edit::ser::ValueSerializer::new())?);
        }
    });

    quote! {
        #[automatically_derived]
        impl PersistSave for #mut_ident {
            fn save(self) -> anyhow::Result<()> {
                let mut doc = str::parse::<toml_edit::DocumentMut>(&#ident::load())?;

                #(#field_expressions)*

                std::fs::write(#path, doc.to_string())?;

                Ok(())
            }
        }

        // FIXME: hardcoded to fix public visibility
        #[automatically_derived]
        impl #mut_ident {
            pub fn save(self) -> anyhow::Result<()> {
                <Self as PersistSave>::save(self)
            }
        }
    }
}

fn parse_fields(fields: &Fields) -> syn::Result<Punctuated<Field, Token![,]>> {
    if let Fields::Named(fields) = fields {
        return Ok(fields.named.clone());
    }

    Err(syn::Error::new(
        fields.span(),
        "Only named struct fields is supported",
    ))
}

fn parse_path(attrs: &[Attribute]) -> syn::Result<String> {
    let Some(attr) = attrs.iter().find(|p| p.path().is_ident("persist")) else {
        return Err(syn::Error::new(
            // some result is guaranteed
            attrs.first().span().join(attrs.last().span()).unwrap(),
            "persist attribute is missing",
        ));
    };

    let mut path: Option<String> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("path") {
            let expression = meta.value()?.parse::<Expr>()?;

            return match expression {
                Expr::Lit(expression) => match expression.lit {
                    Lit::Str(val) => {
                        path = Some(val.value());
                        Ok(())
                    }
                    _ => Err(syn::Error::new(
                        expression.span(),
                        "Only strting literal expressions is supported",
                    )),
                },
                _ => Err(syn::Error::new(
                    expression.span(),
                    "Only literal expressions is supported",
                )),
            };
        }

        Err(syn::Error::new(meta.input.span(), "path is required"))
    })?;

    path.map_or_else(
        || {
            Err(syn::Error::new(
                attr.span(),
                "Cannot parse attribute: Expected #[persist(path = \"...\")]",
            ))
        },
        Ok,
    )
}
