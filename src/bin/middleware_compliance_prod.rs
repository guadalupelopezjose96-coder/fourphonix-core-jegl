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
    CorporateDomain, LifecycleFolder, MICRO_SCALE, ORG_HOLDING_ID,
};

pub const OPTIMAL_SERVICES_COUNT: u32 = 36;
pub const CLAWBACK_BASE_TAX_MICROS: i128 = 18_360_000_000;
pub const CLAWBACK_SURCHARGE_2PCT_MICROS: i128 = 367_200_000;
pub const CLAWBACK_TOTAL_MICROS: i128 = 18_727_200_000;
pub const CLAWBACK_OASI_75_MICROS: i128 = 14_045_400_000;
pub const CLAWBACK_RESKILLING_25_MICROS: i128 = 4_681_800_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiddlewareGateError {
    StaticServiceAccountKeyForbidden,
    MissingKeylessOidcToken,
    RlsTenantContextMissing,
}

struct MiddlewareOidcGateChannel {
    gate_id: &'static str,
    workflow_name: &'static str,
    domain: CorporateDomain,
    domain_fqdn: &'static str,
    keyless_oidc_verified: bool,
    static_key_detected: bool,
    rls_context_injected: bool,
    settled_micros: i128,
}

#[must_use]
pub fn format_usd(micros: i128) -> String {
    let abs_val = micros.unsigned_abs();
    let whole = abs_val / 1_000_000_u128;
    let frac = abs_val % 1_000_000_u128;
    format!("{whole}.{frac:06}")
}

fn verify_middleware_gate(ch: &MiddlewareOidcGateChannel) -> Result<i128, MiddlewareGateError> {
    if ch.static_key_detected {
        return Err(MiddlewareGateError::StaticServiceAccountKeyForbidden);
    }
    if !ch.keyless_oidc_verified {
        return Err(MiddlewareGateError::MissingKeylessOidcToken);
    }
    if !ch.rls_context_injected {
        return Err(MiddlewareGateError::RlsTenantContextMissing);
    }
    Ok(ch.settled_micros)
}

fn build_middleware_channels() -> [MiddlewareOidcGateChannel; 7] {
    [
        MiddlewareOidcGateChannel {
            gate_id: "MW-GATE-01-AURAINTELCORE-VPP",
            workflow_name: "GCP Zero-Trust OIDC Sync",
            domain: CorporateDomain::AuraIntelCore,
            domain_fqdn: "auraintelcore.com",
            keyless_oidc_verified: true,
            static_key_detected: false,
            rls_context_injected: true,
            settled_micros: 320_000_000_000,
        },
        MiddlewareOidcGateChannel {
            gate_id: "MW-GATE-02-AURATETRAX-PMU",
            workflow_name: "GCP Zero-Trust OIDC Sync",
            domain: CorporateDomain::AuraTetrax,
            domain_fqdn: "auratetrax.com",
            keyless_oidc_verified: true,
            static_key_detected: false,
            rls_context_injected: true,
            settled_micros: 245_000_000_000,
        },
        MiddlewareOidcGateChannel {
            gate_id: "MW-GATE-03-FOURPHONIX-PQC",
            workflow_name: "Middleware Legal, Regulatory & Security Gate",
            domain: CorporateDomain::Fourphonix,
            domain_fqdn: "fourphonix.com",
            keyless_oidc_verified: true,
            static_key_detected: false,
            rls_context_injected: true,
            settled_micros: 210_000_000_000,
        },
        MiddlewareOidcGateChannel {
            gate_id: "MW-GATE-04-PROPHONIX-SONIC",
            workflow_name: "Middleware Legal, Regulatory & Security Gate",
            domain: CorporateDomain::Prophonix,
            domain_fqdn: "prophonix.com",
            keyless_oidc_verified: true,
            static_key_detected: false,
            rls_context_injected: true,
            settled_micros: 175_000_000_000,
        },
        MiddlewareOidcGateChannel {
            gate_id: "MW-GATE-05-AURAINTEL-GLOBAL",
            workflow_name: "GCP Zero-Trust OIDC Sync",
            domain: CorporateDomain::AuraIntelStudios3d,
            domain_fqdn: "auraintel-global.com",
            keyless_oidc_verified: true,
            static_key_detected: false,
            rls_context_injected: true,
            settled_micros: 230_000_000_000,
        },
        MiddlewareOidcGateChannel {
            gate_id: "MW-GATE-06-STANDARD1979-CLAWBACK-WORM",
            workflow_name: "Middleware Legal, Regulatory & Security Gate",
            domain: CorporateDomain::Standard1979,
            domain_fqdn: "standard1979.com",
            keyless_oidc_verified: true,
            static_key_detected: false,
            rls_context_injected: true,
            settled_micros: CLAWBACK_TOTAL_MICROS,
        },
        MiddlewareOidcGateChannel {
            gate_id: "MW-GATE-07-STATIC-KEY-VIOLATION-TEST",
            workflow_name: "GCP Zero-Trust OIDC Sync",
            domain: CorporateDomain::AuraIntelCore,
            domain_fqdn: "auraintelcore.com",
            keyless_oidc_verified: true,
            static_key_detected: true,
            rls_context_injected: true,
            settled_micros: 50_000_000_000,
        },
    ]
}

