// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

mod block;
#[cfg(test)]
mod block_tests;
mod single;
#[cfg(test)]
mod single_tests;

use crate::ast::{Input, RegisterSpec, Value};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path};

/// Returns the generated code for a registers! invocation.
pub fn generate(input: Input) -> TokenStream {
    let mut out = TokenStream::new();
    for definition in input.definitions {
        out.extend(match &definition.value {
            Value::Single(register) => {
                single::generate(&input.tock_registers, &definition, register)
            }
            Value::Block(fields) => block::generate(&input.tock_registers, &definition, fields),
        });
    }
    out
}

/// Generates the Real struct for a register definition (one that has an operations list).
/// `struct_name` is the name of the struct to generate, which may not match the name of the
/// register.
// TODO: Add support for docs and cfgs.
fn register_definition(
    tock_registers: &Path,
    struct_name: &Ident,
    register: &RegisterSpec,
    operations: &[Path],
) -> TokenStream {
    let element_type = &register.element_type;
    quote! {
        pub struct #struct_name<B: Bus>(B);
        impl<B: Bus> #struct_name<B> {
            pub unsafe fn new(address: B) -> Self { Self(address) }
        }
        impl<B: Bus> #tock_registers::internal::core::clone::Clone for #struct_name<B> {
            fn clone(&self) -> Self { *self }
        }
        impl<B: Bus> #tock_registers::internal::core::marker::Copy for #struct_name<B> {}
        impl<B: Bus> #tock_registers::Block for #struct_name<B> {
            type Address = B;
            const SIZE: usize = <B as #tock_registers::Bus<<#element_type as #tock_registers::DataType>::Value>>::PADDED_SIZE;
            unsafe fn new(address: B) -> Self {
                Self(address)
            }
        }
        impl<B: Bus> #tock_registers::Register for #struct_name<B> {
            type DataType = #element_type;
        }
        #(#operations!(real_impl, #struct_name, #element_type,);)*
    }
}
