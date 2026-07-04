// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

#![forbid(unsafe_op_in_unsafe_fn)]

//! Demonstrates how a safe DMA abstraction can be developed on top of `register_map!`.

// -------------------------------------------------------------------------------------------------
// Safe DMA abstraction crate
// -------------------------------------------------------------------------------------------------

// This is obviously simplified relative to a real DMA crate, e.g. it only supports 'static
// buffers.
mod safe_dma {
    use core::cell::Cell;
    use tock_registers::{Bus, DataType, Mmio64, Register};

    pub trait UnsafeWrite: Register {
        /// # Safety: The safety invariants are hardware-specific, and depend on the exact
        /// register.
        unsafe fn set(self, value: <Self::DataType as DataType>::Value);
    }
    #[macro_export]
    macro_rules! UnsafeWrite {
        (real_impl, $name:ident, $datatype:ty,, $($rest:tt)*) => {
            impl<B: Bus + tock_registers::BusWrite<<$datatype as tock_registers::DataType>::Value>>
                UnsafeWrite for $name<B>
            {
                unsafe fn set(self, value: <$datatype as tock_registers::DataType>::Value) {
                    // Safety: The caller assured this register accessor points at a register on
                    // bus B with value type $datatype::Value that is safe to write. The code that
                    // constructed `self` guaranteed that they would avoid data races (precondition
                    // of Self::new). The caller followed the hardware's safety invariants.
                    unsafe { self.address.write(value) }
                }
            }
        };
        ($($unknown:tt)*) => {};
    }

    // Operation for a DMA enable register. Assumes that 0 == not running and 1 == enabled.
    pub trait DmaEnable: Register {
        /// Performs the fence necessary to let DMA access this buffer, then starts the DMA
        /// operation.
        ///
        /// # Safety: The address and length registers must point to a buffer which the hardware
        /// may read and write.
        unsafe fn start(self);

        /// Checks if this DMA operation is ongoing.
        fn is_running(self) -> bool;
    }
    #[macro_export]
    macro_rules! DmaEnable {
        (real_impl, $name:ident, $datatype:ty,, $($rest:tt)*) => {
            impl<
                    B: Bus
                        + $crate::safe_dma::BusDmaEnable<
                            <$datatype as tock_registers::DataType>::Value,
                        >,
                > DmaEnable for $name<B>
            {
                unsafe fn start(self) {
                    // Safety: The caller assured this register accessor points at a register on
                    // bus B with value type $datatype::Value that is safe to write. The code that
                    // constructed `self` guaranteed that they would avoid data races (precondition
                    // of Self::new). The caller guarantees that the address and length registers
                    // are set correctly.
                    unsafe { self.address.start() }
                }

                fn is_running(self) -> bool {
                    self.address.get() != 0
                }
            }
        };
        ($($unknown:tt)*) => {};
    }
    /// # Safety
    /// `get` must never return `0` unless the corresponding DMA operation has been stopped.
    pub unsafe trait BusDmaEnable<T>: Bus<T> {
        /// # Safety
        /// There must be a writable register of type T at `pointer`. The caller is responsible for
        /// avoiding data races. The DMA address and length register but point to a buffer the
        /// hardware can read and write.
        unsafe fn start(self);

        /// Gets the current value of the enable register. Note that this must be an
        /// Acquire-ordered operation, so that DMA writes are ordered-before anything the Rust code
        /// does after getting a `0` value.
        fn get(self) -> T;
    }
    /// Safety: get() must be correct.
    #[cfg(target_arch = "x86_64")]
    unsafe impl BusDmaEnable<u8> for Mmio64 {
        unsafe fn start(self) {
            let ptr = self.address().as_ptr();
            // Safety: We know this is a u8 register that we can write 1 to. This assumes the
            // register is located in memory for which DMA is cache-coherent (so no memory fence
            // instruction is required). This functions as a release fence, guaranteeing the buffer
            // is fully written before the DMA operation is started.
            #[cfg(not(miri))]
            unsafe {
                core::arch::asm!("mov BYTE PTR [{}],1", in(reg) ptr);
            }
            #[cfg(miri)]
            unsafe { &*ptr.cast::<core::sync::atomic::AtomicU8>() }
                .store(1, core::sync::atomic::Ordering::Release);
        }

