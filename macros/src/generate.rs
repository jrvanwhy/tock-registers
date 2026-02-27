// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

// TODO: This entire implementation is awful, clear it up/totally overhaul it.

use crate::{Definition, Field, FieldDef, Input, PerBusInt, RegisterDef, Value};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path};

pub fn generate(input: &Input, definition: &Definition) -> TokenStream {
    match &definition.value {
        Value::Single(
            register @ RegisterDef {
                operations: Some(ops),
                ..
            },
        ) => single_definition(&input.tock_registers, definition, register, ops),
        Value::Single(
            register @ RegisterDef {
                operations: None, ..
            },
        ) => single_reference(&input.tock_registers, definition, register),
        Value::Block { name, fields } => block(&input.tock_registers, definition, name, fields),
    }
}

fn block(tock_registers: &Path, definition: &Definition, name: &Ident, fields: &[Field]) -> TokenStream {
    let attributes = &definition.attributes;
    let visibility = &definition.visibility;
    let registers = fields.iter().filter_map(|field| match &field.field_def {
        FieldDef::Padding(_) => None,
        FieldDef::Register(register) => Some(register),
    });
    let interface_fields = registers.clone().map(|register| interface_field(tock_registers, register));
    let bus_bounds = registers.clone().map(|register| {
        let element_type = &register.element_type;
        match &register.operations {
            None => quote![#element_type::Bus],
            Some(_) => quote![#tock_registers::Bus<<#element_type as #tock_registers::DataType>::Value>],
        }
    });
    let bus_offset_defs = registers.clone().map(|register| {
        let name = Ident::new(&format!("{}_offset", register.name), register.name.span());
        quote![const #name: usize;]
    });
    // TODO: Implement offset tests.
    let bus_impls = definition.buses.iter().enumerate().map(|(i, bus)| {
        let offsets = fields.iter().filter_map(|field| {
            let FieldDef::Register(register) = &field.field_def else { return None };
            let name = Ident::new(&format!("{}_offset", register.name), register.name.span());
            let offset = match &field.offsets {
                PerBusInt::Array(offsets) => &offsets[i],
                PerBusInt::Single(offset) => offset,
            };
            Some(quote![const #name: usize = #offset;])
        });
        quote![impl Bus for #bus { #(#offsets)* }]
    });
    let buses = &definition.buses;
    quote! {
        #(#attributes)*
        #visibility mod #name {
            use super::*;
            #[allow(non_camel_case_types)]
            pub trait Interface: #tock_registers::reexport::core::marker::Copy {#(#interface_fields)*}
            #[allow(non_upper_case_globals)]
            pub trait Bus: sealed::Bus #(+ #bus_bounds)* {#(#bus_offset_defs)*}
            #(#bus_impls)*
            #(impl sealed::Bus for #buses {})*
            mod sealed {
                pub trait Bus {}
            }
            pub struct Real<B: Bus>(B);
            impl<B: Bus> #tock_registers::reexport::core::clone::Clone for Real<B> {
                fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> #tock_registers::reexport::core::marker::Copy for Real<B> {}
            // TODO(CHECKPOINT): Continue implementing register blocks (real_$name structs next).
        }
    }
}

fn interface_field(tock_registers: &Path, register: &RegisterDef) -> TokenStream {
    let name = &register.name;
    let element_type = &register.element_type;
    match &register.operations {
        None => quote! {
            type #name: #element_type::Interface;
            fn #name(self) -> Self::#name;
        },
        Some(ops) => quote! {
            type #name: #tock_registers::Register<DataType = #element_type> #(+ #ops)*;
            fn #name(self) -> Self::#name;
        },
    }
}

fn single_reference(
    tock_registers: &Path,
    definition: &Definition,
    register: &RegisterDef,
) -> TokenStream {
    let attributes = &definition.attributes;
    let visibility = &definition.visibility;
    let name = &register.name;
    let element_type = &register.element_type;
    let mut bound = quote![#element_type::Interface];
    let mut real = quote![#element_type::Real<B>];
    for size in &register.array_sizes {
        bound = quote![#tock_registers::RegisterArray<Element: #bound>];
        real = quote![#tock_registers::RealRegisterArray<#real, #size>];
    }
    let bus_bound =
        quote![#tock_registers::Bus<<#element_type as #tock_registers::DataType>::Value>];
    let buses = &definition.buses;
    quote! {
        #(#attributes)*
        #visibility mod #name {
            use super::*;
            pub trait Interface: #bound {}
            pub trait Bus: sealed::Bus + #element_type::Bus {}
            #(impl Bus for #buses {})*
            #(impl sealed::Bus for #buses {})*
            mod sealed { pub trait Bus {} }
            pub type Real<B> = #element_type::Real<B>;
            impl<B: Bus> Interface for Real<B> where Self: #bound {}
        }
    }
}

fn single_definition(
    tock_registers: &Path,
    definition: &Definition,
    register: &RegisterDef,
    operations: &[Path],
) -> TokenStream {
    let attributes = &definition.attributes;
    let visibility = &definition.visibility;
    let name = &register.name;
    let element_type = &register.element_type;
    let buses = &definition.buses;
    let mut bound = quote![#tock_registers::Register<DataType = #element_type> #(+ #operations)*];
    let mut real = quote![Element<B>];
    for size in &register.array_sizes {
        bound = quote![#tock_registers::RegisterArray<Element: #bound>];
        real = quote![#tock_registers::RealRegisterArray<#real, #size>];
    }
    let (element_struct, real_alias) = match register.array_sizes[..] {
        [] => (quote![Real], quote![]),
        _ => (quote![Element], quote![pub type Real<B> = #real;]),
    };
    let bus_bound =
        quote![#tock_registers::Bus<<#element_type as #tock_registers::DataType>::Value>];
    quote! {
        #(#attributes)*
        #visibility mod #name {
            use super::*;
            pub trait Interface: #bound {}
            pub trait Bus: sealed::Bus + #bus_bound {}
            #(impl Bus for #buses {})*
            #(impl sealed::Bus for #buses {})*
            mod sealed { pub trait Bus {} }
            pub struct #element_struct<B: Bus>(B);
            impl<B: Bus> #tock_registers::reexport::core::clone::Clone for #element_struct<B> {
                fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> #tock_registers::reexport::core::marker::Copy for #element_struct<B> {}
            impl<B: Bus> #tock_registers::Block for #element_struct<B> {
                type Address = B;
                const SIZE: usize = <B as #bus_bound>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> #tock_registers::Register for #element_struct<B> {
                type DataType = #element_type;
            }
            #(#operations!(real_impl, #element_struct, #element_type,);)*
            #real_alias
        }
    }
}
