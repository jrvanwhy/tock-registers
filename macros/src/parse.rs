// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Input parser.

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Brace;
use syn::{braced, Attribute, Expr, Ident, Path, Token, Type};

/// TODO: Visibility qualifiers.
/// TODO: Padding.

/// The parsed input to the registers! macro.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Input {
    /// The $crate passed in by the registers! macro_rules macro (used to refer to the
    /// tock_registers crate).
    pub tock_registers: Path,
    /// Attributes that apply to this input block as a whole. These are all inner attributes.
    pub attributes: Vec<Attribute>,
    /// Register definitions inside this registers! block.
    pub definitions: Punctuated<Definition, Token![,]>,
}

/// A register definition in a registers! invocation.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Definition {
    /// Attributes on this definition. These are all outer attributes.
    pub attributes: Vec<Attribute>,
    pub name: Ident,
    pub value: Value,
}

/// The value of a definition in `registers`. This can be an individual register definition or a
/// register block.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Value {
    Block {
        brace: Brace,
        /// Inner attributes that apply to this block.
        attributes: Vec<Attribute>,
        fields: Punctuated<Field, Token![,]>,
    },
    Single(Single),
}

/// A single field in a register block, including associated attributes, e.g.:
/// ```text
/// /// Doc comment for the item.
/// [0x1, 0x2] => control: u8 { Read, Write }
/// ```
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Field {
    /// Attributes on this field (such as doc comments). These are all outer attributes.
    pub attributes: Vec<Attribute>,
    /// The offset or list of offsets.
    pub offsets: Expr,
    /// The => between the offsets and name.
    pub arrow: Token![=>],
    /// Name of the field
    pub name: Ident,
    pub single: Single,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Single {
    /// The : between the name and type.
    pub colon: Token![:],
    /// The register's type specification.
    pub type_spec: Type,
    /// The operations on the register, if this is an inline register definition.
    pub operations: Option<Operations>,
}

/// A list of operations for a register/field/array, e.g. `{ Read, Write }`.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Operations {
    pub brace: Brace,
    pub operations: Punctuated<Path, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Input> {
        Ok(Input {
            tock_registers: input.parse()?,
            attributes: input.call(Attribute::parse_inner)?,
            definitions: Punctuated::parse_terminated(input)?,
        })
    }
}

impl Parse for Definition {
    fn parse(input: ParseStream) -> syn::Result<Definition> {
        Ok(Definition {
            attributes: input.call(Attribute::parse_outer)?,
            name: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for Value {
    fn parse(input: ParseStream) -> syn::Result<Value> {
        match input.lookahead1() {
            l if l.peek(Token![:]) => return Ok(Value::Single(input.parse()?)),
            l if !l.peek(Brace) => return Err(l.error()),
            _ => {}
        }
        let contents;
        Ok(Value::Block {
            brace: braced!(contents in input),
            attributes: contents.call(Attribute::parse_inner)?,
            fields: Punctuated::parse_terminated(&contents)?,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Field> {
        Ok(Field {
            attributes: input.call(Attribute::parse_outer)?,
            offsets: input.parse()?,
            arrow: input.parse()?,
            name: input.parse()?,
            single: input.parse()?,
        })
    }
}

impl Parse for Single {
    fn parse(input: ParseStream) -> syn::Result<Single> {
        Ok(Single {
            colon: input.parse()?,
            type_spec: input.parse()?,
            operations: input.peek(Brace).then(|| input.parse()).transpose()?,
        })
    }
}

impl Parse for Operations {
    fn parse(input: ParseStream) -> syn::Result<Operations> {
        let operations;
        Ok(Operations {
            brace: braced!(operations in input),
            operations: Punctuated::parse_terminated(&operations)?,
        })
    }
}
