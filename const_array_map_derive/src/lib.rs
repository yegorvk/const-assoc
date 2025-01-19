use anyhow::{anyhow, bail, Result};
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, AttrStyle, Attribute, Data, DataEnum, DeriveInput};

#[proc_macro_derive(PrimitiveEnum)]
pub fn derive_primitive_enum(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    derive(&input).unwrap().into()
}

fn derive(input: &DeriveInput) -> Result<TokenStream2> {
    let data = match &input.data {
        Data::Enum(data) => data,
        _ => bail!("`PrimitiveEnum` only applies to enums"),
    };

    let name = &input.ident;
    let repr = parse_repr_attribute(&input.attrs)?;
    let max_variants = max_discriminant(data)? + 1;

    Ok(quote! {
        unsafe impl ::const_array_map::PrimitiveEnum for #name {
            type Layout = ::const_array_map::PrimitiveEnumLayout<#repr, #max_variants>;
        }
    })
}

fn parse_repr_attribute(attrs: &[Attribute]) -> Result<EnumRepr> {
    let attr = attrs
        .iter()
        .find(|attr| {
            let matching_ident = attr
                .path()
                .get_ident()
                .map(|ident| *ident == "repr")
                .unwrap_or(false);

            matching_ident && matches!(attr.style, AttrStyle::Outer)
        })
        .ok_or(anyhow!("enum must have a primitive representation"))?;

    let repr_ident = attr.meta.require_list()?.parse_args::<Ident>()?;

    let repr_str = repr_ident.to_string();

    let repr = match repr_str.as_str() {
        "u8" => EnumRepr::U8,
        "u16" => EnumRepr::U16,
        "u32" => EnumRepr::U32,
        "u64" => EnumRepr::U64,
        "usize" => EnumRepr::USize,
        _ => bail!("`{repr_str}` is not a supported primitive enum representation"),
    };

    Ok(repr)
}

fn max_discriminant(enum_: &DataEnum) -> Result<usize> {
    for variant in &enum_.variants {
        if variant.discriminant.is_some() {
            bail!("enums that have variants with explicitly set discriminants are not supported");
        }
    }

    Ok(enum_.variants.len() - 1)
}

enum EnumRepr {
    U8,
    U16,
    U32,
    U64,
    USize,
}

impl ToTokens for EnumRepr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let token_stream = match self {
            EnumRepr::U8 => quote! { u8 },
            EnumRepr::U16 => quote! { u16 },
            EnumRepr::U32 => quote! { u32 },
            EnumRepr::U64 => quote! { u64 },
            EnumRepr::USize => quote! { usize },
        };

        token_stream.to_tokens(tokens);
    }
}
