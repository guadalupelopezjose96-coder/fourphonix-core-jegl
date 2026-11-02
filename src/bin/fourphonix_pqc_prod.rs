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
    PlatformVector, SovereignError, SovereignOperation, StatutorilySettledCommit, MICRO_SCALE,
    ORG_HOLDING_ID,
};

pub const FOURPHONIX_TENANT_ID: &str = "964521086569";
pub const PRODUCTION_FOLDER_ID: &str = "978426653396";

struct PostQuantumBackboneTunnel {
    tunnel_id: &'static str,
    route_corridor: &'static str,
    idempotency_key: [u8; 32],
    ml_kem_768_active: bool,
    macsec_active: bool,
    bytes_transferred: i128,
    settlement_micros: i128,
}

#[must_use]
pub fn format_micros(micros: i128) -> String {
    let abs_val = micros.unsigned_abs();
    let whole = abs_val / 1_000_000_u128;
    let frac = abs_val % 1_000_000_u128;
    format!("{whole}.{frac:06}")
}

fn build_fourphonix_pqc_mesh() -> [PostQuantumBackboneTunnel; 4] {
    [
        PostQuantumBackboneTunnel {
            tunnel_id: "FOURPHONIX-PQC-TUNNEL-01-VPP-CORE",
            route_corridor: "fourphonix.com <-> auraintelcore.com (FIPS 203 ML-KEM-768 + MACsec)",
            idempotency_key: [0xC1; 32],
            ml_kem_768_active: true,
            macsec_active: true,
            bytes_transferred: 85_000_000_000_000,
            settlement_micros: 240_000_000_000,
        },
        PostQuantumBackboneTunnel {
            tunnel_id: "FOURPHONIX-PQC-TUNNEL-02-TETRAX-EDGE",
            route_corridor: "fourphonix.com <-> auratetrax.com (Zero-Trust WireGuard PQC)",
            idempotency_key: [0xC2; 32],
            ml_kem_768_active: true,
            macsec_active: true,
            bytes_transferred: 62_000_000_000_000,
            settlement_micros: 175_000_000_000,
        },
        PostQuantumBackboneTunnel {
            tunnel_id: "FOURPHONIX-PQC-TUNNEL-03-PROPHONIX-AES67",
            route_corridor: "fourphonix.com <-> prophonix.com (Low-Latency Sonic Mesh)",
            idempotency_key: [0xC3; 32],
            ml_kem_768_active: true,
            macsec_active: true,
            bytes_transferred: 48_000_000_000_000,
            settlement_micros: 115_000_000_000,
        },
        PostQuantumBackboneTunnel {
            tunnel_id: "FOURPHONIX-PQC-TUNNEL-04-DOWNGRADE-TEST",
            route_corridor: "Fail-Closed Guard Test (Classical TLS Without ML-KEM-768)",
            idempotency_key: [0xC4; 32],
            ml_kem_768_active: false, // Intento de degradación criptográfica -> BLOQUEO FAIL-CLOSED
            macsec_active: true,
            bytes_transferred: 5_000_000_000,
            settlement_micros: 50_000_000_000,
        },
    ]
}

fn verify_and_settle_tunnel(
    tunnel: &PostQuantumBackboneTunnel,
) -> Result<SovereignOperation<StatutorilySettledCommit>, SovereignError> {
    let unverified = SovereignOperation::new_unverified(
        tunnel.idempotency_key,
        LifecycleFolder::Production,
        CorporateDomain::Fourphonix,
        PlatformVector::PostQuantumLink {
            ml_kem_768_active: tunnel.ml_kem_768_active,
            macsec_active: tunnel.macsec_active,
            bytes_transferred: tunnel.bytes_transferred,
        },
        tunnel.settlement_micros,
        true,
        2,
    );

    let attested: SovereignOperation<AttestedAndCompliantAction> =
        unverified.verify_safety_and_policy()?;

    attested.settle_statutory_ledger(AaufsaDeploymentContext::ExternalCloudSaasProvider)
}

fn main() -> ExitCode {
    let tunnels = build_fourphonix_pqc_mesh();
    let mut total_bytes: i128 = 0;
    let mut total_settled_micros: i128 = 0;

    let mut out = String::with_capacity(4096);
    out.push_str("{\n");
    out.push_str("  \"deployment_state\": \"FOURPHONIX_PQC_PRODUCTION_ACTIVE_100PCT\",\n");
    out.push_str(&format!("  \"holding_org_id\": \"{ORG_HOLDING_ID}\",\n"));
    out.push_str(&format!(
        "  \"lifecycle_folder\": \"02-Production ({PRODUCTION_FOLDER_ID})\",\n"
    ));
    out.push_str(&format!(
        "  \"primary_domain\": \"fourphonix.com ({FOURPHONIX_TENANT_ID})\",\n"
    ));
    out.push_str("  \"pqc_standard\": \"NIST FIPS 203 ML-KEM-768 + IEEE 802.1AE MACsec\",\n");
    out.push_str("  \"backbone_tunnels\": [\n");

    for (idx, tn) in tunnels.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"tunnel_id\": \"{}\",\n", tn.tunnel_id));
        out.push_str(&format!("      \"corridor\": \"{}\",\n", tn.route_corridor));
        out.push_str(&format!(
            "      \"ml_kem_768_active\": {},\n",
            tn.ml_kem_768_active
        ));
        out.push_str(&format!("      \"macsec_active\": {},\n", tn.macsec_active));

        match verify_and_settle_tunnel(tn) {
            Ok(settled) => {
                total_bytes = total_bytes.saturating_add(tn.bytes_transferred);
                total_settled_micros =
                    total_settled_micros.saturating_add(settled.gross_amount_micros);
                out.push_str("      \"status\": \"PQC_ENCRYPTED_AND_SETTLED\",\n");
                out.push_str(&format!(
                    "      \"bytes_transferred\": {},\n",
                    tn.bytes_transferred
                ));
                out.push_str(&format!(
                    "      \"settled_usd\": \"{}\"\n",
                    format_micros(settled.gross_amount_micros)
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
        "  \"total_verified_bytes_transferred\": {total_bytes},\n"
    ));
    out.push_str(&format!(
        "  \"total_fourphonix_settled_usd\": \"{}\"\n}}\n",
        format_micros(total_settled_micros)
    ));

    if let Ok(mut f) = File::create("fourphonix_pqc_production_state.json") {
        let _ = f.write_all(out.as_bytes());
    }
    println!("{out}");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod pqc_tests {
    use super::*;

    #[test]
    fn test_fourphonix_pqc_mesh_and_downgrade_guard() {
        let mesh = build_fourphonix_pqc_mesh();
        assert!(verify_and_settle_tunnel(&mesh[0]).is_ok());
        assert!(verify_and_settle_tunnel(&mesh[1]).is_ok());
        assert!(verify_and_settle_tunnel(&mesh[2]).is_ok());
        assert_eq!(
            verify_and_settle_tunnel(&mesh[3]),
            Err(SovereignError::UnencryptedNetworkLink)
        );
    }
}
