use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    path::{Path, PathBuf},
};

use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::{Ident, Span};
use quote::quote;
use scraper::{Html, Selector};
use syn::{LitStr, Token};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
};

#[proc_macro]
pub fn form_struct_(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    form_struct(input)
}
#[proc_macro]
pub fn form_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(input as Args);
    // TODO
    let (_html_path, html_source) = match load_html(&args.path) {
        Ok(data) => data,
        Err(err) => return err,
    };

    let doc = Html::parse_document(&html_source);
    let Some(form) = Selector::parse(&args.form.value())
        .ok()
        .and_then(|sel| doc.select(&sel).next())
    else {
        return syn::Error::new_spanned(&args.form, "form selector did not match any elements")
            .to_compile_error()
            .into();
    };

    let mut enums = BTreeMap::<String, BTreeSet<String>>::new();
    let mut fields = BTreeMap::<String, FieldInfo>::new();
    let mut errors = Vec::new();
    let form_selector = args.form.value();

    collect_input_fields(&form, &form_selector, &mut enums, &mut fields, &mut errors);

    collect_select_fields(&form, &form_selector, &mut enums, &mut fields, &mut errors);

    if !errors.is_empty() {
        let mut tokens = proc_macro2::TokenStream::new();
        for err in errors {
            tokens.extend(err.to_compile_error());
        }
        return tokens.into();
    }

    let mut enum_defs = Vec::new();
    let mut field_defs = Vec::new();

    for (name, info) in fields {
        match &info.kind {
            FieldKind::Scalar { ty } => {
                let field_ident = ident_for_field(&name);
                let field_ty = wrap_optional(ty.clone(), !info.required);
                let field_attrs = serde_field_attrs(&info);
                field_defs.push(quote! { #field_attrs pub #field_ident: #field_ty });
            }

            FieldKind::Enum => {
                let enum_ident = ident_for_type(&name);
                let field_ident = ident_for_field(&name);

                let Some(values) = enums.get(&name) else {
                    errors.push(syn::Error::new(
                        Span::call_site(),
                        format!("form '{form_selector}' enum field '{name}' has no values"),
                    ));

                    continue;
                };

                if values.is_empty() {
                    errors.push(syn::Error::new(
                        Span::call_site(),
                        format!("form '{form_selector}' enum field '{name}' has no values"),
                    ));

                    continue;
                }

                let mut variant_names = BTreeMap::<String, Vec<String>>::new();
                for value in values {
                    let variant = ident_for_variant(value);
                    variant_names
                        .entry(variant.to_string())
                        .or_default()
                        .push(value.clone());
                }

                for (variant, originals) in &variant_names {
                    if originals.len() > 1 {
                        errors.push(syn::Error::new(
                            Span::call_site(),
                            format!(
                                "form '{form_selector}' enum field '{name}' has multiple values mapping to '{variant}': {}",
                                originals.join(", ")
                            ),
                        ));
                    }
                }

                let variant_defs = variant_names
                    .iter()
                    .map(|(variant, originals)| {
                        let ident = Ident::new(variant, Span::call_site());
                        let rename = LitStr::new(&originals[0], Span::call_site());
                        quote! {
                            #[serde(rename = #rename)]
                            #ident
                        }
                    })
                    .collect::<Vec<_>>();

                if variant_defs.is_empty() {
                    continue;
                }

                enum_defs.push(quote! {
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
                    pub enum #enum_ident {
                        #( #variant_defs, )*
                    }
                });

                let field_ty = wrap_optional(quote! { #enum_ident }, !info.required);
                let field_attrs = serde_field_attrs(&info);

                field_defs.push(quote! { #field_attrs pub #field_ident: #field_ty });
            }
        }
    }

    if !errors.is_empty() {
        let mut tokens = proc_macro2::TokenStream::new();
        for err in errors {
            tokens.extend(err.to_compile_error());
        }
        return tokens.into();
    }

    let ident = &args.name;
    //let html_path_literal = html_path.to_string_lossy();

    let expanded = quote! {
        //const _HTML_FORM_STRUCT_TRACK: &str = include_str!(#html_path_literal);

        #( #enum_defs )*

        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub struct #ident {
            #( #field_defs, )*
        }

    };

    expanded.into()
}

struct Args {
    path: LitStr,
    form: LitStr,
    name: Ident,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let form: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let name: Ident = input.parse()?;

        Ok(Self { path, form, name })
    }
}

#[derive(Clone)]
enum FieldKind {
    Scalar { ty: proc_macro2::TokenStream },
    Enum,
}

#[derive(Clone)]
struct FieldInfo {
    kind: FieldKind,
    required: bool,
    with: Option<LitStr>,
}

fn load_html(path: &LitStr) -> Result<(PathBuf, String), proc_macro::TokenStream> {
    let html_path = resolve_call_site_path(path.value().as_ref())?;
    match std::fs::read_to_string(&html_path) {
        Ok(source) => Ok((html_path, source)),
        Err(err) => Err(
            syn::Error::new_spanned(path, format!("failed to read html file: {err}"))
                .to_compile_error()
                .into(),
        ),
    }
}

fn resolve_call_site_path(path: &Path) -> Result<PathBuf, proc_macro::TokenStream> {
    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,

        Err(err) => {
            return Err(proc_macro::TokenStream::from(
                syn::Error::new(
                    Span::call_site(),
                    format!("CARGO_MANIFEST_DIR unavailable: {err}"),
                )
                .to_compile_error(),
            ));
        }
    };

    Ok(PathBuf::from(manifest_dir).join(path))
}

fn collect_input_fields(
    form: &scraper::ElementRef,
    form_selector: &str,
    enums: &mut BTreeMap<String, BTreeSet<String>>,
    fields: &mut BTreeMap<String, FieldInfo>,
    errors: &mut Vec<syn::Error>,
) {
    let mut checkbox_required_errors = BTreeSet::<String>::new();

    let selector = Selector::parse("input[name]").expect("selector is valid");
    for input in form.select(&selector) {
        let Some(name) = input.value().attr("name") else {
            continue;
        };

        let input_type = input.value().attr("type").unwrap_or("text");
        let multiple = input.value().attr("multiple").is_some();

        let mut required = input.value().attr("required").is_some();
        if input_type == "checkbox" && multiple && required {
            if checkbox_required_errors.insert(name.to_string()) {
                errors.push(syn::Error::new(
                    Span::call_site(),
                    format!(
                        "form '{form_selector}' checkbox group '{name}' uses required on individual inputs; this is not representable as a single field"
                    ),
                ));
            }

            required = false;
        }

        let comment = match comment_spec_for_element(&input) {
            Ok(spec) => spec,
            Err(err) => {
                let mut error = syn::Error::new(
                    Span::call_site(),
                    format!("form '{form_selector}' field '{name}' has invalid comment spec"),
                );

                error.combine(err);
                errors.push(error);

                None
            }
        };

        if input_type == "radio" || (input_type == "checkbox" && multiple) {
            let mut info = FieldInfo {
                kind: FieldKind::Enum,
                required,
                with: None,
            };

            if let Some(spec) = comment {
                apply_comment_spec(&mut info, &spec);
            }

            fields
                .entry(name.to_string())
                .and_modify(|existing| merge_field_info(existing, info.clone()))
                .or_insert(info);

            let values = enums.entry(name.to_string()).or_default();

            if let Some(value) = input.value().attr("value") {
                values.insert(value.to_string());
            }

            continue;
        }

        if input_type == "checkbox" {
            let mut info = FieldInfo {
                kind: FieldKind::Scalar {
                    ty: quote! { bool },
                },
                required,
                with: None,
            };

            if let Some(spec) = comment {
                apply_comment_spec(&mut info, &spec);
            }

            fields
                .entry(name.to_string())
                .and_modify(|existing| merge_field_info(existing, info.clone()))
                .or_insert(info);

            continue;
        }

        let ty = if input_type == "number" {
            quote! { i32 }
        } else {
            quote! { String }
        };

        let mut info = FieldInfo {
            kind: FieldKind::Scalar { ty },
            required,
            with: None,
        };

        if let Some(spec) = comment {
            apply_comment_spec(&mut info, &spec);
        }

        fields
            .entry(name.to_string())
            .and_modify(|existing| merge_field_info(existing, info.clone()))
            .or_insert(info);
    }
}

fn collect_select_fields(
    form: &scraper::ElementRef,
    form_selector: &str,
    enums: &mut BTreeMap<String, BTreeSet<String>>,
    fields: &mut BTreeMap<String, FieldInfo>,
    errors: &mut Vec<syn::Error>,
) {
    let selector = Selector::parse("select[name]").expect("selector is valid");
    let option_selector = Selector::parse("option").expect("selector is valid");

    for select in form.select(&selector) {
        let Some(name) = select.value().attr("name") else {
            continue;
        };

        if select.value().attr("multiple").is_some() {
            errors.push(syn::Error::new(
                Span::call_site(),
                format!(
                    "form '{form_selector}' select field '{name}' uses multiple, which is not yet supported"
                ),
            ));

            continue;
        }

        let required = select.value().attr("required").is_some();

        let comment = match comment_spec_for_element(&select) {
            Ok(spec) => spec,
            Err(err) => {
                let mut error = syn::Error::new(
                    Span::call_site(),
                    format!("form '{form_selector}' field '{name}' has invalid comment spec"),
                );

                error.combine(err);
                errors.push(error);

                None
            }
        };

        let mut info = FieldInfo {
            kind: FieldKind::Enum,
            required,
            with: None,
        };

        if let Some(spec) = comment {
            apply_comment_spec(&mut info, &spec);
        }

        fields
            .entry(name.to_string())
            .and_modify(|existing| merge_field_info(existing, info.clone()))
            .or_insert(info);

        let values = enums.entry(name.to_string()).or_default();

        for option in select.select(&option_selector) {
            if let Some(value) = option.value().attr("value") {
                values.insert(value.to_string());
            } else if let Some(text) = option.text().next() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    values.insert(trimmed.to_string());
                }
            }
        }
    }
}

