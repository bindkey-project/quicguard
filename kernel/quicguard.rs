// SPDX-License-Identifier: GPL-2.0

//! QuicGuard: MASQUE VPN data path.

use kernel::prelude::*;

module! {
    type: QuicGuard,
    name: "quicguard",
    authors: ["QuicGuard team"],
    description: "QuicGuard MASQUE VPN data path",
    license: "GPL",
}

struct QuicGuard;

impl kernel::Module for QuicGuard {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("loaded QuicGuard\n");
        Ok(QuicGuard)
    }
}

impl Drop for QuicGuard {
    fn drop(&mut self) {
        pr_info!("unloaded QuicGuard\n");
    }
}
