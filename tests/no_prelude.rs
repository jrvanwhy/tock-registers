// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

#![no_implicit_prelude]

::tock_registers::registers! {
    // An individual register, which can be re-used later.
    // Roughly equivalent to (from tock registers v1):
    //     type Status = ReadOnly<u8>;
    status: u8 { Read },

    // A register array, which can be re-used later.
    // Roughly equivalent to (from tock registers v1):
    //     type Buttons = [ReadOnly<u8>; 4];
    buttons: [u8; 4] { Read },

    // Arrays can be created by referring to other registers too.
    // Roughly equivalent to (from tock registers v1):
    //     type StatusArray = [Status; 4];
    status_array: [status; 4],

    // A register block. This is the equivalent to register_structs!
    simple_foo {
        // This defines a field called simple_status, whose type is defined
        // above as `status` (i.e. this is a read-only u8).
        0x0 => simple_status: status,

        // This defines a field called simple_buttons, whose type is the
        // register array `buttons`.
        0x1 => simple_buttons: buttons,

        // This defines a register field inline. Most registers will be defined
        // this way.
        0x5 => control: u16 { Read, Write },
    },

    // A larger register block. This register block contains an instance of a
    // simple_foo inside of itself.
    complex_foo {
        // The nested simple_foo
        0x0 => nested_foo: simple_foo,

        // An array of status registers.
        0x7 => status_array: [status; 2],
    },

    // Arrays can be created out of register blocks as well.
    many_simple_foos: [simple_foo; 8],

    // All of the above are MMIO-only (and do not support LiteX). Types that
    // want to support other uses need to explicitly declare what use cases they
    // support:
    #[bus_adapters(litex_registers::C8B32, litex_registers::C32B32)]
    litex_foo {
        0x0 => control: u16 { Read, Write },
        // Different offsets for different bus adapters.
        [0x8, 0x4] => status: u8 { Read },
    },

    #[bus_adapters(riscv_csrs::CSR)]
    csr_foo {
        // Different operation names are used here, because CSRs support
        // different operations than MMIO registers. For example, they have
        // read_and_set_bits and read_and_clear_bits operations. These operation
        // names are traits, so the CSR traits are separate from the MMIO
        // traits.
        0x0 => example_csr: u32 { CsrRead, CsrWrite },
    },

    // Oh, and to keep the example's size limited, I don't show this, but:
    // types should be able to refer to types defined in other crates/modules.
    // I.e. simple_foo could be defined in one crate, which the crate that
    // defines complex_foo depends on.
}
