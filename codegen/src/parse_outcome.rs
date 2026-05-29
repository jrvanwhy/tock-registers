// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use syn::{Error, Result};

/// `Result<T, Error>` only allows us to express two outcomes: perfect success, or immediate error.
/// However, an immediate error is a pretty harsh outcome: it stops parsing, which prevents the
/// macro from outputting more than one error at a time, and it prevents code generation, which
/// will result in many "unknown module" errors from the code that depends on the generated module.
/// Therefore, for any AST node with non-immediate errors, we parse into `Result<Outcome<T>,
/// Error>` instead. Note that because `syn::parse::Parse` always returns `Result<Self>`, we still
/// use `Result::Err` to communicate errors that should immediately stop parsing.
#[cfg_attr(test, derive(Debug))]
pub enum Outcome<T> {
    /// Full success (no errors)
    Ok(T),
    /// An error that does not stop parsing or code generation.
    #[allow(dead_code)]
    Continue(T, Error),
    /// An error that stops code generation but not parsing.
    #[allow(dead_code)]
    NoGenerate(Error),
}

/// API used to populate an Outcome. Generally, [`Parse`](syn::Parse) impls will use an
/// `Outcome<T>` to track their errors and to return early if an error prevents them from
/// generating a T (either an unrecoverable error or a NoGenerate error). On success, the Parse
/// impls will use [`success`] to attach return the Outcome with the newly-parsed value inside.
impl Outcome<()> {
    /// Constructs a new Outcome with empty contents.
    pub fn new() -> Outcome<()> {
        Outcome::Ok(())
    }

    /// Attaches new data to the Outcome and returns the new Outcome wrapped in a [`Result`]. Used
    /// at the end of [`Parse`](syn::Parse) implementations.
    pub fn success<T>(self, value: T) -> Result<Outcome<T>> {
        Ok(match self {
            Outcome::Ok(()) => Outcome::Ok(value),
            Outcome::Continue((), err) => Outcome::Continue(value, err),
            Outcome::NoGenerate(err) => Outcome::NoGenerate(err),
        })
    }
}

impl<T> Outcome<T> {
    #[cfg(all(test, not(miri)))]
    #[track_caller]
    pub fn unwrap(self) -> T
    where
        T: std::fmt::Debug,
    {
        match self {
            Outcome::Ok(val) => val,
            _ => panic!("expected Outcome::Ok, got {self:?}"),
        }
    }
}
