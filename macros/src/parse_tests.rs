use crate::parse::{Definition, Field, Input, Single, Value};
use syn::parse_quote;

#[test]
fn input() {
    let input: Input = parse_quote! {::tock_registers
        //!A
        ///B
        status: u8 { Read },
    };
    let expected = Input {
        tock_registers: parse_quote!(::tock_registers),
        attributes: vec![parse_quote!(#![doc = r"A"])],
        definitions: parse_quote!(#[doc = r"B"] status: u8 { Read },),
    };
    assert_eq!(input, expected);
}

#[test]
fn definition() {
    let definition: Definition = parse_quote! {
        ///A
        buttons: [u8; 4] { Read }
    };
    let expected = Definition {
        attributes: vec![parse_quote!(#[doc = r"A"])],
        name: parse_quote!(buttons),
        value: Value::Single(Single {
            colon: parse_quote!(:),
            type_spec: parse_quote!([u8; 4]),
            operations: Some(parse_quote!({ Read })),
        }),
    };
    assert_eq!(definition, expected);
    let definition: Definition = parse_quote! {
        ///A
        foo {
            //!B
            ///C
            0x0 => nested_foo: simple_foo,
        }
    };
    let expected = Definition {
        attributes: vec![parse_quote!(#[doc = r"A"])],
        name: parse_quote!(foo),
        value: Value::Block {
            brace: Default::default(),
            attributes: vec![parse_quote!(#![doc = r"B"])],
            fields: parse_quote!(#[doc = r"C"] 0x0 => nested_foo: simple_foo,),
        },
    };
    assert_eq!(definition, expected);
}

#[test]
fn field() {
    let field: Field = parse_quote!([0x1, 0x2] => control: u8 { Read, Write });
    let expected = Field {
        attributes: vec![],
        offsets: parse_quote!([0x1, 0x2]),
        arrow: parse_quote!(=>),
        name: parse_quote!(control),
        single: Single {
            colon: parse_quote!(:),
            type_spec: parse_quote!(u8),
            operations: Some(parse_quote!({ Read, Write })),
        },
    };
    assert_eq!(field, expected);
    let field: Field = parse_quote!(0 => exact_array: [u16; u8] {});
    let expected = Field {
        attributes: vec![],
        offsets: parse_quote!(0),
        arrow: parse_quote!(=>),
        name: parse_quote!(exact_array),
        single: Single {
            colon: parse_quote!(:),
            type_spec: parse_quote!([u16; u8]),
            operations: Some(parse_quote!({})),
        },
    };
    assert_eq!(field, expected);
    let field: Field = parse_quote!(0 => reference: mod_path);
    let expected = Field {
        attributes: vec![],
        offsets: parse_quote!(0),
        arrow: parse_quote!(=>),
        name: parse_quote!(reference),
        single: Single {
            colon: parse_quote!(:),
            type_spec: parse_quote!(mod_path),
            operations: None,
        },
    };
    assert_eq!(field, expected);
}
