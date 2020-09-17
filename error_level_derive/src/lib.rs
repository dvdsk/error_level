use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{self, spanned::Spanned, punctuated::Punctuated, Variant, token::Comma, Fields};

#[proc_macro_derive(ErrorLevel, attributes(report))]
pub fn log_level_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_error_level_macro(&ast)
}

#[derive(Debug)]
enum Level {
    No,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn from_ident(id: &syn::Ident) -> Self {
        match id.to_string().as_str() {
            "No" => Self::No,
            "Trace" => Self::Trace,
            "Debug" => Self::Debug,
            "Info" => Self::Info,
            "Warn" => Self::Warn,
            "Error" => Self::Error,
            _ => panic!("options are only: No, Trace, Debug, Info, Warn or Error"),
        }
    }
}

impl quote::ToTokens for Level {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let token = match self {
            Self::No => quote! {None},
            Self::Trace => quote! {Some(log::Level::Trace)},
            Self::Debug => quote! {Some(log::Level::Debug)},
            Self::Info => quote! {Some(log::Level::Info)},
            Self::Warn => quote! {Some(log::Level::Warn)},
            Self::Error => quote! {Some(log::Level::Error)},
        };
        tokens.extend(token);
    }
}

#[derive(Debug)]
struct Marked {
    level: Level,
    variant_id: syn::Ident,
}

fn has_level_path(m: &syn::MetaList) -> bool {
    if let Some(ident) = m.path.get_ident() {
        ident == "report"
    } else {
        false
    }
}

fn with_log_level(v: &Variant) -> Option<Level> { 
    fn unwrap_meta(n: &syn::NestedMeta) -> &syn::Meta {
        if let syn::NestedMeta::Meta(m) = n {
            return m;
        }
        panic!("nested argument list should not be a rust literal but a structured meta item");
    }
   
    for a in &v.attrs {
        let m = a.parse_meta().unwrap();
        if let syn::Meta::List(list) = m {
            if !has_level_path(&list){continue;}
            let nested = list.nested.first().unwrap();
            let meta = unwrap_meta(&nested);
            let ident = meta.path().get_ident().unwrap();
            return Some(Level::from_ident(ident));
        }
    }
    None
}

#[derive(Debug)]
struct WithInnError {
    inner_span: proc_macro2::Span,
    variant_id: syn::Ident,
}

fn is_valid_inner(ty: &syn::Type) -> Result<proc_macro2::Span, proc_macro2::Span> {
    // handle multi segment (::) paths
    fn handle_path(p: &syn::TypePath) -> proc_macro2::Span {
        let p = &p.path;
        if p.segments.len() > 1 {
            let span_begin = p.segments.first().unwrap().span();
            let span_end = p.segments.last().unwrap().span();
            span_begin.join(span_end).unwrap_or(span_end)
        } else {
            p.get_ident().span()
        }
    }

    match ty {
        syn::Type::Path(p) => Ok(handle_path(p)),
        syn::Type::Reference(r) =>
            if let syn::Type::Path(p) = &*r.elem {
                Ok(handle_path(p))
            } else {
                Err(r.span())
            },
        _ => Err(ty.span())
    }
}

fn has_inner(v: &Variant) -> Option<&syn::Type> { 
    for a in &v.attrs {
        let m = a.parse_meta().unwrap();
        if let syn::Meta::List(list) = m { 
            if has_level_path(&list){return None;}
        }
    }

    if let Fields::Unnamed(syn::FieldsUnnamed {ref unnamed, ..}) = v.fields {
        let ty = &unnamed.first()?.ty;
        Some(ty)
    } else {
        None
    }
}

fn extract_variants(variants: &Punctuated<Variant, Comma>) -> (Vec<Marked>, Vec<WithInnError>, Vec<proc_macro2::TokenStream>) {
    let mut marked = Vec::new();
    let mut w_inner = Vec::new();
    let mut errs = Vec::new();
    for v in variants {
        if let Some(level) = with_log_level(v){
            let variant_id = v.ident.clone();
            marked.push(Marked {
                level,
                variant_id
            });
        } else if let Some(inner) = has_inner(v){
            match is_valid_inner(inner) {
                Ok(inner_span) => {    
                    let variant_id = v.ident.clone();
                    w_inner.push(WithInnError {
                        inner_span,
                        variant_id
                    });
                },
                Err(span) => {
                    errs.push(quote_spanned! {
                        span =>
                        compile_error!("Needs 'report' attribute, variant content can not have an 'ErrorLevel' trait implementation");
                    });
                },
            }
        } else {
            errs.push(quote_spanned! {
                v.span() =>
                compile_error!("Needs 'report' attribute");
            })
        }
    }
    (marked, w_inner, errs)
}

fn impl_error_level_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let data = &ast.data;
    let variants = &unwrap_enum(data).variants;
    let (marked, w_inner, errs) = extract_variants(variants);

    //save list of variants with a level attribute
    let level_with_attr = marked.iter().map(|m| &m.level);
    let ident_with_attr = marked.iter().map(|m| &m.variant_id);

    //for idents without attr call the error_level function
    //if error_level is undefined for that type the user will
    let spanned = w_inner.iter().map(|m| {
        let ident = &m.variant_id;
        let span = m.inner_span;
        quote_spanned! {
            span =>
            #name::#ident(inn_err) => inn_err.error_level(),
        }
    });

    let gen = quote! {
        impl ErrorLevel for #name {
            fn error_level(&self) -> Option<log::Level> {
                #(#errs)*;
                match self {
                    #(#name::#ident_with_attr => #level_with_attr,)*
                    #(#spanned)*
                }
            }
        }
    };
    gen.into()
}

fn unwrap_enum(data: &syn::Data) -> &syn::DataEnum {
    if let syn::Data::Enum(v) = data {
        return v;
    } else {
        panic!("can only implement error level on enums");
    }
}
