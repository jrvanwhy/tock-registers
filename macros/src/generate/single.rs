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
    let is_scalar = register.array_sizes.is_empty();
    let is_definition = register.operations.is_some();
    let element_type = &register.element_type;
    let docs = &definition.docs;
    let cfgs = &definition.cfgs;
    let visibility = &definition.visibility;
    let name = &definition.name;
    let element_bound;
    let bus_bound;
    let buses = &definition.buses;
    let element_definition;
    let mut real;
    if let Some(operations) = &register.operations {
        // This RegisterSpec is a register definition.
        element_bound =
            quote![#tock_registers::Register<DataType = #element_type> #(+ #operations)*];
        bus_bound = quote![#tock_registers::DataTypeBus<#element_type>];
        let struct_name = match is_scalar {
            true => "Real",
            false => "Element",
        };
        element_definition = register_definition(
            tock_registers,
            &[],
            &Ident::new(struct_name, Span::call_site()),
            register,
            operations,
        );
        real = quote![Element<B>];
    } else {
        // This RegisterSpec is a register reference.
        element_bound = quote![#element_type::Interface];
        bus_bound = quote![#element_type::Bus];
        element_definition = quote![];
        real = quote![#element_type::Real<B>];
    }
    let mut interface_bound = element_bound.clone();
    for size in &register.array_sizes {
        interface_bound = quote![#tock_registers::RegisterArray<Element: #interface_bound>];
        real = quote![#tock_registers::RealRegisterArray<#real, #size>];
    }
    let real_alias = match (is_scalar, is_definition) {
        (true, true) => quote![],
        _ => quote![pub type Real<B> = #real;],
    };
    let impl_bound_type = match (is_scalar, is_definition) {
        (true, _) => quote![Self],
        (false, true) => quote![Element<B>],
        (false, false) => quote![#element_type::Real<B>],
    };
    quote! {
        #(#docs)*
        #(#cfgs)*
        #visibility mod #name {
            use super::*;
            pub trait Interface: #interface_bound {}
            pub trait Bus: #bus_bound + sealed::Bus {}
            #(impl Bus for #buses {})*
            #(impl sealed::Bus for #buses {})*
            mod sealed { pub trait Bus {} }
            #element_definition
            #real_alias
            impl<B: Bus> Interface for Real<B> where #impl_bound_type: #element_bound {}
        }
    }
}
