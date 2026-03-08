// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

// TODO: Figure out #[cfg] and the trait bounds lists.
// TODO: Figure out multi-core support. Some sort of RegisterSender? Can the lifetime be on the bus
//       or does it have to be on Real<> (perhaps a BorrowedBus<'s, B: Bus>?)?

mod ast;
mod generate;
mod parse;
#[cfg(test)]
mod test_util;

use ast::Input;
use generate::generate;
use syn::parse_macro_input;

#[proc_macro]
pub fn registers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    generate(parse_macro_input!(input as Input)).into()
}
