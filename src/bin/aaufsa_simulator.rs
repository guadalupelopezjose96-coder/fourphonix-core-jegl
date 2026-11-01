#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]
#![allow(unused_imports, dead_code, clippy::all, clippy::pedantic)]

use std::env;
use std::fs::File;
use std::io::Write;
use std::process::ExitCode;

use sovereign_multi_entity_governor::{
    AaufsaDeploymentContext, AttestedAndCompliantAction, CorporateDomain, LifecycleFolder,
    PlatformVector, SovereignError, SovereignOperation, StatutorilySettledCommit,
    AAUFSA_SAFE_HARBOR_MONTHS, AAUFSA_SMALL_BUSINESS_THRESHOLD_MICROS, AAUFSA_TAX_RATE_NUMERATOR,
    MICRO_SCALE, ORG_HOLDING_ID,
};

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

#[must_use]
pub fn sha256_digest(message: &[u8]) -> [u8; 32] {
    let mut h0: u32 = 0x6a09e667;
    let mut h1: u32 = 0xbb67ae85;
    let mut h2: u32 = 0x3c6ef372;
    let mut h3: u32 = 0xa54ff53a;
    let mut h4: u32 = 0x510e527f;
    let mut h5: u32 = 0x9b05688c;
    let mut h6: u32 = 0x1f83d9ab;
    let mut h7: u32 = 0x5be0cd19;

    let bit_len = (message.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(message.len().saturating_add(72));
    padded.extend_from_slice(message);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word_bytes) in chunk.chunks_exact(4).enumerate() {
            if let Some(slot) = w.get_mut(i) {
                let b0 = word_bytes.first().copied().unwrap_or(0);
                let b1 = word_bytes.get(1).copied().unwrap_or(0);
                let b2 = word_bytes.get(2).copied().unwrap_or(0);
                let b3 = word_bytes.get(3).copied().unwrap_or(0);
                *slot = u32::from_be_bytes([b0, b1, b2, b3]);
            }
        }
        for i in 16usize..64usize {
            let w15 = w.get(i.saturating_sub(15)).copied().unwrap_or(0);
            let w2 = w.get(i.saturating_sub(2)).copied().unwrap_or(0);
            let w16 = w.get(i.saturating_sub(16)).copied().unwrap_or(0);
            let w7 = w.get(i.saturating_sub(7)).copied().unwrap_or(0);
            let s0 = w15.rotate_right(7) ^ w15.rotate_right(18) ^ (w15 >> 3);
            let s1 = w2.rotate_right(17) ^ w2.rotate_right(19) ^ (w2 >> 10);
            if let Some(slot) = w.get_mut(i) {
                *slot = w16.wrapping_add(s0).wrapping_add(w7).wrapping_add(s1);
            }
        }
        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        let mut f = h5;
        let mut g = h6;
        let mut hh = h7;

        for i in 0usize..64usize {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let ki = SHA256_K.get(i).copied().unwrap_or(0);
            let wi = w.get(i).copied().unwrap_or(0);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(ki)
                .wrapping_add(wi);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
        h5 = h5.wrapping_add(f);
        h6 = h6.wrapping_add(g);
        h7 = h7.wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (idx, word) in [h0, h1, h2, h3, h4, h5, h6, h7].iter().enumerate() {
        let base = idx.saturating_mul(4);
        for (off, b_val) in word.to_be_bytes().iter().enumerate() {
            if let Some(target) = out.get_mut(base.saturating_add(off)) {
                *target = *b_val;
            }
        }
    }
    out
}

#[must_use]
pub fn to_hex_string(bytes: &[u8]) -> String {
    const HEX: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
    ];
    let mut s = String::with_capacity(bytes.len().saturating_mul(2));
    for b in bytes {
        s.push(HEX.get(usize::from(b >> 4)).copied().unwrap_or('0'));
        s.push(HEX.get(usize::from(b & 0x0f)).copied().unwrap_or('0'));
    }
    s
}

#[must_use]
pub fn format_usd_micros(micros: i128) -> String {
    let abs_val = micros.unsigned_abs();
    let whole = abs_val / 1_000_000_u128;
    let frac = abs_val % 1_000_000_u128;
    format!("{whole}.{frac:06}")
}

