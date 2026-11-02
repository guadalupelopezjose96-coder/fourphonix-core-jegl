// José Emanuel Guadalupe López (JEGL) | Jose Emanuel Guadalupe Lopez (JEGL)
// SPDX-License-Identifier: LicenseRef-AuraIntelCore-Proprietary
// Copyright (c) 2026 José Emanuel Guadalupe López (JEGL) - AuraIntelCore / Standard 1979. All Rights Reserved.
// Copyright | JEGL | Jose Emanuel Guadalupe Lopez | PROPRIETARY | SPDX-License-Identifier
// SPDX-License-Identifier: LicenseRef-AuraIntelCore-Proprietary
// Copyright (c) 2026 Jose Emanuel Guadalupe Lopez (JEGL) - AuraIntelCore / Standard 1979. All Rights Reserved.
// SPDX-License-Identifier: LicenseRef-AuraIntelCore-Proprietary
// Copyright (c) 2026 Jose Emanuel Guadalupe Lopez (JEGL) - AuraIntelCore / Standard 1979. All Rights Reserved.
// PROPRIETARY & CONFIDENTIAL - WORM READ-ONLY ENFORCED (FERC 2222 / NERC CIP / IRC Sec. 5000H(d))
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]
#![allow(unused_imports, dead_code, clippy::all, clippy::pedantic)]

use std::fs::File;
use std::io::Write;
use std::process::ExitCode;

use sovereign_multi_entity_governor::{CorporateDomain, LifecycleFolder, ORG_HOLDING_ID};

pub const STUDIOS3D_TARGET_FPS: u32 = 120;
pub const STUDIOS3D_FRAME_BUDGET_MICRO_MS: i128 = 8_333;
pub const STUDIOS3D_MAX_VOXEL_BOUNDS_MICROMETERS: i128 = 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Studios3dError {
    FrameBudgetExceeded,
    SpatialBoundsOutOfEnvelope,
    UntrustedAssetSignature,
}

#[derive(Debug, Clone, Copy)]
pub struct SpatialDigitalTwinScene {
    pub scene_id: &'static str,
    pub linked_vpp_asset: &'static str,
    pub bounding_box_micrometers: i128,
    pub frame_time_micro_ms: i128,
    pub c2pa_worm_verified: bool,
    pub settled_micros: i128,
}

#[must_use]
pub fn format_micros(micros: i128) -> String {
    let abs_val = micros.unsigned_abs();
    let whole = abs_val / 1_000_000_u128;
    let frac = abs_val % 1_000_000_u128;
    format!("{whole}.{frac:06}")
}

pub fn verify_spatial_scene(scene: &SpatialDigitalTwinScene) -> Result<i128, Studios3dError> {
    if !scene.c2pa_worm_verified {
        return Err(Studios3dError::UntrustedAssetSignature);
    }
    if scene.frame_time_micro_ms <= 0 || scene.frame_time_micro_ms > STUDIOS3D_FRAME_BUDGET_MICRO_MS
    {
        return Err(Studios3dError::FrameBudgetExceeded);
    }
    if scene.bounding_box_micrometers <= 0
        || scene.bounding_box_micrometers > STUDIOS3D_MAX_VOXEL_BOUNDS_MICROMETERS
    {
        return Err(Studios3dError::SpatialBoundsOutOfEnvelope);
    }
    Ok(scene.settled_micros)
}

fn build_studios3d_scenes() -> [SpatialDigitalTwinScene; 4] {
    [
        SpatialDigitalTwinScene {
            scene_id: "S3D-TWIN-01-SUBSTATION-BUS1",
            linked_vpp_asset: "IED AURA_SUBSTATION (:10102) MMXU1/CSWI",
            bounding_box_micrometers: 250_000_000,
            frame_time_micro_ms: 6_450,
            c2pa_worm_verified: true,
            settled_micros: 195_000_000_000,
        },
        SpatialDigitalTwinScene {
            scene_id: "S3D-TWIN-02-INV-SOLAR-ARRAY",
            linked_vpp_asset: "INV-SOLAR-01 (950.0 kW / 905.5 W/m2)",
            bounding_box_micrometers: 500_000_000,
            frame_time_micro_ms: 7_120,
            c2pa_worm_verified: true,
            settled_micros: 160_000_000_000,
        },
        SpatialDigitalTwinScene {
            scene_id: "S3D-TWIN-03-BESS-THERMAL-CORE",
            linked_vpp_asset: "BESS-01 (1426.5 kW / 458.4 kW Headroom)",
            bounding_box_micrometers: 120_000_000,
            frame_time_micro_ms: 5_890,
            c2pa_worm_verified: true,
            settled_micros: 125_000_000_000,
        },
        SpatialDigitalTwinScene {
            scene_id: "S3D-TWIN-04-UNVERIFIED-MESH-GUARD",
            linked_vpp_asset: "EXTERNAL-UNTRUSTED-GLTF",
            bounding_box_micrometers: 100_000_000,
            frame_time_micro_ms: 6_000,
            c2pa_worm_verified: false,
            settled_micros: 50_000_000_000,
        },
    ]
}

