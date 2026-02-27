// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

mod generate;
mod parse;

use generate::generate;
use proc_macro2::TokenStream;
use syn::{parse_macro_input, Attribute, Ident, LitInt, Path, TypePath, Visibility};

#[proc_macro]
pub fn registers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Input);
    TokenStream::from_iter(input.definitions.iter().map(|d| generate(&input, d))).into()
}

/// Represents the full input to the registers! procedural macro. Note that
/// `tock_registers::registers!` prepends `$crate` to the input provided by the user, so that the
/// generated code can refer to tock_registers even if the user has renamed the crate. Therefore,
/// after `tock_registers::registers!` is expanded, the full input looks like:
///
/// ```
/// tock_registers_macros::registers! {
///     ::tock_registers             // The prepended $crate
///     // TODO: Add inner attribute support + uncomment the following line.
///     // //! Global doc comment       // This doc comment should attach to everything in the macro.
///     #![buses(Mmio32)]            // Global buses attribute
///     a: u8 { Read, Write },       // A register defined by primitive type and operation list
///     /// Doc comment              // Doc comment that should attach to `b`
///     pub b: [a; 2],               // A register array that refers to another definition
///     /// Doc comment              // Doc comment that should attach to `foo`
///     #[cfg(feature = "foo")]      // cfg attribute to apply to `f`
///     pub foo {                    // Start of a register block
///         0 => c: u8 { Read },     // Field register defined by primitive type and operation list
///         1 => _: 1,               // Padding of size 1 byte
///         2 => d: a,               // Field register that refers to another definition
///         3 => e: [b; 2],          // Field array register that contains another definition
///         /// Doc comment          // Doc comment that should attach to `f`
///         #[cfg(feature = "big")]  // cfg attribute to apply to `f`
///         7 => f: [a; 256],        // ExactSizeRegister field
///     }
/// }
/// ```
struct Input {
    /// The $crate passed in by the registers! macro_rules macro (used to refer to the
    /// tock_registers crate).
    tock_registers: Path,
    definitions: Vec<Definition>,
}

/// An individual register or register block definition.
///
/// ```
/// tock_registers::mmio32_registers! {
///     // `a` is a Definition
///     a: u8 { Read, Write },
///
///     // `b` is a Definition, and it includes the doc comment before it.
///     /// Doc comment
///     pub b: [a; 2],
///
///     // `foo` is a Definition, and it includes the doc comment and attributes before it.
///     /// Doc comment
///     #[cfg(feature = "foo")]
///     pub foo {
///         0 => c: u8 { Read },  // Individual fields are `Field`s, not Definitions
///         1 => _: 1,
///         2 => d: a,
///         3 => e: [b; 2],
///     }
/// }
/// ```
struct Definition {
    /// Attributes that apply to this definition that are just copied from the input (doc comments
    /// and cfg attributes). During parsing, these are converted into outer attributes.
    attributes: Vec<Attribute>,
    buses: Vec<TypePath>,
    visibility: Visibility,
    value: Value,
}

/// The part of a register that begin with the register's name. For individual register
/// definitions, this starts after the visibility qualifier, and in register blocks this begins
/// after the =>.
///
/// ```
/// tock_registers::mmio32_registers! {
///     a: u8 { Read, Write },
///   //^^^^^^^^^^^^^^^^^^^^^ Value::Single
///
///     /// Doc comment
///     pub b: [a; 2],
///     //  ^^^^^^^^^ Value::Single
///
///     /// Doc comment
///     #[cfg(feature = "foo")]
///     pub foo {
///     //  ^^^^^  The Value::Block starts here, and continues through the final }
///         0 => c: u8 { Read },
///         1 => _: 1,
///         2 => d: a,
///         3 => e: [b; 2],
///     }
/// }
/// ```
enum Value {
    Block { name: Ident, fields: Vec<Field> },
    Single(RegisterDef),
}

