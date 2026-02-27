// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::{Address, Bus};

/// MMIO register bus for 32-bit systems.
#[derive(Clone, Copy)]
pub struct Mmio32(pub *mut ());

impl Address for Mmio32 {
    unsafe fn byte_add(self, offset: usize) -> Mmio32 {
        // Safety: The safety requirements of Address::byte_add require self + offset to remain
        // within this register span and not wrap.
        Mmio32(unsafe { self.0.byte_add(offset) })
    }
}

impl Bus<u8> for Mmio32 {
    const PADDED_SIZE: usize = 1;
}
impl Bus<u16> for Mmio32 {
    const PADDED_SIZE: usize = 2;
}
impl Bus<u32> for Mmio32 {
    const PADDED_SIZE: usize = 4;
}
impl Bus<u64> for Mmio32 {
    const PADDED_SIZE: usize = 8;
}
impl Bus<u128> for Mmio32 {
    const PADDED_SIZE: usize = 16;
}
impl Bus<usize> for Mmio32 {
    const PADDED_SIZE: usize = 4;
}
