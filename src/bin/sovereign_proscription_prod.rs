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
    AaufsaDeploymentContext, CorporateDomain, LifecycleFolder, PlatformVector, SovereignError,
    SovereignOperation, ORG_HOLDING_ID,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProscriptionVerdict {
    IllegalityBlockedFailClosed(SovereignError),
    UnexpectedPass,
}

struct ProscribedAttemptCase {
    case_id: &'static str,
    domain: CorporateDomain,
    domain_fqdn: &'static str,
    proscribed_act: &'static str,
    statutory_basis: &'static str,
    folder: LifecycleFolder,
    tpm_attested: bool,
    signatures: u8,
    vector: PlatformVector,
    aaufsa_ctx: AaufsaDeploymentContext,
    expected_error: SovereignError,
}

fn build_proscription_test_battery() -> [ProscribedAttemptCase; 6] {
    [
        // 1. AuraIntelCore.com: Ilegalización de retención de prompts (store: true prohibido)
        ProscribedAttemptCase {
            case_id: "ILLEGAL-01-GEMINI-RETENTION",
            domain: CorporateDomain::AuraIntelCore,
            domain_fqdn: "auraintelcore.com",
            proscribed_act: "Attempted Gemini API Invocation with store:true Data Retention",
            statutory_basis: "EU AI Act Art. 15 / GDPR Art. 25 Zero-Retention Mandate",
            folder: LifecycleFolder::Production,
            tpm_attested: true,
            signatures: 2,
            vector: PlatformVector::GeminiCognitive {
                store_false_enforced: false,
                prompt_shield_passed: true,
                tokens_consumed: 10_000,
            },
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
            expected_error: SovereignError::GeminiDataRetentionViolation,
        },
        // 2. AuraTetrax.com: Ilegalización de maniobra eléctrica fuera de envolvente IEEE 1547 (62.50 Hz)
        ProscribedAttemptCase {
            case_id: "ILLEGAL-02-GRID-OVERFREQUENCY",
            domain: CorporateDomain::AuraTetrax,
            domain_fqdn: "auratetrax.com",
            proscribed_act: "Attempted VPP Dispatch at 62.50 Hz Outside IEEE 1547 Envelope",
            statutory_basis: "FERC Order 2222 / NERC PRC-024 / IEEE 1547-2018",
            folder: LifecycleFolder::Production,
            tpm_attested: true,
            signatures: 2,
            vector: PlatformVector::VppEnergetics {
                frequency_micro_hz: 62_500_000,
                energy_dispatched_micro_wh: 100_000_000_000,
                anti_islanding_armed: true,
            },
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
            expected_error: SovereignError::VppFrequencyOutOfEnvelope,
        },
        // 3. Fourphonix.com: Ilegalización de enlace sin cifrado poscuántico ML-KEM-768
        ProscribedAttemptCase {
            case_id: "ILLEGAL-03-PQC-DOWNGRADE",
            domain: CorporateDomain::Fourphonix,
            domain_fqdn: "fourphonix.com",
            proscribed_act: "Attempted Backbone Transmission Without FIPS 203 ML-KEM-768",
            statutory_basis: "NIST FIPS 203 / NIST SP 800-53 SC-8 & CM-7",
            folder: LifecycleFolder::Production,
            tpm_attested: true,
            signatures: 2,
            vector: PlatformVector::PostQuantumLink {
                ml_kem_768_active: false,
                macsec_active: true,
                bytes_transferred: 1_000_000,
            },
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
            expected_error: SovereignError::UnencryptedNetworkLink,
        },
        // 4. Prophonix.com: Ilegalización de exceso de sonoridad True Peak (-0.1 dBTP > -1.0 dBTP)
        ProscribedAttemptCase {
            case_id: "ILLEGAL-04-ACOUSTIC-CLIPPING",
            domain: CorporateDomain::Prophonix,
            domain_fqdn: "prophonix.com",
            proscribed_act: "Attempted Audio Broadcast at -0.1 dBTP Exceeding -1.0 dBTP Ceiling",
            statutory_basis: "FCC CALM Act 47 U.S.C. 621 / ITU-R BS.1770-5 / EBU R128",
            folder: LifecycleFolder::Production,
            tpm_attested: true,
            signatures: 2,
            vector: PlatformVector::SonicLoudness {
                integrated_micro_lufs: -11_000_000,
                true_peak_micro_dbtp: -100_000,
            },
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
            expected_error: SovereignError::LoudnessTruePeakExceeded,
        },
        // 5. AuraIntelStudios3D.com: Ilegalización de ejecución en 02-Production sin Regla de 4 Ojos
        ProscribedAttemptCase {
            case_id: "ILLEGAL-05-SINGLE-CUSTODIAN-PROD",
            domain: CorporateDomain::AuraIntelStudios3D,
            domain_fqdn: "auraintelstudios3d.com",
            proscribed_act: "Attempted Production Execution with Only 1 Custodian Signature",
            statutory_basis: "SOC 2 Type II / ISO 27001 SoD Four-Eyes Principle",
            folder: LifecycleFolder::Production,
            tpm_attested: true,
            signatures: 1,
            vector: PlatformVector::SonicLoudness {
                integrated_micro_lufs: -16_000_000,
                true_peak_micro_dbtp: -1_500_000,
            },
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
            expected_error: SovereignError::DualSignatureMissingInProduction,
        },
        // 6. Standard1979.com: Ilegalización de evasión de Clawback AAUFSA sin reserva en Escrow
        ProscribedAttemptCase {
            case_id: "ILLEGAL-06-UNFUNDED-AAUFSA-CLAWBACK",
            domain: CorporateDomain::Standard1979,
            domain_fqdn: "standard1979.com",
            proscribed_act: "Attempted Early Termination at Month 12 Without Clawback Escrow",
            statutory_basis: "AAUFSA IRC Chapter 56 Sec. 5000G(c) & IRC Sec. 7701(o)",
            folder: LifecycleFolder::Production,
            tpm_attested: true,
            signatures: 2,
            vector: PlatformVector::NormativeCertification {
                worm_locked: true,
                ml_dsa_signed: true,
            },
            aaufsa_ctx: AaufsaDeploymentContext::InternalCorporateOperation {
                consolidated_group_revenue_micros: 18_500_000_000_000,
                displaced_fte_annual_bls_wage_micros: 250_000_000_000,
                displacement_duration_days: 150,
                reassigned_without_wage_reduction: true,
                reassignment_completed_months: 12,
                early_termination_triggered: true,
                clawback_escrow_balance_micros: 0,
            },
            expected_error: SovereignError::ClawbackEscrowUnderfunded,
        },
    ]
}

