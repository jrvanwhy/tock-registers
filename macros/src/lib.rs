// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

mod parse;
#[cfg(test)]
mod parse_tests;
mod validation;

use quote::quote;
use syn::{parse_macro_input, Attribute, Ident, Path};

#[proc_macro]
pub fn registers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // At a high level, registers! operates in three steps:
    // 1. Parsing: Implemented in the `parse` module. The input is parsed into a tree of `syn`
    //    types. This parsing is relatively loose: inputs that match the macro's syntax but not its
    //    semantics will still succeed. Parsing ends on the first error encountered.
    // 2. Validation: Implemented in the `validation` module. Performs validation checks to verify
    //    the input is invalid, and extracts semantic meaning from the parsed tree. When validation
    //    encounters an error, it is usually able to recover and continue, allowing it to output
    //    multiple errors in a single macro invocation.
    // 3. Code generation: Generates the output code.
    let _input = parse_macro_input!(input as parse::Input);
    quote! {}.into()
}

/// The full validated input to a registers! invocation.
struct Input {
    /// The $crate passed in by the registers! macro_rules macro (used to refer to the
    /// tock_registers crate).
    pub tock_registers: Path,
    pub definitions: Vec<Definition>,
}

/// An individual register or register block definition.
struct Definition {
    /// Attributes that apply to this definition that are just copied from the input (doc comments
    /// and cfg attributes). These have been converted into outer attributes.
    pub attributes: Vec<Attribute>,
    /// Bus adapters that apply to this definition. None if no `#[bus_adapters]` attribute applies
    /// to this definition.
    pub bus_adapters: Option<Vec<Path>>,
    pub name: Ident,
    pub value: Value,
}

enum Value {
    Block(Vec<Field>),
    Single(Single),
}

struct Field {}

struct Single {}
