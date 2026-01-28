// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use super::*;
use crate::test_util::assert_tokens_eq;
use syn::parse_quote;

// TODO: Add cfg/attributes and doc comments to these test cases.

#[test]
fn empty() {
    let input = parse_quote! {
        ::tock_registers
        #[buses(Mmio32, Mmio64)]
        pub foo {}
    };
    let expected = quote! {
        pub mod foo {
            #![allow(non_camel_case_types)]
            use super::*;
            pub trait Interface: ::tock_registers::internal::core::marker::Copy {}
            #[allow(non_upper_case_globals)]
            pub trait Bus: ::tock_registers::Address + sealed::Bus {
                const BLOCK_INFO: ::tock_registers::internal::BlockInfo<0usize>;
            }
            impl Bus for Mmio32 {
                const BLOCK_INFO: ::tock_registers::internal::BlockInfo<0usize>
                    = ::tock_registers::internal::BlockInfo::new([], [], []);
            }
            impl sealed::Bus for Mmio32 {}
            impl Bus for Mmio64 {
                const BLOCK_INFO: ::tock_registers::internal::BlockInfo<0usize>
                    = ::tock_registers::internal::BlockInfo::new([], [], []);
            }
            impl sealed::Bus for Mmio64 {}
            const _: () = {};
            mod sealed { pub trait Bus {} }
            pub struct Real<B: Bus>(B);
            impl<B: Bus> Real<B> {
                pub const unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone for Real<B> {
                fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> Interface for Real<B> where {}
            impl<B: Bus> ::tock_registers::Block for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::BLOCK_INFO.block_size;
                unsafe fn new(address: B) -> Self { Self(address) }
            }
        }
    };
    assert_tokens_eq(generate(input), expected);
}

#[test]
fn various() {
    let input = parse_quote! {
        ::tock_registers
        #[buses(Mmio32, Mmio64)]
        pub foo {
            0 => scalar_definition: u8 { Read, Write },
            1 => _: 1,
            2 => array_definition: [[u16; 2]; 3] { Read, Write },
            14 => scalar_reference: status,
            15 => array_reference: [[ctrl; 2]; 3],
            21 => _: 3,
            24 => variable_size: usize { Read },
            [28, 32] => variable_pos: u32 { Read },
            [32, 36] => _: [4, 0],
            36 => final_fixed_pos: u32 { Read },
        }
    };
    let expected = quote! {
        pub mod foo {
            #![allow(non_camel_case_types)]
            use super::*;
            pub trait Interface: ::tock_registers::internal::core::marker::Copy {
                type scalar_definition: ::tock_registers::Register<DataType = u8> + Read + Write;
                fn scalar_definition(self) -> Self::scalar_definition;
                type array_definition: ::tock_registers::RegisterArray<
                    Element: ::tock_registers::RegisterArray<
                        Element: ::tock_registers::Register<DataType = u16> + Read + Write
                > >;
                fn array_definition(self) -> Self::array_definition;
                type scalar_reference: status::Interface;
                fn scalar_reference(self) -> Self::scalar_reference;
                type array_reference: ::tock_registers::RegisterArray<
                    Element: ::tock_registers::RegisterArray<Element: ctrl::Interface>
                >;
                fn array_reference(self) -> Self::array_reference;
                type variable_size: ::tock_registers::Register<DataType = usize> + Read;
                fn variable_size(self) -> Self::variable_size;
                type variable_pos: ::tock_registers::Register<DataType = u32> + Read;
                fn variable_pos(self) -> Self::variable_pos;
                type final_fixed_pos: ::tock_registers::Register<DataType = u32> + Read;
                fn final_fixed_pos(self) -> Self::final_fixed_pos;
            }
            #[allow(non_upper_case_globals)]
            pub trait Bus: ::tock_registers::Address + ::tock_registers::DataTypeBus<u8> +
                ::tock_registers::DataTypeBus<u16> + status::Bus + ctrl::Bus +
                ::tock_registers::DataTypeBus<usize> + ::tock_registers::DataTypeBus<u32> +
                ::tock_registers::DataTypeBus<u32> + sealed::Bus
            {
                const BLOCK_INFO: ::tock_registers::internal::BlockInfo<10usize>;
                const scalar_definition_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[0usize];
                const array_definition_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[2usize];
                const scalar_reference_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[3usize];
                const array_reference_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[4usize];
                const variable_size_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[6usize];
                const variable_pos_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[7usize];
                const final_fixed_pos_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[9usize];
            }
            impl Bus for Mmio32 {
                const BLOCK_INFO: ::tock_registers::internal::BlockInfo<10usize> =
                    ::tock_registers::internal::BlockInfo::new(
                        [::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),],
                        [<Self as ::tock_registers::DataTypeBus<u8> >::PADDED_SIZE,
                         1,
                         <Self as ::tock_registers::DataTypeBus<u16> >::PADDED_SIZE,
                         <status::Real<Self> as ::tock_registers::Block>::SIZE,
                         <ctrl::Real<Self> as ::tock_registers::Block>::SIZE,
                         3,
                         <Self as ::tock_registers::DataTypeBus<usize> >::PADDED_SIZE,
                         <Self as ::tock_registers::DataTypeBus<u32> >::PADDED_SIZE,
                         4,
                         <Self as ::tock_registers::DataTypeBus<u32> >::PADDED_SIZE,],
                        [&[], &[], &[2, 3], &[], &[2, 3], &[], &[], &[], &[], &[],]);
            }
            impl sealed::Bus for Mmio32 {}
            impl Bus for Mmio64 {
                const BLOCK_INFO: ::tock_registers::internal::BlockInfo<10usize> =
                    ::tock_registers::internal::BlockInfo::new(
                        [::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),
                         ::tock_registers::internal::core::cfg!(all()),],
                        [<Self as ::tock_registers::DataTypeBus<u8> >::PADDED_SIZE,
                         1,
                         <Self as ::tock_registers::DataTypeBus<u16> >::PADDED_SIZE,
                         <status::Real<Self> as ::tock_registers::Block>::SIZE,
                         <ctrl::Real<Self> as ::tock_registers::Block>::SIZE,
                         3,
                         <Self as ::tock_registers::DataTypeBus<usize> >::PADDED_SIZE,
                         <Self as ::tock_registers::DataTypeBus<u32> >::PADDED_SIZE,
                         0,
                         <Self as ::tock_registers::DataTypeBus<u32> >::PADDED_SIZE,],
                        [&[], &[], &[2, 3], &[], &[2, 3], &[], &[], &[], &[], &[],]);
            }
            impl sealed::Bus for Mmio64 {}
            const _: () = {
                assert!(0 == <Mmio32 as Bus>::scalar_definition_offset, "offset mismatch");
                assert!(0 == <Mmio64 as Bus>::scalar_definition_offset, "offset mismatch");
                assert!(1 == <Mmio32 as Bus>::BLOCK_INFO.offsets[1usize], "offset mismatch");
                assert!(1 == <Mmio64 as Bus>::BLOCK_INFO.offsets[1usize], "offset mismatch");
                assert!(2 == <Mmio32 as Bus>::array_definition_offset, "offset mismatch");
                assert!(2 == <Mmio64 as Bus>::array_definition_offset, "offset mismatch");
                assert!(14 == <Mmio32 as Bus>::scalar_reference_offset, "offset mismatch");
                assert!(14 == <Mmio64 as Bus>::scalar_reference_offset, "offset mismatch");
                assert!(15 == <Mmio32 as Bus>::array_reference_offset, "offset mismatch");
                assert!(15 == <Mmio64 as Bus>::array_reference_offset, "offset mismatch");
                assert!(21 == <Mmio32 as Bus>::BLOCK_INFO.offsets[5usize], "offset mismatch");
                assert!(21 == <Mmio64 as Bus>::BLOCK_INFO.offsets[5usize], "offset mismatch");
                assert!(24 == <Mmio32 as Bus>::variable_size_offset, "offset mismatch");
                assert!(24 == <Mmio64 as Bus>::variable_size_offset, "offset mismatch");
                assert!(28 == <Mmio32 as Bus>::variable_pos_offset, "offset mismatch");
                assert!(32 == <Mmio64 as Bus>::variable_pos_offset, "offset mismatch");
                assert!(32 == <Mmio32 as Bus>::BLOCK_INFO.offsets[8usize], "offset mismatch");
                assert!(36 == <Mmio64 as Bus>::BLOCK_INFO.offsets[8usize], "offset mismatch");
                assert!(36 == <Mmio32 as Bus>::final_fixed_pos_offset, "offset mismatch");
                assert!(36 == <Mmio64 as Bus>::final_fixed_pos_offset, "offset mismatch");
            };
            mod sealed { pub trait Bus {} }
            pub struct Real<B: Bus>(B);
            impl<B: Bus> Real<B> {
                pub const unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone for Real<B> {
                fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> Interface for Real<B>
            where
                // TODO: Can any of these bounds be removed or simplified?
                real_scalar_definition<B>: ::tock_registers::Register<DataType = u8> + Read + Write,
                ::tock_registers::RealRegisterArray<
                    ::tock_registers::RealRegisterArray<real_array_definition<B>,2>,3>:
                        ::tock_registers::RegisterArray<Element: ::tock_registers::RegisterArray<
                            Element: ::tock_registers::Register<DataType = u16> + Read + Write> >,
                status::Real<B>: status::Interface,
                ::tock_registers::RealRegisterArray<
                    ::tock_registers::RealRegisterArray<ctrl::Real<B>,2>,3>:
                        ::tock_registers::RegisterArray<Element:
                            ::tock_registers::RegisterArray<Element: ctrl::Interface> >,
                real_variable_size<B>: ::tock_registers::Register<DataType = usize> + Read,
                real_variable_pos<B>: ::tock_registers::Register<DataType = u32> + Read,
                real_final_fixed_pos<B>: ::tock_registers::Register<DataType = u32> + Read,
            {
                type scalar_definition = real_scalar_definition<B>;
                fn scalar_definition(self) -> Self::scalar_definition {
                    unsafe {
                        Self::scalar_definition::new(
                            self.0.byte_add(<B as Bus>::scalar_definition_offset)
                        )
                    }
                }
                type array_definition = ::tock_registers::RealRegisterArray<
                    ::tock_registers::RealRegisterArray<real_array_definition<B>, 2>, 3>;
                fn array_definition(self) -> Self::array_definition {
                    unsafe {
                        Self::array_definition::new(
                            self.0.byte_add(<B as Bus>::array_definition_offset)
                        )
                    }
                }
                type scalar_reference = status::Real<B>;
                fn scalar_reference(self) -> Self::scalar_reference {
                    unsafe {
                        Self::scalar_reference::new(
                            self.0.byte_add(<B as Bus>::scalar_reference_offset)
                        )
                    }
                }
                type array_reference = ::tock_registers::RealRegisterArray<
                    ::tock_registers::RealRegisterArray<ctrl::Real<B>, 2>, 3>;
                fn array_reference(self) -> Self::array_reference {
                    unsafe {
                        Self::array_reference::new(
                            self.0.byte_add(<B as Bus>::array_reference_offset)
                        )
                    }
                }
                type variable_size = real_variable_size<B>;
                fn variable_size(self) -> Self::variable_size {
                    unsafe {
                        Self::variable_size::new(self.0.byte_add(<B as Bus>::variable_size_offset))
                    }
                }
                type variable_pos = real_variable_pos<B>;
                fn variable_pos(self) -> Self::variable_pos {
                    unsafe {
                        Self::variable_pos::new(self.0.byte_add(<B as Bus>::variable_pos_offset))
                    }
                }
                type final_fixed_pos = real_final_fixed_pos<B>;
                fn final_fixed_pos(self) -> Self::final_fixed_pos {
                    unsafe {
                        Self::final_fixed_pos::new(
                            self.0.byte_add(<B as Bus>::final_fixed_pos_offset)
                        )
                    }
                }
            }
            impl<B: Bus> ::tock_registers::Block for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::BLOCK_INFO.block_size;
                unsafe fn new(address: B) -> Self { Self(address) }
            }
            // TODO(CHECKPOINT): I was going through this expected generated code, cleaning up the
            // formatting and noting anything that I think needs further investigation/revision
            // (like the bounds on impl<...> Interface for Real<B>).
            pub struct real_scalar_definition<B: Bus>(B);
            impl<B: Bus> real_scalar_definition<B> {
                pub unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone
            for real_scalar_definition<B> {
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_scalar_definition<B> {}
            impl<B: Bus> ::tock_registers::Block for real_scalar_definition<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> ::tock_registers::Register for real_scalar_definition<B> {
                type DataType = u8;
            }
            Read!(real_impl, real_scalar_definition, u8,);
            Write!(real_impl, real_scalar_definition, u8,);
            pub struct real_array_definition<B: Bus>(B);
            impl<B: Bus> real_array_definition<B> {
                pub unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone
            for real_array_definition<B> {
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_array_definition<B> {}
            impl<B: Bus> ::tock_registers::Block for real_array_definition<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u16>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> ::tock_registers::Register for real_array_definition<B> {
                type DataType = u16;
            }
            Read!(real_impl, real_array_definition, u16,);
            Write!(real_impl, real_array_definition, u16,);
            pub struct real_variable_size<B: Bus>(B);
            impl<B: Bus> real_variable_size<B> {
                pub unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone
            for real_variable_size<B> {
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_variable_size<B> {}
            impl<B: Bus> ::tock_registers::Block for real_variable_size<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<usize>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> ::tock_registers::Register for real_variable_size<B> {
                type DataType = usize;
            }
            Read!(real_impl, real_variable_size, usize,);
            pub struct real_variable_pos<B: Bus>(B);
            impl<B: Bus> real_variable_pos<B> {
                pub unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone
            for real_variable_pos<B> {
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_variable_pos<B> {}
            impl<B: Bus> ::tock_registers::Block for real_variable_pos<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u32>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> ::tock_registers::Register for real_variable_pos<B> {
                type DataType = u32;
            }
            Read!(real_impl, real_variable_pos, u32,);
            pub struct real_final_fixed_pos<B: Bus>(B);
            impl<B: Bus> real_final_fixed_pos<B> {
                pub unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone
            for real_final_fixed_pos<B> {
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_final_fixed_pos<B> {}
            impl<B: Bus> ::tock_registers::Block for real_final_fixed_pos<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u32>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self {
                    Self(address)
                }
            }
            impl<B: Bus> ::tock_registers::Register for real_final_fixed_pos<B> {
                type DataType = u32;
            }
            Read!(real_impl, real_final_fixed_pos, u32,);
        }
    };
    assert_tokens_eq(generate(input), expected);
}