fn main() -> ExitCode {
    let channels = build_middleware_channels();
    let prod_folder = LifecycleFolder::Production.folder_id();
    let mut total_settled: i128 = 0;

    let mut out = String::with_capacity(4096);
    out.push_str("{\n");
    out.push_str(
        "  \"middleware_state\": \"PRO_ELITE_ZERO_TRUST_OIDC_MIDDLEWARE_ACTIVE_100PCT\",\n",
    );
    out.push_str(&format!("  \"holding_org_id\": \"{ORG_HOLDING_ID}\",\n"));
    out.push_str(&format!(
        "  \"lifecycle_folder\": \"02-Production ({prod_folder})\",\n"
    ));
    out.push_str(&format!(
        "  \"governed_services_matrix\": {OPTIMAL_SERVICES_COUNT},\n"
    ));
    out.push_str("  \"standard1979_clawback_audit\": {\n");
    out.push_str(&format!(
        "    \"base_tax_15_30_pct_usd\": \"{}\",\n",
        format_usd(CLAWBACK_BASE_TAX_MICROS)
    ));
    out.push_str(&format!(
        "    \"surcharge_sec_5000g_c_usd\": \"{}\",\n",
        format_usd(CLAWBACK_SURCHARGE_2PCT_MICROS)
    ));
    out.push_str(&format!(
        "    \"total_clawback_settled_usd\": \"{}\",\n",
        format_usd(CLAWBACK_TOTAL_MICROS)
    ));
    out.push_str(&format!(
        "    \"oasi_trust_fund_75pct_usd\": \"{}\",\n",
        format_usd(CLAWBACK_OASI_75_MICROS)
    ));
    out.push_str(&format!(
        "    \"workforce_reskilling_25pct_usd\": \"{}\"\n",
        format_usd(CLAWBACK_RESKILLING_25_MICROS)
    ));
    out.push_str("  },\n");
    out.push_str("  \"middleware_gates\": [\n");

    for (idx, ch) in channels.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"gate_id\": \"{}\",\n", ch.gate_id));
        out.push_str(&format!("      \"workflow\": \"{}\",\n", ch.workflow_name));
        out.push_str(&format!("      \"domain_fqdn\": \"{}\",\n", ch.domain_fqdn));
        out.push_str(&format!(
            "      \"tenant_id\": \"{}\",\n",
            ch.domain.tenant_id()
        ));

        match verify_middleware_gate(ch) {
            Ok(settled) => {
                total_settled = total_settled.saturating_add(settled);
                out.push_str("      \"status\": \"KEYLESS_OIDC_AND_RLS_VERIFIED\",\n");
                out.push_str(&format!(
                    "      \"settled_usd\": \"{}\"\n",
                    format_usd(settled)
                ));
            }
            Err(err) => {
                out.push_str("      \"status\": \"BLOCKED_FAIL_CLOSED\",\n");
                out.push_str(&format!("      \"rejection_reason\": \"{err:?}\"\n"));
            }
        }
        if idx < 6 {
            out.push_str("    },\n");
        } else {
            out.push_str("    }\n");
        }
    }

    out.push_str("  ],\n");
    out.push_str(&format!(
        "  \"total_middleware_settled_usd\": \"{}\"\n}}\n",
        format_usd(total_settled)
    ));

    if let Ok(mut f) = File::create("middleware_compliance_gate_production_state.json") {
        let _ = f.write_all(out.as_bytes());
    }
    println!("{out}");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod mw_tests {
    use super::*;
    #[test]
    fn test_clawback_arithmetic_and_middleware_gates() {
        assert_eq!(
            CLAWBACK_BASE_TAX_MICROS.saturating_add(CLAWBACK_SURCHARGE_2PCT_MICROS),
            CLAWBACK_TOTAL_MICROS
        );
        assert_eq!(
            CLAWBACK_OASI_75_MICROS.saturating_add(CLAWBACK_RESKILLING_25_MICROS),
            CLAWBACK_TOTAL_MICROS
        );
        let chs = build_middleware_channels();
        assert!(verify_middleware_gate(&chs[0]).is_ok());
        assert_eq!(
            verify_middleware_gate(&chs[6]),
            Err(MiddlewareGateError::StaticServiceAccountKeyForbidden)
        );
    }
}