        fn get(self) -> u8 {
            let val: u16;
            let ptr = self.address().as_ptr();
            // Safety: We know this points to a u8 DMA enable register, so we can read it. This
            // is an Acquire-ordered operation, so a read of 0 is sequenced after the DMA
            // operation's writes.
            #[cfg(not(miri))]
            unsafe {
                core::arch::asm!("movzx {0:x}, BYTE PTR [{1}]", out(reg) val, in(reg) ptr)
            };
            #[cfg(miri)]
            {
                use core::sync::atomic::{AtomicU8, Ordering};
                val = unsafe { &*ptr.cast::<AtomicU8>() }
                    .load(Ordering::Acquire)
                    .into();
            }
            val as u8
        }
    }

    /// Generates the DmaManager type for a peripheral.
    // This is currently hardcoded to a single DMA channel, a real DMA abstraction crate would need
    // to support multiple DMA channels.
    #[macro_export]
    macro_rules! dma_manager {
        (
            // Visibility of the DmaManager struct.
            $visibility:vis
            // The name of the DmaManager struct to be generated.
            $struct_name:ident,
            // The register map module.
            // Note: this should actually be a path, but macro_rules! macros don't seem to be able
            // to append to a path (to generate the Interface trait reference). A procedural macro
            // could do that, but for simplicity we'll just use ident for this example.
            $map:ident,
            // The name of the address register. Must have datatype *mut u8 and be UnsafeWrite.
            $address:ident,
            // The name of the length register. Must have datatype usize and be UnsafeWrite.
            $len:ident,
            // The name of the enable register. Must have datatype u8 and be DmaEnable.
            $enable:ident $(,)?
        ) => {
            $visibility struct $struct_name<R> {
                // The DMA buffer that is current in use, or None if no DMA operation is ongoing.
                // Safety invariant: If Some(), this is identical to a `&'static mut [u8]` with one
                // exception: it may alias with an ongoing operation by this DMA channel.
                buffer: core::cell::Cell<Option<core::ptr::NonNull<[u8]>>>,
                // The registers for the peripheral this DMA manager supports.
                registers: R,
            }

            impl<R: $map::Interface> $struct_name<R> {
                /// Constructs this DMA manager.
                ///
                /// # Safety
                /// `registers` must point to a valid peripheral instance. The returned struct must
                /// be the only thing that performs `unsafe` operations on this peripheral's DMA
                /// registers.
                pub unsafe fn new(registers: R) -> Self {
                    Self {
                        // Safety: This is not Some(_) so no safety invariant applies.
                        buffer: core::cell::Cell::new(None),
                        registers,
                    }
                }

                /// Returns a copy of the peripheral's register handle.
                // This example doesn't use it, but this is how the driver would get access to the
                // registers for manipulating non-DMA registers.
                #[allow(unused)]
                pub fn registers(&self) -> R {
                    self.registers
                }

                /// Starts the DMA operation with the given buffer.
                pub fn start(&self, buffer: &'static mut [u8]) {
                    let enable_reg = self.registers.$enable();
                    if enable_reg.is_running() {
                        panic!("DMA operation already ongoing");
                    }
                    let buffer = core::ptr::NonNull::from(buffer);
                    let address = buffer.as_ptr().cast();
                    let len = buffer.len();
                    let address_reg = self.registers.$address();
                    let len_reg = self.registers.$len();
                    unsafe {
                        // Safety: The DMA operation is disabled.
                        address_reg.set(address);
                        // Safety: The DMA operation is disabled.
                        len_reg.set(len);
                    }
                    // Safety: The address and len point to a static-lifetime mutable buffer. We
                    // know that when this function was called, `buffer` was the only live
                    // reference to the buffer (because it was a &mut reference), and we have
                    // converted it into a NonNull pointer, so there are no live references
                    // pointing at the buffer.
                    unsafe {
                        enable_reg.start();
                    }
                    // Safety: This was converted directly from a valid &'static mut [u8].
                    self.buffer.set(Some(buffer));
                }

                /// Returns the DMA buffer, if the DMA operation has completed.
                pub fn get_buffer(&self) -> Option<&'static mut [u8]> {
                    if self.registers.$enable().is_running() {
                        return None;
                    }
                    // Safety: We already checked the DMA operation is ongoing, so by
                    // `self.buffer`'s safety invariant nothing aliases with `b`. By
                    // `self.buffer`'s safety invariant, `b` meets all other requirements to be
                    // converted back to a `&'static mut [u8]`.
                    self.buffer.take().map(|mut b| unsafe { b.as_mut() })
                }
            }
        };
    }

    /// A fake register that implements UnsafeWrite by writing the passed value into the given
    /// Cell.
    pub struct FakeUnsafeWrite<'c, T: DataType>(&'c Cell<T::Value>);

    impl<'c, T: DataType> FakeUnsafeWrite<'c, T> {
        #[cfg_attr(not(test), allow(dead_code))]
        pub fn new(value: &'c Cell<T::Value>) -> Self {
            FakeUnsafeWrite(value)
        }
    }

    impl<'c, T: DataType> Clone for FakeUnsafeWrite<'c, T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<'c, T: DataType> Copy for FakeUnsafeWrite<'c, T> {}

    impl<'c, T: DataType> Register for FakeUnsafeWrite<'c, T> {
        type DataType = T;
    }

    impl<'c, T: DataType> UnsafeWrite for FakeUnsafeWrite<'c, T> {
        unsafe fn set(self, value: T::Value) {
            self.0.set(value)
        }
    }
}

