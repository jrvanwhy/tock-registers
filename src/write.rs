// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::{Bus, DataType, Register, UIntLike};

/// A register that can be written.
pub trait Write: Register {
    fn write(self, value: <Self::DataType as DataType>::Value);
}

/// A Bus that implements BusWrite<T> can support Write implementations with DataType T. Other
/// crates (e.g. LiteX registers) can implement this on their own buses so that Write works with
/// them as well.
pub trait BusWrite<T: UIntLike>: Bus<T> {
    /// # Safety
    /// There must be a register of type T at `pointer`, and if the register itself has safety
    /// invariants (i.e. it is `UnsafeWrite`) the caller must satisfy those.
    unsafe fn write(self, value: T);
}

/// The macro that goes along with the Write trait. We don't expect this macro to be used by
/// tock_register's users, instead it is invoked by the generated code.
#[macro_export]
macro_rules! Write {
    // Provides a real implementation of the trait. The trailing $rest argument is for future
    // compatibility: it allows the procedural macro to pass additional arguments in the future
    // without breaking compatibility with this implementation of Write!.
    (real_impl, $name:ident, $datatype:ty, $($rest:tt)*) => {
        impl<B: Bus + $crate::BusWrite<<$datatype as $crate::DataType>::Value>> $crate::Write
            for $name<B>
        {
            fn write(self, value: <$datatype as $crate::DataType>::Value) {
                // Safety: The caller assured this GenericReal points at a register on bus B with
                // value type $datatype::Value that is safe to write.
                unsafe { self.0.write(value) }
            }
        }
    };
}