fn wrap_optional(ty: proc_macro2::TokenStream, optional: bool) -> proc_macro2::TokenStream {
    if optional {
        quote! { ::core::option::Option<#ty> }
    } else {
        ty
    }
}

fn serde_field_attrs(info: &FieldInfo) -> proc_macro2::TokenStream {
    if let Some(with) = info.with.as_ref() {
        return quote! { #[serde(with = #with)] };
    }

    proc_macro2::TokenStream::new()
}

fn comment_spec_for_element(
    element: &scraper::ElementRef,
) -> Result<Option<CommentSpec>, syn::Error> {
    let mut node = element.prev_sibling();
    while let Some(sibling) = node {
        match sibling.value() {
            scraper::node::Node::Comment(comment) => {
                return parse_comment_spec(comment);
            }

            scraper::node::Node::Text(text) => {
                if text.trim().is_empty() {
                    node = sibling.prev_sibling();
                    continue;
                }
                return Ok(None);
            }

            _ => return Ok(None),
        }
    }

    Ok(None)
}

#[derive(Default)]
struct CommentSpec {
    type_override: Option<proc_macro2::TokenStream>,
    with: Option<LitStr>,
}

fn parse_comment_spec(comment: &str) -> Result<Option<CommentSpec>, syn::Error> {
    let trimmed = comment.trim();
    let Some(rest) = trimmed.strip_prefix("form_struct:") else {
        return Ok(None);
    };

    let mut spec = CommentSpec::default();

    for token in rest.split_whitespace() {
        if let Some((key, value)) = token.split_once('=') {
            let value = value.trim_matches('"');
            if key == "type" {
                let ty: syn::Type = syn::parse_str(value).map_err(|err| {
                    syn::Error::new(Span::call_site(), format!("invalid type override: {err}"))
                })?;

                spec.type_override = Some(quote! { #ty });
            } else if key == "with" && !value.is_empty() {
                spec.with = Some(LitStr::new(value, Span::call_site()));
            }
        }
    }

    Ok(Some(spec))
}

fn apply_comment_spec(info: &mut FieldInfo, spec: &CommentSpec) {
    let Some(override_ty) = &spec.type_override else {
        apply_serde_overrides(info, spec);
        return;
    };

    apply_serde_overrides(info, spec);

    info.kind = FieldKind::Scalar {
        ty: override_ty.clone(),
    };
}

fn apply_serde_overrides(info: &mut FieldInfo, spec: &CommentSpec) {
    if let Some(with) = spec.with.clone() {
        info.with = Some(with);
    }
}

fn merge_field_info(existing: &mut FieldInfo, incoming: FieldInfo) {
    existing.required |= incoming.required;

    if incoming.with.is_some() {
        existing.with = incoming.with;
    }

    if let FieldKind::Scalar { .. } = incoming.kind {
        existing.kind = incoming.kind;
    }
}

fn ident_for_field(name: &str) -> Ident {
    let field = match name.to_snake_case() {
        x if x.is_empty() => "field".to_string(),
        x if x.chars().next().is_some_and(|ch| ch.is_ascii_digit()) => format!("field_{x}"),
        x if syn::parse_str::<Ident>(&x).is_err() => format!("r#{x}"),
        x => x,
    };

    Ident::new(&field, Span::call_site())
}

fn ident_for_type(name: &str) -> Ident {
    let ty = match name.to_upper_camel_case() {
        x if x.is_empty() => "Generated".to_string(),
        x if x.chars().next().is_some_and(|ch| ch.is_ascii_digit()) => format!("T{x}"),
        x if syn::parse_str::<Ident>(&x).is_err() => format!("{x}Type"),
        x => x,
    };

    Ident::new(&ty, Span::call_site())
}

fn ident_for_variant(value: &str) -> Ident {
    let variant = match value.to_upper_camel_case() {
        x if x.is_empty() => "Unknown".to_string(),
        x if x.chars().next().is_some_and(|ch| ch.is_ascii_digit()) => format!("V{x}"),
        x if syn::parse_str::<Ident>(&x).is_err() => format!("{x}Value"),
        x => x,
    };

    Ident::new(&variant, Span::call_site())
}