/// An individual field definition in a register block. A Field can be padding or a register.
///
/// ```
/// tock_registers::mmio32_registers! {
///     pub foo {
///         0 => c: u8 { Read },
///       //^^^^^^^^^^^^^^^^^^^ Field
///
///         1 => _: 1,
///       //^^^^^^^^ Field (padding)
///
///         2 => d: a,
///       //^^^^^^^^^ Field
///
///         // The doc comment and cfg are also part of the field
///         /// Doc comment
///         #[cfg(feature = "big")]
///         3 => f: [a; 256],
///     }
/// }
/// ```
struct Field {
    /// Attributes that apply to this definition that are just copied from the input (doc comments
    /// and cfg attributes).
    attributes: Vec<Attribute>,
    offsets: PerBusInt,
    field_def: FieldDef,
}

/// Contents of a field.
///
/// ```
/// tock_registers::mmio32_registers! {
///     foo {
///         0 => c: u8 { Read },
///         //   ^^^^^^^^^^^^^^ FieldDef::Register
///
///         1 => _: 1,
///         //   ^^^^ FieldDef::Padding
///
///         2 => d: status,
///         //   ^^^^^^^^^ FieldDef::Register
///     }
/// }
/// ```
#[cfg_attr(test, derive(Debug, PartialEq))]
enum FieldDef {
    Padding(PerBusInt),
    Register(RegisterDef),
}

/// Per-bus integer literal. Used for both field offsets and padding sizes. This can be a single
/// value, which applies to all buses, or an array of values. The number of values in the array
/// must match the number of buses.
///
/// ```
/// tock_registers::registers! {
///     #[buses(Mmio32, Mmio64)]
///     foo {
///         0 => c: u8 { Read },
///       //^ PerBusInt::Single
///
///       //v PerBusInt::Single
///         1 => _: 1,
///       //        ^ PerBusInt::Single
///
///         2 => d: usize { Read, Write },
///       //^ PerBusInt::Single
///
///         [6, 10] => e: u8 { Read },
///       //^^^^^^^ PerBusInt::Array
///
///       //vvvvvvv PerBusInt::Array
///         [7, 11] => _: [4, 0],
///       //              ^^^^^^ PerBusInt::Array
///     }
/// }
/// ```
#[cfg_attr(test, derive(Debug, PartialEq))]
enum PerBusInt {
    Array(Vec<LitInt>),
    Single(LitInt),
}

/// A single register definition. This can be its own top-level definition or the definition within
/// a register block (in which case, this is only the tokens after the `=>`).
///
/// ```
/// tock_registers::registers! {
///     pub status: u8 { Read, Write },
///     //  ^^^^^^^^^^^^^^^^^^^^^^^^^^ RegisterDef
///     pub d: [status; 2],
///     //  ^^^^^^^^^^^^^^^ RegisterDef
///     pub foo {
///         0x0 => ctrl: u8 { Read, Write },
///         //     ^^^^^^^^^^^^^^^^^^^^^^^^ RegisterDef
///
///         0x1 => _: 1,  // Padding is NOT a RegisterDef
///
///         0x2 => pins: [u8; 2] { Read, Write },
///         //     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RegisterDef
///     }
/// }
/// ```
#[cfg_attr(test, derive(Debug, PartialEq))]
struct RegisterDef {
    name: Ident,

    /// element_type can be a primitive type (for register definitions with operation lists) or a
    /// path to another register definition (for register references). If the register type
    /// specification is an array, this is the innermost type (i.e. element_type does not mention
    /// that it is an array).
    element_type: TypePath,
    /// The array sizes. If this register definition is a nested array, the sizes are listed from
    /// the innermost array to the outermost. For example, `[[[u8; 2]; 3]; 4]` would have sizes
    /// list `[2, 3, 4]`.
    array_sizes: Vec<LitInt>,

    // Operations, if this defines the element type. If register_type's element type references a
    // different register definition, this will be None.
    operations: Option<Vec<Path>>,
}
