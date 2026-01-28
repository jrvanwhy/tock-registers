// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Better Bytes 2026.
// Copyright Google LLC 2024.

//! Items used by macros that are not part of tock-registers' public interface. Not for use by
//! outside crates (the contents of this module are not stable).

#![doc(hidden)]

/// It's possible for a crate that is not libcore to be named `core` in a calling crate. Re-export
/// so registers! can reliably find libcore.
pub use core;
pub use tock_registers_macros::registers;

/// Information about a register block.
///
/// BlockInfo is embedded inside the generated Bus implementation for a register block. NUM_FIELDS
/// includes padding fields as well as fields that are #[cfg]-disabled. Note that a #[cfg]-disabled
/// field may have an incorrect offset.
pub struct BlockInfo<const NUM_FIELDS: usize> {
    pub offsets: [usize; NUM_FIELDS],
    pub block_size: usize,
}

impl<const NUM_FIELDS: usize> BlockInfo<NUM_FIELDS> {
    /// Computes the block info for a register with the given field sizes. If a field is
    /// #[cfg]-disabled, its size should be specified as 0.
    pub const fn new(sizes: [usize; NUM_FIELDS]) -> BlockInfo<NUM_FIELDS> {
        let mut i = 0;
        let mut offsets = [0; NUM_FIELDS];
        let mut block_size = 0;
        while i < NUM_FIELDS {
            offsets[i] = block_size;
            block_size += sizes[i];
            i += 1;
        }
        BlockInfo {
            offsets,
            block_size,
        }
    }
}
