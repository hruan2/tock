// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for the AddOne capsule.
//!
//! Capsule: capsules/extra/src/add_one.rs

use capsules_extra::add_one::AddOne;
use core::mem::MaybeUninit;
use kernel::{capabilities::MemoryAllocationCapability, component::Component};

#[macro_export]
macro_rules! add_one_component_static {
    () => {{ kernel::static_buf!(capsules_extra::add_one::AddOne) }};
}

pub type AddOneComponentType = capsules_extra::add_one::AddOne;

pub struct AddOneComponent<CAP: MemoryAllocationCapability + 'static> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mem_cap: CAP,
}

impl<CAP: MemoryAllocationCapability + 'static> AddOneComponent<CAP> {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize, mem_cap: CAP) -> Self {
        Self {
            board_kernel,
            driver_num,
            mem_cap,
        }
    }
}

impl<CAP: MemoryAllocationCapability + 'static> Component for AddOneComponent<CAP> {
    type StaticInput = &'static mut MaybeUninit<AddOne>;
    type Output = &'static AddOne;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(AddOne::new(
            self.board_kernel
                .create_grant(self.driver_num, &self.mem_cap),
        ))
    }
}
