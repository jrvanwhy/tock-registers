// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Input parser.

// TODO: Support both inner and outer attributes on definitions (this will require removing the
// Parse impl for Value and parsing values inside Definition::parse).
// TODO: Manually test to verify that error spans are optimal on both stable and nightly Rust.

use crate::{Definition, Field, FieldDef, Input, PerBusInt, RegisterDef, Value};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket};
use syn::{braced, bracketed, Attribute, Error, LitInt, Result, Token, TypePath};

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Input> {
        let tock_registers = input.parse()?;
        let inner_attributes = Attribute::parse_inner(input)?;
        let definitions = Punctuated::<Definition, Token![,]>::parse_terminated(input)?
            .into_iter()
            .map(|mut definition| {
                // TODO: Temporary hack to hardcode buses
                let mut attributes = inner_attributes.clone();
                attributes.append(&mut definition.attributes);
                definition.attributes = attributes;
                definition
                    .attributes
                    .retain(|attr| !attr.path().is_ident("buses"));
                definition.buses = vec![syn::parse_quote![Mmio32]];
                definition
            })
            .collect();
        Ok(Input {
            tock_registers,
            definitions,
        })
    }
}

impl Parse for Definition {
    fn parse(input: ParseStream) -> Result<Definition> {
        Ok(Definition {
            // Note: We do not attempt to handle #[buses()] attributes here. Instead, `Input`
            // handles them, as it has to combine them with the inner attributes on the macro
            // invocation itself.
            attributes: Attribute::parse_outer(input)?,
            buses: Vec::new(),
            visibility: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for Value {
    fn parse(input: ParseStream) -> Result<Value> {
        // Distinguish between a single register and block by looking at the next token after the
        // name, which should be either : (single) or { (block).
        if input.peek2(Token![:]) {
            return input.parse().map(Value::Single);
        }
        let name = input.parse()?;
        if !input.peek(Brace) {
            return Err(input.error("expected one of: `:`, `{`"));
        }
        let fields;
        braced!(fields in input);
        let fields = Punctuated::<_, Token![,]>::parse_terminated(&fields)?
            .into_iter()
            .collect();
        Ok(Value::Block { name, fields })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Field> {
        // TODO: Validate that only certain attributes (doc, cfg, cfg_attr) are present, and unit
        // test that validation.
        let attributes = Attribute::parse_outer(input)?;
        let offsets = input.parse()?;
        input.parse::<Token![=>]>()?;
        let field_def = input.parse()?;
        Ok(Field {
            attributes,
            offsets,
            field_def,
        })
    }
}

impl Parse for FieldDef {
    fn parse(input: ParseStream) -> Result<FieldDef> {
        if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            input.parse::<Token![:]>()?;
            return input.parse().map(FieldDef::Padding);
        }
        input.parse().map(FieldDef::Register)
    }
}

impl Parse for PerBusInt {
    fn parse(input: ParseStream) -> Result<PerBusInt> {
        if !input.peek(Bracket) {
            return input.parse().map(PerBusInt::Single);
        }
        let ints;
        bracketed!(ints in input);
        let ints = Punctuated::<_, Token![,]>::parse_terminated(&ints)?;
        Ok(PerBusInt::Array(ints.into_iter().collect()))
    }
}

impl Parse for RegisterDef {
    fn parse(input: ParseStream) -> Result<RegisterDef> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        // Recursive function to parse the type specification (because syn makes it hard to consume
        // individual bracket tokens).
        fn parse_type(input: ParseStream, array_sizes: &mut Vec<LitInt>) -> Result<TypePath> {
            if !input.peek(Bracket) {
                return input.parse();
            }
            let inner;
            bracketed!(inner in input);
            let out = parse_type(&inner, array_sizes)?;
            inner.parse::<Token![;]>()?;
            array_sizes.push(inner.parse()?);
            Ok(out)
        }
        let mut array_sizes = Vec::new();
        let element_type = parse_type(input, &mut array_sizes)?;
        let operations = if input.peek(Brace) {
            let ops;
            braced!(ops in input);
            let ops = Punctuated::<_, Token![,]>::parse_terminated(&ops)?;
            Some(ops.into_iter().collect())
        } else if element_type.qself.is_some() {
            // TODO: Do we need Error::new_spanned here, or is Error::new(element_type.span()
            // possible?
            // TODO: Move up into Input::parse so we can emit multiple errors?
            let err = Error::new_spanned(element_type, "register reference must be to a module");
            return Err(err);
        } else {
            None
        };
        Ok(RegisterDef {
            name,
            element_type,
            array_sizes,
            operations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::{parse2, parse_quote, Ident};

    #[test]
    fn field_def() {
        let field: FieldDef = parse_quote![_: 3];
        assert_eq!(field, FieldDef::Padding(parse_quote![3]));
        let field: FieldDef = parse_quote![a: status];
        assert_eq!(
            field,
            FieldDef::Register(RegisterDef {
                name: parse_quote![a],
                element_type: parse_quote![status],
                array_sizes: vec![],
                operations: None,
            })
        );
    }

    #[test]
    fn per_bus_int() {
        let offsets: PerBusInt = parse_quote![0x0];
        assert_eq!(offsets, PerBusInt::Single(parse_quote![0x0]));
        let offsets: PerBusInt = parse_quote!([1, 2, 1]);
        let expected = vec![parse_quote![1], parse_quote![2], parse_quote![1]];
        assert_eq!(offsets, PerBusInt::Array(expected));
    }

    #[test]
    fn register_def() {
        let register: RegisterDef = parse_quote![a: <Foo as Bar>::Associated { Read, Write }];
        let expected_name: Ident = parse_quote![a];
        assert_eq!(register.name, expected_name);
        let expected_type: TypePath = parse_quote![<Foo as Bar>::Associated];
        assert_eq!(register.element_type, expected_type);
        assert_eq!(register.array_sizes, []);
        let expected_operations = vec![parse_quote![Read], parse_quote![Write]];
        assert_eq!(register.operations, Some(expected_operations));

        let register: RegisterDef = parse_quote![a: status];
        assert_eq!(register.name, expected_name);
        let expected_type: TypePath = parse_quote![status];
        assert_eq!(register.element_type, expected_type);
        assert_eq!(register.array_sizes, []);
        assert_eq!(register.operations, None);

        let register: RegisterDef = parse_quote![a: [[[status; 2]; 3]; 4]];
        assert_eq!(register.name, expected_name);
        let expected_type: TypePath = parse_quote![status];
        assert_eq!(register.element_type, expected_type);
        let expected_sizes = [parse_quote![2], parse_quote![3], parse_quote![4]];
        assert_eq!(register.array_sizes, expected_sizes);
        assert_eq!(register.operations, None);

        let error = parse2::<RegisterDef>(quote![a: <Foo as Bar>::Associated]).unwrap_err();
        assert!(error.to_string().contains("reference must be to a mod"));
    }
}
