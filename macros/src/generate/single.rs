// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use super::register_definition;
use crate::ast::{Definition, RegisterSpec};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Path};

pub fn generate(
    tock_registers: &Path,
    definition: &Definition,
    register: &RegisterSpec,
) -> TokenStream {
    match &register.operations {
        None => reference(tock_registers, definition, register),
        Some(operations) => single_definition(tock_registers, definition, register, operations),
    }
}

pub fn reference(
    tock_registers: &Path,
    definition: &Definition,
    register: &RegisterSpec,
) -> TokenStream {
    let docs = &definition.docs;
    let cfgs = &definition.cfgs;
    let visibility = &definition.visibility;
    let name = &definition.name;
    let element_type = &register.element_type;
    let element_bound = quote![#element_type::Interface];
    let mut bound = element_bound.clone();
    let mut real = quote![#element_type::Real<B>];
    for size in &register.array_sizes {
        bound = quote![#tock_registers::RegisterArray<Element: #bound>];
        real = quote![#tock_registers::RealRegisterArray<#real, #size>];
    }
    let impl_bound_type = match register.array_sizes[..] {
        [] => quote![Self],
        _ => quote![#element_type::Real<B>],
    };
    let buses = &definition.buses;
    quote! {
        #(#docs)*
        #(#cfgs)*
        #visibility mod #name {
            use super::*;
            pub trait Interface: #bound {}
            pub trait Bus: #element_type::Bus + sealed::Bus {}
            #(impl Bus for #buses {})*
            #(impl sealed::Bus for #buses {})*
            mod sealed { pub trait Bus {} }
            pub type Real<B> = #real;
            impl<B: Bus> Interface for Real<B> where #impl_bound_type: #element_bound {}
        }
    }
}

pub fn single_definition(
    tock_registers: &Path,
    definition: &Definition,
    register: &RegisterSpec,
    operations: &[Path],
) -> TokenStream {
    let docs = &definition.docs;
    let cfgs = &definition.cfgs;
    let visibility = &definition.visibility;
    let name = &definition.name;
    let element_type = &register.element_type;
    let buses = &definition.buses;
    let element_bound =
        quote![#tock_registers::Register<DataType = #element_type> #(+ #operations)*];
    let mut bound = element_bound.clone();
    let mut real = quote![Element<B>];
    for size in &register.array_sizes {
        bound = quote![#tock_registers::RegisterArray<Element: #bound>];
        real = quote![#tock_registers::RealRegisterArray<#real, #size>];
    }
    let (struct_name, real_alias, impl_bound_type) = match register.array_sizes[..] {
        [] => ("Real", quote![], quote![Self]),
        _ => (
            "Element",
            quote![pub type Real<B> = #real;],
            quote![Element<B>],
        ),
    };
    let real_struct = register_definition(
        tock_registers,
        &Ident::new(struct_name, Span::call_site()),
        register,
        operations,
    );
    let bus_bound =
        quote![#tock_registers::Bus<<#element_type as #tock_registers::DataType>::Value>];
    quote! {
        #(#docs)*
        #(#cfgs)*
        #visibility mod #name {
            use super::*;
            pub trait Interface: #bound {}
            pub trait Bus: #bus_bound + sealed::Bus {}
            #(impl Bus for #buses {})*
            #(impl sealed::Bus for #buses {})*
            mod sealed { pub trait Bus {} }
            #real_struct
            #real_alias
            impl<B: Bus> Interface for Real<B> where #impl_bound_type: #element_bound {}
        }
    }
}
