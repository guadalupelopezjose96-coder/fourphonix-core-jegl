// José Emanuel Guadalupe López (JEGL) | SPDX-License-Identifier: LicenseRef-AuraIntelCore-Proprietary
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

pub const PRODUCTION_FOLDER_ID: &str = "978426653396";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityValidationError {
    PriorityInversionDetected,
    DeadlineWcEtExceeded,
    SandboxOrNonProdPreemptionAttempt,
    UnderlyingSovereignError(SovereignError),
}

struct DomainPrioritySlot {
    slot_code: &'static str,
    domain: CorporateDomain,
    domain_fqdn: &'static str,
    source_folder: LifecycleFolder,
    priority_tier: u8,
    resource_ceiling_tier: u8,
    dscp_class: &'static str,
    wcet_budget_micros: i128,
    observed_exec_micros: i128,
    idempotency_key: [u8; 32],
    vector: PlatformVector,
    gross_settlement_micros: i128,
    aaufsa_ctx: AaufsaDeploymentContext,
}

#[must_use]
pub fn format_micros(micros: i128) -> String {
    let abs_val = micros.unsigned_abs();
    let whole = abs_val / 1_000_000_u128;
    let frac = abs_val % 1_000_000_u128;
    format!("{whole}.{frac:06}")
}

fn build_all_domains_priority_schedule() -> [DomainPrioritySlot; 7] {
    [
        DomainPrioritySlot {
            slot_code: "PRIO-01-AURAINTELCORE-TIER0",
            domain: CorporateDomain::AuraIntelCore,
            domain_fqdn: "auraintelcore.com",
            source_folder: LifecycleFolder::Production,
            priority_tier: 0,
            resource_ceiling_tier: 0,
            dscp_class: "CS7 (SIL-4 Override & VPP Core)",
            wcet_budget_micros: 1_000,
            observed_exec_micros: 420,
            idempotency_key: [0xD1; 32],
            vector: PlatformVector::GeminiCognitive {
                store_false_enforced: true,
                prompt_shield_passed: true,
                tokens_consumed: 8_500_000,
            },
            gross_settlement_micros: 340_000_000_000,
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
        },
        DomainPrioritySlot {
            slot_code: "PRIO-02-AURATETRAX-TIER1",
            domain: CorporateDomain::AuraTetrax,
            domain_fqdn: "auratetrax.com",
            source_folder: LifecycleFolder::Production,
            priority_tier: 1,
            resource_ceiling_tier: 1,
            dscp_class: "CS6 (IEC 61850 GOOSE / PMU)",
            wcet_budget_micros: 3_000,
            observed_exec_micros: 1_150,
            idempotency_key: [0xD2; 32],
            vector: PlatformVector::VppEnergetics {
                frequency_micro_hz: 60_000_000,
                energy_dispatched_micro_wh: 310_000_000_000,
                anti_islanding_armed: true,
            },
            gross_settlement_micros: 280_000_000_000,
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
        },
        DomainPrioritySlot {
            slot_code: "PRIO-03-FOURPHONIX-TIER1",
            domain: CorporateDomain::Fourphonix,
            domain_fqdn: "fourphonix.com",
            source_folder: LifecycleFolder::Production,
            priority_tier: 1,
            resource_ceiling_tier: 1,
            dscp_class: "CS6 (IEEE 802.1Qbv TSN + ML-KEM-768)",
            wcet_budget_micros: 2_000,
            observed_exec_micros: 890,
            idempotency_key: [0xD3; 32],
            vector: PlatformVector::PostQuantumLink {
                ml_kem_768_active: true,
                macsec_active: true,
                bytes_transferred: 75_000_000_000_000,
            },
            gross_settlement_micros: 260_000_000_000,
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
        },
        DomainPrioritySlot {
            slot_code: "PRIO-04-PROPHONIX-TIER2",
            domain: CorporateDomain::Prophonix,
            domain_fqdn: "prophonix.com",
            source_folder: LifecycleFolder::Production,
            priority_tier: 2,
            resource_ceiling_tier: 1,
            dscp_class: "EF (AES67 / EBU R128 Real-Time)",
            wcet_budget_micros: 5_000,
            observed_exec_micros: 2_100,
            idempotency_key: [0xD4; 32],
            vector: PlatformVector::SonicLoudness {
                integrated_micro_lufs: -23_000_000,
                true_peak_micro_dbtp: -1_400_000,
            },
            gross_settlement_micros: 195_000_000_000,
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
        },
        DomainPrioritySlot {
            slot_code: "PRIO-05-STUDIOS3D-TIER2",
            domain: CorporateDomain::AuraIntelStudios3D,
            domain_fqdn: "auraintelstudios3d.com",
            source_folder: LifecycleFolder::Production,
            priority_tier: 2,
            resource_ceiling_tier: 2,
            dscp_class: "AF41 (Spatial 3D & Binaural HRTF)",
            wcet_budget_micros: 8_000,
            observed_exec_micros: 4_300,
            idempotency_key: [0xD5; 32],
            vector: PlatformVector::SonicLoudness {
                integrated_micro_lufs: -16_000_000,
                true_peak_micro_dbtp: -1_200_000,
            },
            gross_settlement_micros: 220_000_000_000,
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
        },
        DomainPrioritySlot {
            slot_code: "PRIO-06-STANDARD1979-TIER0",
            domain: CorporateDomain::Standard1979,
            domain_fqdn: "standard1979.com",
            source_folder: LifecycleFolder::Production,
            priority_tier: 0,
            resource_ceiling_tier: 0,
            dscp_class: "CS7 (IRC Sec. 7501 First Lien & WORM)",
            wcet_budget_micros: 1_500,
            observed_exec_micros: 610,
            idempotency_key: [0xD6; 32],
            vector: PlatformVector::NormativeCertification {
                worm_locked: true,
                ml_dsa_signed: true,
            },
            gross_settlement_micros: 180_000_000_000,
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
        },
        DomainPrioritySlot {
            slot_code: "PRIO-07-INVERSION-GUARD-TEST",
            domain: CorporateDomain::Standard1979,
            domain_fqdn: "sandbox-inversion-test.internal",
            source_folder: LifecycleFolder::Sandbox,
            priority_tier: 0,
            resource_ceiling_tier: 4,
            dscp_class: "BE (Unprivileged Sandbox)",
            wcet_budget_micros: 1_000,
            observed_exec_micros: 2_500,
            idempotency_key: [0xD7; 32],
            vector: PlatformVector::NormativeCertification {
                worm_locked: true,
                ml_dsa_signed: true,
            },
            gross_settlement_micros: 50_000_000_000,
            aaufsa_ctx: AaufsaDeploymentContext::ExternalCloudSaasProvider,
        },
    ]
}