struct Scenario {
    code: &'static str,
    domain: CorporateDomain,
    name: &'static str,
    key: [u8; 32],
    vector: PlatformVector,
    gross_micros: i128,
    ctx: AaufsaDeploymentContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Outcome {
    Settled(SovereignOperation<StatutorilySettledCommit>),
    Blocked(SovereignError),
}

fn run_scenario(scn: &Scenario) -> Outcome {
    let op = SovereignOperation::new_unverified(
        scn.key,
        LifecycleFolder::Production,
        scn.domain,
        scn.vector,
        scn.gross_micros,
        true,
        2,
    );
    let attested: SovereignOperation<AttestedAndCompliantAction> =
        match op.verify_safety_and_policy() {
            Ok(v) => v,
            Err(e) => return Outcome::Blocked(e),
        };
    match attested.settle_statutory_ledger(scn.ctx) {
        Ok(s) => Outcome::Settled(s),
        Err(e) => Outcome::Blocked(e),
    }
}

fn build_scenarios() -> [Scenario; 6] {
    [
        Scenario {
            code: "SCN-01-SAAS-IMMUNITY",
            domain: CorporateDomain::AuraIntelCore,
            name: "AuraIntelCore-com",
            key: [0x11; 32],
            vector: PlatformVector::GeminiCognitive {
                store_false_enforced: true,
                prompt_shield_passed: true,
                tokens_consumed: 25_000_000,
            },
            gross_micros: 450_000_000_000,
            ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
        },
        Scenario {
            code: "SCN-02-SAFE-HARBOR-36M",
            domain: CorporateDomain::AuraTetrax,
            name: "AuraTetrax-com",
            key: [0x22; 32],
            vector: PlatformVector::PostQuantumLink {
                ml_kem_768_active: true,
                macsec_active: true,
                bytes_transferred: 10_000_000_000,
            },
            gross_micros: 820_000_000_000,
            ctx: AaufsaDeploymentContext::InternalCorporateOperation {
                consolidated_group_revenue_micros: 18_500_000_000_000,
                displaced_fte_annual_bls_wage_micros: 240_000_000_000,
                displacement_duration_days: 180,
                reassigned_without_wage_reduction: true,
                reassignment_completed_months: 36,
                early_termination_triggered: false,
                clawback_escrow_balance_micros: 40_000_000_000,
            },
        },
        Scenario {
            code: "SCN-03-TAXABLE-1530BPS",
            domain: CorporateDomain::Prophonix,
            name: "Prophonix-com",
            key: [0x33; 32],
            vector: PlatformVector::SonicLoudness {
                integrated_micro_lufs: -23_000_000,
                true_peak_micro_dbtp: -1_200_000,
            },
            gross_micros: 610_000_000_000,
            ctx: AaufsaDeploymentContext::InternalCorporateOperation {
                consolidated_group_revenue_micros: 18_500_000_000_000,
                displaced_fte_annual_bls_wage_micros: 300_000_000_000,
                displacement_duration_days: 145,
                reassigned_without_wage_reduction: false,
                reassignment_completed_months: 0,
                early_termination_triggered: false,
                clawback_escrow_balance_micros: 0,
            },
        },
        Scenario {
            code: "SCN-04-CLAWBACK-SETTLED",
            domain: CorporateDomain::Fourphonix,
            name: "Fourphonix-com",
            key: [0x44; 32],
            vector: PlatformVector::PostQuantumLink {
                ml_kem_768_active: true,
                macsec_active: true,
                bytes_transferred: 95_000_000_000,
            },
            gross_micros: 530_000_000_000,
            ctx: AaufsaDeploymentContext::InternalCorporateOperation {
                consolidated_group_revenue_micros: 18_500_000_000_000,
                displaced_fte_annual_bls_wage_micros: 200_000_000_000,
                displacement_duration_days: 210,
                reassigned_without_wage_reduction: true,
                reassignment_completed_months: 20,
                early_termination_triggered: true,
                clawback_escrow_balance_micros: 35_000_000_000,
            },
        },
        Scenario {
            code: "SCN-05-SMALL-BIZ-EXEMPT",
            domain: CorporateDomain::AuraIntelStudios3d,
            name: "AuraIntel-Global-com",
            key: [0x55; 32],
            vector: PlatformVector::VppEnergetics {
                frequency_micro_hz: 60_000_000,
                energy_dispatched_micro_wh: 500_000_000_000,
                anti_islanding_armed: true,
            },
            gross_micros: 390_000_000_000,
            ctx: AaufsaDeploymentContext::InternalCorporateOperation {
                consolidated_group_revenue_micros: 4_850_000_000_000,
                displaced_fte_annual_bls_wage_micros: 150_000_000_000,
                displacement_duration_days: 120,
                reassigned_without_wage_reduction: false,
                reassignment_completed_months: 0,
                early_termination_triggered: false,
                clawback_escrow_balance_micros: 0,
            },
        },
        Scenario {
            code: "SCN-06-FAIL-CLOSED-GUARD",
            domain: CorporateDomain::Standard1979,
            name: "Standard1979-com",
            key: [0x66; 32],
            vector: PlatformVector::NormativeCertification {
                worm_locked: true,
                ml_dsa_signed: true,
            },
            gross_micros: 275_000_000_000,
            ctx: AaufsaDeploymentContext::InternalCorporateOperation {
                consolidated_group_revenue_micros: 18_500_000_000_000,
                displaced_fte_annual_bls_wage_micros: 200_000_000_000,
                displacement_duration_days: 160,
                reassigned_without_wage_reduction: true,
                reassignment_completed_months: 14,
                early_termination_triggered: true,
                clawback_escrow_balance_micros: 10_000_000_000,
            },
        },
    ]
}

fn main() -> ExitCode {
    let mut output_path = String::from("aaufsa_multi_domain_audit_report.json");
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--output" || arg == "-o" {
            if let Some(p) = args.next() {
                output_path = p;
            }
        }
    }

    let scenarios = build_scenarios();
    let mut body = String::with_capacity(4096);
    body.push_str("{\n");
    body.push_str(&format!(
        "  \"holding_organization_id\": \"{ORG_HOLDING_ID}\",\n"
    ));
    body.push_str(&format!(
        "  \"tax_rate_bps\": {AAUFSA_TAX_RATE_NUMERATOR},\n"
    ));
    body.push_str(&format!(
        "  \"safe_harbor_months\": {AAUFSA_SAFE_HARBOR_MONTHS},\n"
    ));
    body.push_str(&format!(
        "  \"small_business_threshold_usd\": \"{}\",\n",
        format_usd_micros(AAUFSA_SMALL_BUSINESS_THRESHOLD_MICROS)
    ));
    body.push_str("  \"domains\": [\n");

    for (idx, scn) in scenarios.iter().enumerate() {
        let out = run_scenario(scn);
        body.push_str("    {\n");
        body.push_str(&format!("      \"code\": \"{}\",\n", scn.code));
        body.push_str(&format!("      \"domain\": \"{}\",\n", scn.name));
        body.push_str(&format!(
            "      \"tenant_id\": \"{}\",\n",
            scn.domain.tenant_id()
        ));
        match out {
            Outcome::Settled(op) => {
                body.push_str("      \"status\": \"COMMITTED_SERIALIZABLE\",\n");
                if let Some(det) = op.tax_determination {
                    body.push_str(&format!(
                        "      \"cloud_saas_exempt\": {},\n",
                        det.cloud_saas_exempt
                    ));
                    body.push_str(&format!(
                        "      \"safe_harbor_applied\": {},\n",
                        det.safe_harbor_applied
                    ));
                    body.push_str(&format!(
                        "      \"clawback_triggered\": {},\n",
                        det.clawback_triggered
                    ));
                    body.push_str(&format!(
                        "      \"tax_usd\": \"{}\",\n",
                        format_usd_micros(det.total_chapter56_tax_micros)
                    ));
                    body.push_str(&format!(
                        "      \"oasi_75_usd\": \"{}\",\n",
                        format_usd_micros(det.oasi_di_trust_fund_75_micros)
                    ));
                    body.push_str(&format!(
                        "      \"reskilling_25_usd\": \"{}\"\n",
                        format_usd_micros(det.workforce_reskilling_25_micros)
                    ));
                }
            }
            Outcome::Blocked(err) => {
                body.push_str("      \"status\": \"BLOCKED_FAIL_CLOSED\",\n");
                body.push_str(&format!("      \"reason\": \"{err:?}\"\n"));
            }
        }
        if idx < 5 {
            body.push_str("    },\n");
        } else {
            body.push_str("    }\n");
        }
    }
    body.push_str("  ]");
    let seal = to_hex_string(&sha256_digest(body.as_bytes()));
    body.push_str(&format!(",\n  \"sha256_seal\": \"{seal}\"\n}}\n"));

    if let Ok(mut f) = File::create(&output_path) {
        let _ = f.write_all(body.as_bytes());
    }
    println!("{body}");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_nist_sha256_abc() {
        assert_eq!(
            to_hex_string(&sha256_digest(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_all_six_domain_scenarios() {
        let scns = build_scenarios();
        for (idx, scn) in scns.iter().enumerate() {
            let outcome = run_scenario(scn);
            if idx < 5 {
                assert!(matches!(outcome, Outcome::Settled(_)));
            } else {
                assert_eq!(
                    outcome,
                    Outcome::Blocked(SovereignError::ClawbackEscrowUnderfunded)
                );
            }
        }
    }
}
