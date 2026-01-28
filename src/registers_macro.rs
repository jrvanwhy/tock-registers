// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

#[macro_export]
macro_rules! registers {
    {$($arguments:tt)*} => {
        $crate::reexport::registers!($crate $($arguments)*);
    }
}

/// Variation of `registers!` that defaults to only supporting the Mmio32 bus.
#[macro_export]
macro_rules! mmio32_registers {
    {$($arguments:tt)*} => {
        $crate::reexport::registers!($crate #![buses($crate::Mmio32)] $($arguments)*);
    }
}

/// Variation of `registers!` that defaults to only supporting the Mmio64 bus.
#[macro_export]
macro_rules! mmio64_registers {
    {$($arguments:tt)*} => {
        $crate::reexport::registers!($crate #![buses($crate::Mmio64)] $($arguments)*);
    }
}
