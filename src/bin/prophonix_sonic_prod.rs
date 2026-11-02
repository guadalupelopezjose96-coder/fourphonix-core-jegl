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

use sovereign_multi_entity_governor::{
    AaufsaDeploymentContext, AttestedAndCompliantAction, CorporateDomain, LifecycleFolder,
    PlatformVector, SovereignError, SovereignOperation, StatutorilySettledCommit,
    MAX_ALLOWED_TRUE_PEAK_MICRO_DBTP, MICRO_SCALE, ORG_HOLDING_ID,
};

pub const PROPHONIX_TENANT_ID: &str = "358233233";
pub const PRODUCTION_FOLDER_ID: &str = "978426653396";

struct SonicChannelStream {
    stream_id: &'static str,
    standard_profile: &'static str,
    idempotency_key: [u8; 32],
    integrated_micro_lufs: i128,
    true_peak_micro_dbtp: i128,
    billing_amount_micros: i128,
}

#[must_use]
pub fn format_signed_micros(micros: i128) -> String {
    let is_neg = micros < 0;
    let abs_val = micros.unsigned_abs();
    let whole = abs_val / 1_000_000_u128;
    let frac = abs_val % 1_000_000_u128;
    if is_neg {
        format!("-{whole}.{frac:06}")
    } else {
        format!("{whole}.{frac:06}")
    }
}

fn build_prophonix_sonic_streams() -> [SonicChannelStream; 4] {
    [
        SonicChannelStream {
            stream_id: "PROPHONIX-CH01-EBU-R128-BROADCAST",
            standard_profile: "ITU-R BS.1770-5 / EBU R128 (-23.0 LUFS)",
            idempotency_key: [0xB1; 32],
            integrated_micro_lufs: -23_000_000, // -23.000000 LUFS
            true_peak_micro_dbtp: -1_500_000,   // -1.500000 dBTP (<= -1.0 dBTP)
            billing_amount_micros: 185_000_000_000,
        },
        SonicChannelStream {
            stream_id: "PROPHONIX-CH02-ATSC-A85-CALM-ACT",
            standard_profile: "ATSC A/85 / FCC CALM Act (-24.0 LKFS)",
            idempotency_key: [0xB2; 32],
            integrated_micro_lufs: -24_000_000, // -24.000000 LKFS
            true_peak_micro_dbtp: -2_000_000,   // -2.000000 dBTP (<= -1.0 dBTP)
            billing_amount_micros: 210_000_000_000,
        },
        SonicChannelStream {
            stream_id: "PROPHONIX-CH03-AES77-STREAMING",
            standard_profile: "AES TD1004 / AES77 Spatial (-14.0 LUFS)",
            idempotency_key: [0xB3; 32],
            integrated_micro_lufs: -14_000_000, // -14.000000 LUFS
            true_peak_micro_dbtp: -1_000_000,   // -1.000000 dBTP exactos en el límite
            billing_amount_micros: 165_000_000_000,
        },
        SonicChannelStream {
            stream_id: "PROPHONIX-CH04-OVERMODULATED-TEST",
            standard_profile: "Fail-Closed Guard Test (-0.2 dBTP Violation)",
            idempotency_key: [0xB4; 32],
            integrated_micro_lufs: -12_000_000,
            true_peak_micro_dbtp: -200_000, // -0.200000 dBTP (> -1.0 dBTP -> BLOQUEO FAIL-CLOSED)
            billing_amount_micros: 90_000_000_000,
        },
    ]
}

fn process_sonic_stream(
    stream: &SonicChannelStream,
) -> Result<SovereignOperation<StatutorilySettledCommit>, SovereignError> {
    let unverified = SovereignOperation::new_unverified(
        stream.idempotency_key,
        LifecycleFolder::Production,
        CorporateDomain::Prophonix,
        PlatformVector::SonicLoudness {
            integrated_micro_lufs: stream.integrated_micro_lufs,
            true_peak_micro_dbtp: stream.true_peak_micro_dbtp,
        },
        stream.billing_amount_micros,
        true,
        2,
    );

    let attested: SovereignOperation<AttestedAndCompliantAction> =
        unverified.verify_safety_and_policy()?;

    attested.settle_statutory_ledger(AaufsaDeploymentContext::ExternalCloudSaasProvider)
}

fn main() -> ExitCode {
    let streams = build_prophonix_sonic_streams();
    let mut total_settled_micros: i128 = 0;

    let mut out = String::with_capacity(4096);
    out.push_str("{\n");
    out.push_str("  \"deployment_state\": \"PROPHONIX_SONIC_PRODUCTION_ACTIVE_100PCT\",\n");
    out.push_str(&format!("  \"holding_org_id\": \"{ORG_HOLDING_ID}\",\n"));
    out.push_str(&format!(
        "  \"lifecycle_folder\": \"02-Production ({PRODUCTION_FOLDER_ID})\",\n"
    ));
    out.push_str(&format!(
        "  \"primary_domain\": \"prophonix.com ({PROPHONIX_TENANT_ID})\",\n"
    ));
    out.push_str(&format!(
        "  \"true_peak_ceiling_dbtp\": \"{}\",\n",
        format_signed_micros(MAX_ALLOWED_TRUE_PEAK_MICRO_DBTP)
    ));
    out.push_str("  \"sonic_streams\": [\n");

    for (idx, st) in streams.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"stream_id\": \"{}\",\n", st.stream_id));
        out.push_str(&format!(
            "      \"profile\": \"{}\",\n",
            st.standard_profile
        ));
        out.push_str(&format!(
            "      \"integrated_lufs\": \"{}\",\n",
            format_signed_micros(st.integrated_micro_lufs)
        ));
        out.push_str(&format!(
            "      \"true_peak_dbtp\": \"{}\",\n",
            format_signed_micros(st.true_peak_micro_dbtp)
        ));

        match process_sonic_stream(st) {
            Ok(settled) => {
                total_settled_micros =
                    total_settled_micros.saturating_add(settled.gross_amount_micros);
                out.push_str("      \"status\": \"NORMALIZED_AND_SETTLED\",\n");
                out.push_str(&format!(
                    "      \"settled_usd\": \"{}\"\n",
                    format_signed_micros(settled.gross_amount_micros)
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
        "  \"total_prophonix_settled_usd\": \"{}\"\n}}\n",
        format_signed_micros(total_settled_micros)
    ));

    if let Ok(mut f) = File::create("prophonix_sonic_production_state.json") {
        let _ = f.write_all(out.as_bytes());
    }
    println!("{out}");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod sonic_tests {
    use super::*;

    #[test]
    fn test_prophonix_streams_and_true_peak_guard() {
        let streams = build_prophonix_sonic_streams();
        assert!(process_sonic_stream(&streams[0]).is_ok());
        assert!(process_sonic_stream(&streams[1]).is_ok());
        assert!(process_sonic_stream(&streams[2]).is_ok());
        assert_eq!(
            process_sonic_stream(&streams[3]),
            Err(SovereignError::LoudnessTruePeakExceeded)
        );
    }
}