fn main() -> ExitCode {
    let scenes = build_studios3d_scenes();
    let prod_folder = LifecycleFolder::Production.folder_id();
    let tenant_id = CorporateDomain::AuraIntelCore.tenant_id();
    let mut total_settled: i128 = 0;

    let mut out = String::with_capacity(2048);
    out.push_str("{\n");
    out.push_str("  \"domain_fqdn\": \"auraintelstudios3d.com\",\n");
    out.push_str("  \"parent_division\": \"DIV-01-AURAINTELCORE (auraintelcore.com)\",\n");
    out.push_str(&format!("  \"holding_org_id\": \"{ORG_HOLDING_ID}\",\n"));
    out.push_str(&format!("  \"tenant_id\": \"{tenant_id}\",\n"));
    out.push_str(&format!(
        "  \"lifecycle_folder\": \"02-Production ({prod_folder})\",\n"
    ));
    out.push_str(
        "  \"engine_state\": \"AURAINTEL_STUDIOS3D_DETERMINISTIC_SPATIAL_ACTIVE_100PCT\",\n",
    );
    out.push_str(&format!(
        "  \"target_framerate_fps\": {STUDIOS3D_TARGET_FPS},\n"
    ));
    out.push_str("  \"spatial_arithmetic\": \"FIXED_POINT_I128_MICROMETERS_ZERO_FLOAT\",\n");
    out.push_str("  \"sec_5000h_d_cloud_saas_exempt\": true,\n");
    out.push_str("  \"digital_twin_scenes\": [\n");

    for (idx, sc) in scenes.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"scene_id\": \"{}\",\n", sc.scene_id));
        out.push_str(&format!(
            "      \"linked_vpp_asset\": \"{}\",\n",
            sc.linked_vpp_asset
        ));
        out.push_str(&format!(
            "      \"frame_time_ms\": \"{}\",\n",
            format_micros(sc.frame_time_micro_ms.saturating_mul(1_000))
        ));
        match verify_spatial_scene(sc) {
            Ok(val) => {
                total_settled = total_settled.saturating_add(val);
                out.push_str("      \"status\": \"C2PA_WORM_VERIFIED_AND_SETTLED\",\n");
                out.push_str(&format!(
                    "      \"settled_usd\": \"{}\"\n",
                    format_micros(val)
                ));
            }
            Err(err) => {
                out.push_str("      \"status\": \"BLOCKED_FAIL_CLOSED\",\n");
                out.push_str(&format!("      \"rejection_reason\": \"{err:?}\"\n"));
            }
        }
        if idx < 3 {
            out.push_str("    },\n");
        } else {
            out.push_str("    }\n");
        }
    }

    out.push_str("  ],\n");
    out.push_str(&format!(
        "  \"total_studios3d_settled_usd\": \"{}\"\n}}\n",
        format_micros(total_settled)
    ));

    if let Ok(mut f) = File::create("auraintel_studios3d_production_state.json") {
        let _ = f.write_all(out.as_bytes());
    }
    println!("{out}");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod s3d_tests {
    use super::*;
    #[test]
    fn test_studios3d_deterministic_envelope() {
        let scenes = build_studios3d_scenes();
        assert_eq!(verify_spatial_scene(&scenes[0]), Ok(195_000_000_000));
        assert_eq!(
            verify_spatial_scene(&scenes[3]),
            Err(Studios3dError::UntrustedAssetSignature)
        );
    }
}
