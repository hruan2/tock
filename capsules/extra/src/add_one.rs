// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
//
//! Capsule for adding one to a provided input.
//!
//! This capsule provides three commands:
//!     command 0 provides an existence check
//!     command 1 adds 1 to the provided value,
//!     command 2 adds 1 to an allowed buffer with a provided length.
//!
//! Depending on the command used, the value provided to the command syscall
//! has a different function. For command 1, it adds 1 to that value, while
//! for command 2, it is the length of the buffer.

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

use capsules_core::driver;
pub const DRIVER_NUM: usize = driver::NUM::AddOne as usize;

mod rw_allow {
    /// Writeable buffer for read buffer (copied from console.rs)
    pub const WRITE: usize = 0;
    /// The number of allow buffers the kernel stores for this grant, which
    /// should be 1 (we operate on one buffer at a time)
    pub const COUNT: u8 = 1;
}

#[derive(Default)]
pub struct App;

pub struct AddOne {
    apps: Grant<App, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,
}

impl AddOne {
    pub fn new(
        grant: Grant<App, UpcallCount<0>, AllowRoCount<0>, AllowRwCount<{ rw_allow::COUNT }>>,
    ) -> Self {
        Self { apps: grant }
    }
}

impl SyscallDriver for AddOne {
    fn command(
        &self,
        command_num: usize,
        val: usize,
        _r3: usize,
        _process_id: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),
            // convert to u64 to avoid loss from truncation, as 32-bit systems can probably support
            // u64
            1 => CommandReturn::success_u64((val as u64) + 1),
            2 => match self
                .apps
                .enter(_process_id, |_, kernel_data| {
                    let len = val;
                    // write to buffer if it is valid (check length, get_readwrite_processbuffer
                    // returns a readwrite_processbuffer of length 0 if invalid)
                    if kernel_data
                        .get_readwrite_processbuffer(rw_allow::WRITE)
                        .map_or(0, |write| write.len())
                        .min(len)
                        != 0
                    {
                        // some error matching, honestly guessed my way until rust analyzer stopped
                        // yelling at me
                        //
                        // cells are mutable from iter
                        match kernel_data
                            .get_readwrite_processbuffer(rw_allow::WRITE)
                            .map(|buf| {
                                buf.mut_enter(|piece| {
                                    piece.iter().for_each(|byte| byte.set(byte.get() + 1))
                                })
                            }) {
                            Ok(Ok(())) => Ok(()),
                            Ok(Err(e)) => Err(e),
                            Err(e) => Err(e),
                        }
                    } else {
                        Ok(())
                    }
                })
                .map_err(ErrorCode::from)
            {
                Ok(Ok(())) => CommandReturn::success(),
                Ok(Err(e)) => CommandReturn::failure(e.into()),
                Err(e) => CommandReturn::failure(e),
            },
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(process_id, |_, _| {})
    }
}
