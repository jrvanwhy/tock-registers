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