// Publicly re-export the operations at the crate root, as is required by tock-registers.
pub use safe_dma::{DmaEnable, UnsafeWrite};

// -------------------------------------------------------------------------------------------------
// Unsafe chip crate
// -------------------------------------------------------------------------------------------------

mod chip_unsafe {
    use crate::{dma_manager, DmaEnable, UnsafeWrite};
    use tock_registers::mmio64_register_map;

    mmio64_register_map! {
        /// Register map for a DMA-based RNG. This RNG has a single DMA buffer. When DMA is
        /// initiated, it reads in the contents of the buffer to re-seed the RNG, then replaces the
        /// buffer with newly-generated random data.
        pub rng {
            /// Address of the DMA buffer.
            /// # Safety: As required by the safe_dma crate, this must not be modified while a DMA
            /// operation is ongoing.
            0 => address: *mut u8 { UnsafeWrite },
            /// Length of the DMA buffer.
            /// # Safety: As required by the safe_dma crate, this must not be modified while a DMA
            /// operation is ongoing.
            8 => len: usize { UnsafeWrite },
            /// DMA enable register.
            16 => enable: u8 { DmaEnable },
        }
    }

    dma_manager![pub RngDma, rng, address, len, enable];
}

// -------------------------------------------------------------------------------------------------
// Fake chip crate
//
// Contains fake versions of peripherals. Used by the safe chip crate's unit tests as well as test
// infrastructure in external crates (e.g. integration tests).
// -------------------------------------------------------------------------------------------------

mod chip_fake {
    use crate::{chip_unsafe::rng, safe_dma::{DmaEnable, FakeUnsafeWrite}};
    use core::{cell::Cell, slice::from_raw_parts_mut};
    use tock_registers::Register;

    pub struct FakeRng {
        address: Cell<*mut u8>,
        len: Cell<usize>,
        state: Cell<u8>,
    }

    impl<'f> rng::Interface for &'f FakeRng {
        type address = FakeUnsafeWrite<'f, *mut u8>;
        fn address(self) -> Self::address {
            FakeUnsafeWrite::new(&self.address)
        }
        type len = FakeUnsafeWrite<'f, usize>;
        fn len(self) -> Self::len {
            FakeUnsafeWrite::new(&self.len)
        }
        type enable = FakeEnable<'f>;
        fn enable(self) -> FakeEnable<'f> {
            FakeEnable(self)
        }
    }

    #[derive(Clone, Copy)]
    pub struct FakeEnable<'f>(&'f FakeRng);

    impl DmaEnable for FakeEnable<'_> {
        // This fake RNG always finishes its operation instantly, so the driver can never
        // observe it running.
        fn is_running(self) -> bool {
            false
        }

        unsafe fn start(self) {
            let buffer: &mut [u8] =
                unsafe { from_raw_parts_mut(self.0.address.get(), self.0.len.get()) };
            let mut state = self.0.state.get();
            // Seeding: sum the buffer contents as well as the state to get the new state.
            state = buffer
                .iter()
                .copied()
                .fold(0, u8::wrapping_add)
                .wrapping_add(state);
            // PRNG algorithm: increment for each output.
            for out in buffer {
                state = state.wrapping_add(1);
                *out = state;
            }
            self.0.state.set(state);
        }
    }

    impl Register for FakeEnable<'_> {
        type DataType = u8;
    }
}

