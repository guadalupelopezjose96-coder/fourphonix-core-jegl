// Copyright | JEGL | Jose Emanuel Guadalupe Lopez | PROPRIETARY | SPDX-License-Identifier
// SPDX-License-Identifier: LicenseRef-AuraIntelCore-Proprietary
// Copyright (c) 2026 Jose Emanuel Guadalupe Lopez (JEGL) - AuraIntelCore / Standard 1979. All Rights Reserved.
// SPDX-License-Identifier: LicenseRef-AuraIntelCore-Proprietary
// Copyright (c) 2026 Jose Emanuel Guadalupe Lopez (JEGL) - AuraIntelCore / Standard 1979. All Rights Reserved.
// PROPRIETARY & CONFIDENTIAL - WORM READ-ONLY ENFORCED (FERC 2222 / NERC CIP / IRC Sec. 5000H(d))

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let obj = out_dir.join("aura_tetrax_stub.o");
    let lib = out_dir.join("libaura_tetrax.a");

    let status = Command::new("cc")
        .args(&["-c", "aura_tetrax_stub.c", "-o"])
        .arg(&obj)
        .status();

    if let Ok(s) = status {
        if s.success() {
            let _ = Command::new("ar")
                .args(&["rcs"])
                .arg(&lib)
                .arg(&obj)
                .status();
            println!("cargo:rustc-link-search=native={}", out_dir.display());
        }
    }
}