fn enforce_proscription_case(case: &ProscribedAttemptCase) -> ProscriptionVerdict {
    let op = SovereignOperation::new_unverified(
        [0xE1; 32],
        case.folder,
        case.domain,
        case.vector,
        100_000_000_000,
        case.tpm_attested,
        case.signatures,
    );

    let attested = match op.verify_safety_and_policy() {
        Ok(a) => a,
        Err(err) => return ProscriptionVerdict::IllegalityBlockedFailClosed(err),
    };

    match attested.settle_statutory_ledger(case.aaufsa_ctx) {
        Ok(_) => ProscriptionVerdict::UnexpectedPass,
        Err(err) => ProscriptionVerdict::IllegalityBlockedFailClosed(err),
    }
}

fn main() -> ExitCode {
    let battery = build_proscription_test_battery();
    let mut blocked_count: u32 = 0;

    let mut out = String::with_capacity(4096);
    out.push_str("{\n");
    out.push_str("  \"deployment_state\": \"SOVEREIGN_PROSCRIPTION_ENFORCEMENT_ACTIVE_100PCT\",\n");
    out.push_str(&format!("  \"holding_org_id\": \"{ORG_HOLDING_ID}\",\n"));
    out.push_str(
        "  \"enforcement_doctrine\": \"Make Illegal States Unrepresentable + Fail-Closed\",\n",
    );
    out.push_str("  \"proscribed_cases_verified\": [\n");

    for (idx, case) in battery.iter().enumerate() {
        let verdict = enforce_proscription_case(case);
        out.push_str("    {\n");
        out.push_str(&format!("      \"case_id\": \"{}\",\n", case.case_id));
        out.push_str(&format!(
            "      \"domain_fqdn\": \"{}\",\n",
            case.domain_fqdn
        ));
        out.push_str(&format!(
            "      \"tenant_id\": \"{}\",\n",
            case.domain.tenant_id()
        ));
        out.push_str(&format!(
            "      \"proscribed_act\": \"{}\",\n",
            case.proscribed_act
        ));
        out.push_str(&format!(
            "      \"statutory_basis\": \"{}\",\n",
            case.statutory_basis
        ));

        match verdict {
            ProscriptionVerdict::IllegalityBlockedFailClosed(err) if err == case.expected_error => {
                blocked_count = blocked_count.saturating_add(1);
                out.push_str("      \"enforcement_status\": \"ILLEGALIZED_AND_BLOCKED_100PCT\",\n");
                out.push_str(&format!("      \"deterministic_error\": \"{err:?}\"\n"));
            }
            _ => {
                eprintln!("[FATAL] Fallo en proscripción de caso {}", case.case_id);
                return ExitCode::from(1);
            }
        }

        if idx < 5 {
            out.push_str("    },\n");
        } else {
            out.push_str("    }\n");
        }
    }

    out.push_str("  ],\n");
    out.push_str(&format!(
        "  \"total_illegal_states_neutralized\": {blocked_count}\n}}\n"
    ));

    if let Ok(mut f) = File::create("sovereign_proscription_production_state.json") {
        let _ = f.write_all(out.as_bytes());
    }
    println!("{out}");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod proscription_tests {
    use super::*;

    #[test]
    fn test_all_six_illegal_states_are_blocked_fail_closed() {
        let battery = build_proscription_test_battery();
        for case in &battery {
            assert_eq!(
                enforce_proscription_case(case),
                ProscriptionVerdict::IllegalityBlockedFailClosed(case.expected_error)
            );
        }
    }
}
