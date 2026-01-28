// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use super::register_definition;
use crate::ast::{Definition, Field, FieldDef};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Ident, Path};

/// Generates the module for a register block.
pub fn generate(tock_registers: &Path, definition: &Definition, fields: &[Field]) -> TokenStream {
    let docs = &definition.docs;
    let cfgs = &definition.cfgs;
    let visibility = &definition.visibility;
    let name = &definition.name;
    let mut interface_fields = TokenStream::new();
    let mut bus_bounds = TokenStream::new();
    let num_fields = fields.len();
    let mut bus_offsets = TokenStream::new();
    let buses = &definition.buses;
    let mut block_info_cfgs = TokenStream::new();
    let mut block_info_sizes: Vec<_> = (0..buses.len()).map(|_| TokenStream::new()).collect();
    let mut block_info_array_lens = TokenStream::new();
    let mut offset_tests = TokenStream::new();
    let mut interface_bounds = TokenStream::new();
    let mut interface_impl_items = TokenStream::new();
    let mut real_structs = TokenStream::new();
    for (field_idx, field) in fields.iter().enumerate() {
        let docs = &field.docs;
        let cfgs = &field.cfgs;
        block_info_cfgs.extend(quote![#tock_registers::internal::core::cfg!(all(#(#cfgs),*)),]);
        let (name, register) = match &field.field_def {
            FieldDef::Padding(sizes) => {
                for (bus_idx, bus) in buses.iter().enumerate() {
                    let size = &sizes[bus_idx];
                    block_info_sizes[bus_idx].extend(quote![#size,]);
                    let offset = &field.offsets[bus_idx];
                    offset_tests.extend(quote![#(#[cfg(#cfgs)])*]);
                    offset_tests.extend(quote_spanned! {offset.span()=>
                        assert!(#offset == <#bus as Bus>::BLOCK_INFO.offsets[#field_idx],
                            "offset mismatch");
                    });
                }
                block_info_array_lens.extend(quote![&[],]);
                continue;
            }
            FieldDef::Register { name, definition } => (name, definition),
        };
        let element_type = &register.element_type;
        let mut interface_bound;
        let mut real;
        let size;
        if let Some(operations) = &register.operations {
            interface_bound =
                quote![#tock_registers::Register<DataType = #element_type> #(+ #operations)*];
            let real_name = Ident::new(&format!("real_{name}"), name.span());
            real = quote![#real_name<B>];
            let bus_trait = quote![#tock_registers::DataTypeBus<#element_type>];
            bus_bounds.extend(quote![+ #bus_trait]);
            size = quote![<Self as #bus_trait>::PADDED_SIZE];
            real_structs.extend(register_definition(
                tock_registers,
                cfgs,
                &real_name,
                register,
                operations,
            ));
        } else {
            interface_bound = quote![#element_type::Interface];
            real = quote![#element_type::Real<B>];
            bus_bounds.extend(quote![+ #element_type::Bus]);
            size = quote![<#element_type::Real<Self> as #tock_registers::Block>::SIZE];
        };
        for array_size in &register.array_sizes {
            interface_bound = quote![#tock_registers::RegisterArray<Element: #interface_bound>];
            real = quote![#tock_registers::RealRegisterArray<#real, #array_size>];
        }
        interface_fields.extend(quote! {
            #(#docs)*
            #(#[cfg(#cfgs)])*
            type #name: #interface_bound;
            #(#docs)*
            #(#[cfg(#cfgs)])*
            fn #name(self) -> Self::#name;
        });
        let name_offset = Ident::new(&format!("{name}_offset"), name.span());
        bus_offsets.extend(quote! {
            #(#[cfg(#cfgs)])*
            const #name_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[#field_idx];
        });
        for (bus_idx, bus) in buses.iter().enumerate() {
            block_info_sizes[bus_idx].extend(quote![#size,]);
            let offset = &field.offsets[bus_idx];
            offset_tests.extend(quote![#(#[cfg(#cfgs)])*]);
            offset_tests.extend(quote_spanned! {offset.span()=>
                assert!(#offset == <#bus as Bus>::#name_offset, "offset mismatch");
            });
        }
        let array_lens = &register.array_sizes;
        block_info_array_lens.extend(quote![&[#(#array_lens),*],]);
        // TODO: Can the interface bounds for array fields be simplified?
        interface_bounds.extend(quote![#real: #interface_bound,]);
        interface_impl_items.extend(quote! {
            #(#docs)*
            #(#[cfg(#cfgs)])*
            type #name = #real;
            #(#docs)*
            #(#[cfg(#cfgs)])*
            fn #name(self) -> Self::#name {
                unsafe {
                    Self::#name::new(self.0.byte_add(<B as Bus>::#name_offset))
                }
            }
        });
    }
    quote! {
        #(#docs)*
        #(#cfgs)*
        #visibility mod #name {
            #![allow(non_camel_case_types)]
            use super::*;
            pub trait Interface: #tock_registers::internal::core::marker::Copy {
                #interface_fields
            }
            #[allow(non_upper_case_globals)]
            pub trait Bus: #tock_registers::Address #bus_bounds + sealed::Bus {
                const BLOCK_INFO: #tock_registers::internal::BlockInfo<#num_fields>;
                #bus_offsets
            }
            #(
                impl Bus for #buses {
                    const BLOCK_INFO: #tock_registers::internal::BlockInfo<#num_fields> =
                        #tock_registers::internal::BlockInfo::new(
                            [#block_info_cfgs], [#block_info_sizes], [#block_info_array_lens]);
                }
                impl sealed::Bus for #buses {}
            )*
            const _: () = { #offset_tests };
            mod sealed { pub trait Bus {} }
            pub struct Real<B: Bus>(B);
            impl<B: Bus> Real<B> {
                pub const unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> #tock_registers::internal::core::clone::Clone for Real<B> {
                fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> #tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> Interface for Real<B> where #interface_bounds {
                #interface_impl_items
            }
            impl<B: Bus> #tock_registers::Block for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::BLOCK_INFO.block_size;
                unsafe fn new(address: B) -> Self { Self(address) }
            }
            #real_structs
        }
    }
}
