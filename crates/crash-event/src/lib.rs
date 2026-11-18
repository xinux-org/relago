use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, spanned::Spanned, Data, DeriveInput, Field, Fields,
    Ident, LitStr, Meta, Token, Type,
};

struct MatchFilter {
    field: String,
    value: LitStr,
}

struct FieldMeta {
    journal_key: LitStr,
    required: bool,
}

#[proc_macro_derive(CrashEvent, attributes(journal))]
pub fn derive_journal_report(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    expand(&ast).unwrap_or_else(|e| e.to_compile_error()).into()
}

fn expand(ast: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &ast.ident;

    let filters = parse_struct_filters(&ast.attrs)?;
    let fields = named_fields(ast)?;

    let filter_pairs = filters.iter().map(|f| {
        let k = LitStr::new(&f.field, Span::call_site());
        let v = &f.value;
        quote! { (#k, #v) }
    });

    let mut extractions = Vec::new();
    let mut initializers = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let meta = parse_field_meta(field)?;
        let key = &meta.journal_key;

        let expr = if meta.required {
            if is_string(ty) {
                quote! { let #ident: #ty = journal.field(#key).ok_or(())?; }
            } else {
                quote! {
                    let #ident: #ty = journal
                        .field(#key)
                        .ok_or(())?
                        .parse()
                        .map_err(|_| ())?;
                }
            }
        } else if is_option_string(ty) {
            quote! { let #ident: #ty = journal.field(#key); }
        } else {
            // Option<T> where T: FromStr
            quote! {
                let #ident: #ty = journal
                    .field(#key)
                    .and_then(|s| s.parse().ok());
            }
        };

        extractions.push(expr);
        initializers.push(quote! { #ident });
    }

    Ok(quote! {
        impl #name {
            pub fn filters() -> &'static [(&'static str, &'static str)] {
                &[ #(#filter_pairs),* ]
            }

            pub fn detect(journal: &mut impl JournalExt) -> Option<Self> {
                (|| -> Result<Self, ()> {

                    #(#extractions)*
                    Ok(Self { #(#initializers),* })
                })().ok()
            }
        }
    })
}

// ---- Attribute parsing ----

fn parse_struct_filters(attrs: &[syn::Attribute]) -> syn::Result<Vec<MatchFilter>> {
    let mut out = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("journal") {
            continue;
        }
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in &nested {
            if let Meta::List(list) = meta {
                if list.path.is_ident("filter") {
                    // ← was "match"
                    let (ident, value) = list.parse_args_with(|p: syn::parse::ParseStream| {
                        let i: Ident = p.parse()?;
                        let _: Token![=] = p.parse()?;
                        let v: LitStr = p.parse()?;
                        Ok((i, v))
                    })?;
                    out.push(MatchFilter {
                        field: ident.to_string(),
                        value,
                    });
                }
            }
        }
    }
    Ok(out)
}

fn parse_field_meta(field: &Field) -> syn::Result<FieldMeta> {
    let mut journal_key: Option<LitStr> = None;
    let mut required = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("journal") {
            continue;
        }

        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

        for meta in &nested {
            match meta {
                Meta::NameValue(nv) if nv.path.is_ident("field") => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        journal_key = Some(s.clone());
                    }
                }
                Meta::Path(p) if p.is_ident("required") => {
                    required = true;
                }
                _ => {}
            }
        }
    }

    let journal_key = journal_key.ok_or_else(|| {
        syn::Error::new(
            field.span(),
            "each field needs #[journal(field = \"JOURNAL_FIELD_NAME\")]",
        )
    })?;

    Ok(FieldMeta {
        journal_key,
        required,
    })
}

// --- Helpers ---

fn named_fields(ast: &DeriveInput) -> syn::Result<&Punctuated<Field, Token![,]>> {
    match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => Ok(&f.named),
            _ => Err(syn::Error::new(
                ast.span(),
                "CrashEvent requires named fields",
            )),
        },
        _ => Err(syn::Error::new(
            ast.span(),
            "CrashEvent can only be derived for structs",
        )),
    }
}

fn is_string(ty: &Type) -> bool {
    matches!(ty, Type::Path(p) if p.path.is_ident("String"))
}

fn is_option_string(ty: &Type) -> bool {
    let Type::Path(p) = ty else { return false };
    let Some(seg) = p.path.segments.last() else {
        return false;
    };
    if seg.ident != "Option" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(inner)) = args.args.first() else {
        return false;
    };
    is_string(inner)
}