// -------------------------------------------------------------------------------------------------
// Safe chip crate
// -------------------------------------------------------------------------------------------------

mod chip_safe {
    #![forbid(unsafe_code)]

    use crate::chip_unsafe::{rng, RngDma};

    /// Driver for the RNG peripheral.
    pub struct Rng<R: rng::Interface = rng::Real> {
        manager: RngDma<R>,
    }

    // A real driver would more complex than this (not just a wrapper around the DMA manager),
    // because it would do things other than a pure DMA transfer.
    impl Rng<rng::Real> {
        /// Constructs an instance of this driver with the given DMA manager.
        pub fn new(manager: RngDma<rng::Real>) -> Self {
            Rng { manager }
        }
    }

    impl<R: rng::Interface> Rng<R> {
        /// Starts filling the provided buffer with random data. The existing data in the buffer is
        /// used to seed the RNG as well.
        pub fn getrandom_start(&self, buffer: &'static mut [u8]) {
            self.manager.start(buffer)
        }

        /// Stops the getrandom operation, returning the filled buffer. If no getrandom operation
        /// was started, or one is still ongoing, returns None.
        pub fn getrandom_finish(&self) -> Option<&'static mut [u8]> {
            self.manager.get_buffer()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn rng() {
            
        }
    }
}

// -------------------------------------------------------------------------------------------------
// Example board crate
// -------------------------------------------------------------------------------------------------

fn main() {
    use core::cell::UnsafeCell;
    use core::mem::{offset_of, replace};
    use core::ptr::{null_mut, NonNull};
    use chip_safe::Rng;
    use tock_registers::Mmio64;
    use chip_unsafe::RngDma;

    // Fake version of the peripheral registers so we can demonstrate how chip_unsafe::RngDma will
    // be used for real (in a board crate).
    #[repr(C)]
    struct Fake {
        address: *mut u8,
        len: usize,
        enable: u8,
    }
    assert_eq!(offset_of!(Fake, address), 0);
    assert_eq!(offset_of!(Fake, len), 8);
    assert_eq!(offset_of!(Fake, enable), 16);

    let fake = UnsafeCell::new(Fake {
        address: null_mut(),
        len: 0,
        enable: 0,
    });
    let mmio = Mmio64::new(NonNull::new(fake.get()).unwrap().cast());
    // Safety: `Fake` correctly matches the register map, and this handle (and everything derived
    // from it) is dropped by this function before `fake` is dropped.
    let registers = unsafe { chip_unsafe::rng::Real::new(mmio) };
    // Safety: Nothing other than this DMA manager mutates `fake` (this is a safety invariant the
    // board crate can assert but no other crate can). Slight exception: we do set `enable` to 0 to
    // simulate the DMA operation completing, but we don't simulate any DMA operations after that.
    let manager = unsafe { RngDma::new(registers) };
    let driver = Rng::new(manager);

    let buffer = Box::leak(Box::new([0; 4]));
    driver.getrandom_start(buffer);

    // Verify the peripheral was configured correctly, then simulate DMA operations.
    let fake: &mut Fake = unsafe { &mut *fake.get() };
    assert_eq!(fake.enable, 1);
    assert_eq!(fake.len, 4);
    let buffer: &mut [u8; 4] = unsafe { &mut *fake.address.cast() };
    assert_eq!(replace(buffer, [1, 2, 3, 4]), [0; 4]);
    // Simulate the DMA operation ending.
    // Safety: No other thread is accessing fake.
    fake.enable = 0;

    let buffer = driver.getrandom_finish().unwrap();
    assert_eq!(buffer, [1, 2, 3, 4]);

    drop(unsafe { Box::from_raw(buffer) });
}
