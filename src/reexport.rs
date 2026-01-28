// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Better Bytes 2026.
// Copyright Google LLC 2024.

//! Re-exports of items from external crates, for use by tock-registers' macros.
//! Not for use by outside crates (the contents of this module are not stable).

#![doc(hidden)]

// It's possible for a crate that is not libcore to be named `core` in a calling crate. Re-export
// so registers! can reliably find libcore.
pub use core;
pub use tock_registers_macros::registers;