fn arbitrate_and_settle_slot(
    slot: &DomainPrioritySlot,
) -> Result<SovereignOperation<StatutorilySettledCommit>, PriorityValidationError> {
    if slot.source_folder != LifecycleFolder::Production {
        return Err(PriorityValidationError::SandboxOrNonProdPreemptionAttempt);
    }
    if slot.priority_tier < slot.resource_ceiling_tier {
        return Err(PriorityValidationError::PriorityInversionDetected);
    }
    if slot.observed_exec_micros <= 0 || slot.observed_exec_micros > slot.wcet_budget_micros {
        return Err(PriorityValidationError::DeadlineWcEtExceeded);
    }
    let unverified = SovereignOperation::new_unverified(
        slot.idempotency_key,
        slot.source_folder,
        slot.domain,
        slot.vector,
        slot.gross_settlement_micros,
        true,
        2,
    );
    let attested: SovereignOperation<AttestedAndCompliantAction> = unverified
        .verify_safety_and_policy()
        .map_err(PriorityValidationError::UnderlyingSovereignError)?;
    attested
        .settle_statutory_ledger(slot.aaufsa_ctx)
        .map_err(PriorityValidationError::UnderlyingSovereignError)
}

fn main() -> ExitCode {
    let schedule = build_all_domains_priority_schedule();
    let mut total_settled_micros: i128 = 0;
    let mut out = String::with_capacity(4096);
    out.push_str("{\n");
    out.push_str("  \"deployment_state\": \"SOVEREIGN_PRIORITY_ARBITRATION_ACTIVE_100PCT\",\n");
    out.push_str(&format!("  \"holding_org_id\": \"{ORG_HOLDING_ID}\",\n"));
    out.push_str(&format!(
        "  \"lifecycle_folder\": \"02-Production ({PRODUCTION_FOLDER_ID})\",\n"
    ));
    out.push_str("  \"domain_priority_slots\": [\n");

    for (idx, slot) in schedule.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str(&format!("      \"slot_code\": \"{}\",\n", slot.slot_code));
        out.push_str(&format!(
            "      \"domain_fqdn\": \"{}\",\n",
            slot.domain_fqdn
        ));
        out.push_str(&format!(
            "      \"tenant_id\": \"{}\",\n",
            slot.domain.tenant_id()
        ));
        out.push_str(&format!(
            "      \"priority_tier\": {},\n",
            slot.priority_tier
        ));
        out.push_str(&format!("      \"dscp_class\": \"{}\",\n", slot.dscp_class));
        match arbitrate_and_settle_slot(slot) {
            Ok(settled) => {
                total_settled_micros =
                    total_settled_micros.saturating_add(settled.gross_amount_micros);
                out.push_str("      \"status\": \"PREEMPTIVELY_DISPATCHED_AND_SETTLED\",\n");
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
        if idx < 6 {
            out.push_str("    },\n");
        } else {
            out.push_str("    }\n");
        }
    }
    out.push_str("  ],\n");
    out.push_str(&format!(
        "  \"total_priority_arbitrated_usd\": \"{}\"\n}}\n",
        format_micros(total_settled_micros)
    ));
    if let Ok(mut f) = File::create("sovereign_priority_production_state.json") {
        let _ = f.write_all(out.as_bytes());
    }
    println!("{out}");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod priority_tests {
    use super::*;
    #[test]
    fn test_priority_schedule_invariants() {
        let sched = build_all_domains_priority_schedule();
        for (i, slot) in sched.iter().enumerate() {
            if i < 6 {
                assert!(arbitrate_and_settle_slot(slot).is_ok());
            } else {
                assert_eq!(
                    arbitrate_and_settle_slot(slot),
                    Err(PriorityValidationError::SandboxOrNonProdPreemptionAttempt)
                );
            }
        }
    }
}
