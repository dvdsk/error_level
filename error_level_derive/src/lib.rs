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
enum LevelVariant {
    No,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug)]
enum Level {
    Parsed(LevelVariant),
    Error(proc_macro2::Span),
}

impl Level {
    fn from_ident(id: &syn::Ident) -> Self {
        match id.to_string().as_str() {
            "no" => Self::Parsed(LevelVariant::No),
            "trace" => Self::Parsed(LevelVariant::Trace),
            "debug" => Self::Parsed(LevelVariant::Debug),
            "info" => Self::Parsed(LevelVariant::Info),
            "warn" => Self::Parsed(LevelVariant::Warn),
            "error" => Self::Parsed(LevelVariant::Error),
            _ => Self::Error(id.span()),
        }

    }
}

impl quote::ToTokens for Level {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let token = match self {
            Self::Parsed(LevelVariant::No) => quote! {None},
            Self::Parsed(LevelVariant::Trace) => quote! {Some(log::Level::Trace)},
            Self::Parsed(LevelVariant::Debug) => quote! {Some(log::Level::Debug)},
            Self::Parsed(LevelVariant::Info) => quote! {Some(log::Level::Info)},
            Self::Parsed(LevelVariant::Warn) => quote! {Some(log::Level::Warn)},
            Self::Parsed(LevelVariant::Error) => quote! {Some(log::Level::Error)},
            Self::Error(span) => { let span = span.clone();
                quote_spanned! {span=> compile_error!("invalid report level, use: no, trace, debug, info, warn or error")
            }},
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
struct UnMarked {
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
    if let Fields::Unnamed(syn::FieldsUnnamed {ref unnamed, ..}) = v.fields {
        let ty = &unnamed.first()?.ty;
        Some(ty)
    } else {
        None
    }
}

fn extract_variants(variants: &Punctuated<Variant, Comma>)
    -> (Vec<Marked>, Vec<Marked>, Vec<UnMarked>, Vec<proc_macro2::TokenStream>) {

    let mut marked_no_inn = Vec::new();
    let mut marked_w_inn = Vec::new();
    let mut unmarked_no_inn = Vec::new();
    let mut errs = Vec::new();
    for v in variants {
        if let Some(level) = with_log_level(v){
            if let Some(_) = has_inner(v){
                let variant_id = v.ident.clone();
                marked_w_inn.push(Marked {
                    level,
                    variant_id
                });
            } else { 
                let variant_id = v.ident.clone();
                marked_no_inn.push(Marked {
                    level,
                    variant_id
                });
            }
        } else if let Some(inner) = has_inner(v){
            match is_valid_inner(inner) {
                Ok(inner_span) => {    
                    let variant_id = v.ident.clone();
                    unmarked_no_inn.push(UnMarked {
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
    (marked_no_inn, marked_w_inn, unmarked_no_inn, errs)
}

fn impl_error_level_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let data = &ast.data;
    let variants = &unwrap_enum(data).variants;
    let (marked, marked_w_inn, unmarked, errs) = extract_variants(variants);

    let marked_no_inn = marked.iter().map(|m| {
        let level = &m.level;
        let variant = &m.variant_id;
        let span = m.variant_id.span();
        quote_spanned! {
            span =>
            #name::#variant => #level,
        }
    });
    
    let marked_w_inn = marked_w_inn.iter().map(|m| {
        let level = &m.level;
        let variant = &m.variant_id;
        let span = m.variant_id.span();
        quote_spanned! {
            span =>
            #name::#variant(_) => #level,
        }
    });

    let unmarked = unmarked.iter().map(|m| {
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
                match self {
                    #(#marked_no_inn)*
                    #(#marked_w_inn)*
                    #(#unmarked)*
                }
                #(#errs)*
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
