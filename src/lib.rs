#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_arithmetic,
    clippy::indexing_slicing
)]
#![allow(unused_imports, dead_code, clippy::all, clippy::pedantic)]

use core::marker::PhantomData;

pub const MICRO_SCALE: i128 = 1_000_000;
pub const AAUFSA_SMALL_BUSINESS_THRESHOLD_MICROS: i128 = 5_000_000_000_000;
pub const AAUFSA_TAX_RATE_NUMERATOR: i128 = 1_530;
pub const AAUFSA_TAX_RATE_DENOMINATOR: i128 = 10_000;
pub const AAUFSA_PERMANENT_DISPLACEMENT_DAYS: u32 = 90;
pub const AAUFSA_SAFE_HARBOR_MONTHS: u32 = 36;
pub const AAUFSA_TRUST_OASI_DI_PERCENT: i128 = 75;
pub const AAUFSA_TRUST_RESKILLING_PERCENT: i128 = 25;
pub const VPP_MIN_FREQ_MICRO_HZ: i128 = 59_950_000;
pub const VPP_MAX_FREQ_MICRO_HZ: i128 = 60_050_000;
pub const MAX_ALLOWED_TRUE_PEAK_MICRO_DBTP: i128 = -1_000_000;
pub const ORG_HOLDING_ID: &str = "320717657398";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleFolder {
    SharedServices,
    Production,
    NonProduction,
    Sandbox,
}

impl LifecycleFolder {
    #[must_use]
    pub const fn folder_id(&self) -> &'static str {
        match self {
            Self::SharedServices => "938478392755",
            Self::Production => "978426653396",
            Self::NonProduction => "467686749558",
            Self::Sandbox => "10470744625",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorporateDomain {
    AuraTetrax,
    Fourphonix,
    AuraIntelCore,
    AuraIntelStudios3d,
    Standard1979,
    Prophonix,
    AuraIntelStudios3D,
}

impl CorporateDomain {
    #[must_use]
    pub const fn tenant_id(&self) -> &'static str {
        match self {
            Self::AuraTetrax => "877771073297",
            Self::Fourphonix => "964521086569",
            Self::AuraIntelCore | Self::AuraIntelStudios3D => "188680247367",
            Self::AuraIntelStudios3d => "1087754464927",
            Self::Standard1979 => "282230018778",
            Self::Prophonix => "358233233",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SovereignError {
    ArithmeticOverflow,
    InvalidIdempotencyKey,
    ZeroOrNegativeAmount,
    UntrustedHardwareAttestation,
    DualSignatureMissingInProduction,
    SandboxEscapeAttempt,
    NonProdToProductionEscalation,
    CrossTenantAccessViolation,
    GeminiDataRetentionViolation,
    PromptInjectionDetected,
    VppFrequencyOutOfEnvelope,
    VppAntiIslandingDisarmed,
    LoudnessTruePeakExceeded,
    UnencryptedNetworkLink,
    WormImmutabilityNotEnforced,
    UnbalancedDoubleEntryLedger,
    ClawbackEscrowUnderfunded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformVector {
    GeminiCognitive {
        store_false_enforced: bool,
        prompt_shield_passed: bool,
        tokens_consumed: i128,
    },
    VppEnergetics {
        frequency_micro_hz: i128,
        energy_dispatched_micro_wh: i128,
        anti_islanding_armed: bool,
    },
    SonicLoudness {
        integrated_micro_lufs: i128,
        true_peak_micro_dbtp: i128,
    },
    PostQuantumLink {
        ml_kem_768_active: bool,
        macsec_active: bool,
        bytes_transferred: i128,
    },
    NormativeCertification {
        worm_locked: bool,
        ml_dsa_signed: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AaufsaDeploymentContext {
    ExternalCloudSaasProvider,
    InternalCorporateOperation {
        consolidated_group_revenue_micros: i128,
        displaced_fte_annual_bls_wage_micros: i128,
        displacement_duration_days: u32,
        reassigned_without_wage_reduction: bool,
        reassignment_completed_months: u32,
        early_termination_triggered: bool,
        clawback_escrow_balance_micros: i128,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct AaufsaTaxDetermination {
    pub cloud_saas_exempt: bool,
    pub small_business_exempt: bool,
    pub safe_harbor_applied: bool,
    pub clawback_triggered: bool,
    pub total_chapter56_tax_micros: i128,
    pub oasi_di_trust_fund_75_micros: i128,
    pub workforce_reskilling_25_micros: i128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnverifiedDomainAction;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttestedAndCompliantAction;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatutorilySettledCommit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SovereignOperation<S> {
    pub idempotency_key: [u8; 32],
    pub folder: LifecycleFolder,
    pub domain: CorporateDomain,
    pub vector: PlatformVector,
    pub gross_amount_micros: i128,
    pub hardware_tpm_attested: bool,
    pub dual_custodian_signatures: u8,
    pub tax_determination: Option<AaufsaTaxDetermination>,
    _state: PhantomData<S>,
}

impl SovereignOperation<UnverifiedDomainAction> {
    #[must_use]
    pub const fn new_unverified(
        idempotency_key: [u8; 32],
        folder: LifecycleFolder,
        domain: CorporateDomain,
        vector: PlatformVector,
        gross_amount_micros: i128,
        hardware_tpm_attested: bool,
        dual_custodian_signatures: u8,
    ) -> Self {
        Self {
            idempotency_key,
            folder,
            domain,
            vector,
            gross_amount_micros,
            hardware_tpm_attested,
            dual_custodian_signatures,
            tax_determination: None,
            _state: PhantomData,
        }
    }

    /// # Errors
    /// Retorna `SovereignError` en modo Fail-Closed ante cualquier violación de invariantes.
    pub fn verify_safety_and_policy(
        self,
    ) -> Result<SovereignOperation<AttestedAndCompliantAction>, SovereignError> {
        if self.idempotency_key == [0u8; 32] {
            return Err(SovereignError::InvalidIdempotencyKey);
        }
        if self.gross_amount_micros <= 0 {
            return Err(SovereignError::ZeroOrNegativeAmount);
        }
        if !self.hardware_tpm_attested {
            return Err(SovereignError::UntrustedHardwareAttestation);
        }
        if self.folder == LifecycleFolder::Production && self.dual_custodian_signatures < 2 {
            return Err(SovereignError::DualSignatureMissingInProduction);
        }

        match self.vector {
            PlatformVector::GeminiCognitive {
                store_false_enforced,
                prompt_shield_passed,
                tokens_consumed,
            } => {
                if !store_false_enforced {
                    return Err(SovereignError::GeminiDataRetentionViolation);
                }
                if !prompt_shield_passed {
                    return Err(SovereignError::PromptInjectionDetected);
                }
                if tokens_consumed <= 0 {
                    return Err(SovereignError::ZeroOrNegativeAmount);
                }
            }
            PlatformVector::VppEnergetics {
                frequency_micro_hz,
                energy_dispatched_micro_wh,
                anti_islanding_armed,
            } => {
                if !anti_islanding_armed {
                    return Err(SovereignError::VppAntiIslandingDisarmed);
                }
                if !(VPP_MIN_FREQ_MICRO_HZ..=VPP_MAX_FREQ_MICRO_HZ).contains(&frequency_micro_hz) {
                    return Err(SovereignError::VppFrequencyOutOfEnvelope);
                }
                if energy_dispatched_micro_wh <= 0 {
                    return Err(SovereignError::ZeroOrNegativeAmount);
                }
            }
            PlatformVector::SonicLoudness {
                true_peak_micro_dbtp,
                ..
            } => {
                if true_peak_micro_dbtp > MAX_ALLOWED_TRUE_PEAK_MICRO_DBTP {
                    return Err(SovereignError::LoudnessTruePeakExceeded);
                }
            }
            PlatformVector::PostQuantumLink {
                ml_kem_768_active,
                macsec_active,
                bytes_transferred,
            } => {
                if !ml_kem_768_active || !macsec_active {
                    return Err(SovereignError::UnencryptedNetworkLink);
                }
                if bytes_transferred <= 0 {
                    return Err(SovereignError::ZeroOrNegativeAmount);
                }
            }
            PlatformVector::NormativeCertification {
                worm_locked,
                ml_dsa_signed,
            } => {
                if !worm_locked || !ml_dsa_signed {
                    return Err(SovereignError::WormImmutabilityNotEnforced);
                }
            }
        }

        Ok(SovereignOperation {
            idempotency_key: self.idempotency_key,
            folder: self.folder,
            domain: self.domain,
            vector: self.vector,
            gross_amount_micros: self.gross_amount_micros,
            hardware_tpm_attested: self.hardware_tpm_attested,
            dual_custodian_signatures: self.dual_custodian_signatures,
            tax_determination: None,
            _state: PhantomData,
        })
    }
}

impl SovereignOperation<AttestedAndCompliantAction> {
    /// # Errors
    /// Retorna `SovereignError` si ocurre desbordamiento o falta de Escrow en Clawback.
    pub fn settle_statutory_ledger(
        self,
        context: AaufsaDeploymentContext,
    ) -> Result<SovereignOperation<StatutorilySettledCommit>, SovereignError> {
        let determination = evaluate_aaufsa_chapter56(context)?;
        Ok(SovereignOperation {
            idempotency_key: self.idempotency_key,
            folder: self.folder,
            domain: self.domain,
            vector: self.vector,
            gross_amount_micros: self.gross_amount_micros,
            hardware_tpm_attested: self.hardware_tpm_attested,
            dual_custodian_signatures: self.dual_custodian_signatures,
            tax_determination: Some(determination),
            _state: PhantomData,
        })
    }
}

/// # Errors
/// Retorna `SovereignError` ante desbordamiento aritmético o Clawback sin provisión suficiente.
pub fn evaluate_aaufsa_chapter56(
    context: AaufsaDeploymentContext,
) -> Result<AaufsaTaxDetermination, SovereignError> {
    match context {
        AaufsaDeploymentContext::ExternalCloudSaasProvider => Ok(AaufsaTaxDetermination {
            cloud_saas_exempt: true,
            small_business_exempt: false,
            safe_harbor_applied: false,
            clawback_triggered: false,
            total_chapter56_tax_micros: 0,
            oasi_di_trust_fund_75_micros: 0,
            workforce_reskilling_25_micros: 0,
        }),
        AaufsaDeploymentContext::InternalCorporateOperation {
            consolidated_group_revenue_micros,
            displaced_fte_annual_bls_wage_micros,
            displacement_duration_days,
            reassigned_without_wage_reduction,
            reassignment_completed_months,
            early_termination_triggered,
            clawback_escrow_balance_micros,
        } => {
            if consolidated_group_revenue_micros <= AAUFSA_SMALL_BUSINESS_THRESHOLD_MICROS {
                return Ok(AaufsaTaxDetermination {
                    cloud_saas_exempt: false,
                    small_business_exempt: true,
                    safe_harbor_applied: false,
                    clawback_triggered: false,
                    total_chapter56_tax_micros: 0,
                    oasi_di_trust_fund_75_micros: 0,
                    workforce_reskilling_25_micros: 0,
                });
            }
            if displacement_duration_days <= AAUFSA_PERMANENT_DISPLACEMENT_DAYS
                || displaced_fte_annual_bls_wage_micros <= 0
            {
                return Ok(AaufsaTaxDetermination {
                    cloud_saas_exempt: false,
                    small_business_exempt: false,
                    safe_harbor_applied: false,
                    clawback_triggered: false,
                    total_chapter56_tax_micros: 0,
                    oasi_di_trust_fund_75_micros: 0,
                    workforce_reskilling_25_micros: 0,
                });
            }
            let base_tax = displaced_fte_annual_bls_wage_micros
                .checked_mul(AAUFSA_TAX_RATE_NUMERATOR)
                .ok_or(SovereignError::ArithmeticOverflow)?
                .checked_div(AAUFSA_TAX_RATE_DENOMINATOR)
                .ok_or(SovereignError::ArithmeticOverflow)?;

            if reassigned_without_wage_reduction
                && !early_termination_triggered
                && reassignment_completed_months >= AAUFSA_SAFE_HARBOR_MONTHS
            {
                return Ok(AaufsaTaxDetermination {
                    cloud_saas_exempt: false,
                    small_business_exempt: false,
                    safe_harbor_applied: true,
                    clawback_triggered: false,
                    total_chapter56_tax_micros: 0,
                    oasi_di_trust_fund_75_micros: 0,
                    workforce_reskilling_25_micros: 0,
                });
            }
            let clawback = reassigned_without_wage_reduction && early_termination_triggered;
            if clawback && clawback_escrow_balance_micros < base_tax {
                return Err(SovereignError::ClawbackEscrowUnderfunded);
            }
            let oasi_75 = base_tax
                .checked_mul(AAUFSA_TRUST_OASI_DI_PERCENT)
                .ok_or(SovereignError::ArithmeticOverflow)?
                .checked_div(100)
                .ok_or(SovereignError::ArithmeticOverflow)?;
            let reskilling_25 = base_tax
                .checked_sub(oasi_75)
                .ok_or(SovereignError::ArithmeticOverflow)?;

            Ok(AaufsaTaxDetermination {
                cloud_saas_exempt: false,
                small_business_exempt: false,
                safe_harbor_applied: false,
                clawback_triggered: clawback,
                total_chapter56_tax_micros: base_tax,
                oasi_di_trust_fund_75_micros: oasi_75,
                workforce_reskilling_25_micros: reskilling_25,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aaufsa_tax_and_75_25_split_calculation() {
        let wage_micros: i128 = 100_000 * MICRO_SCALE;
        let context = AaufsaDeploymentContext::InternalCorporateOperation {
            consolidated_group_revenue_micros: 12_000_000 * MICRO_SCALE,
            displaced_fte_annual_bls_wage_micros: wage_micros,
            displacement_duration_days: 120,
            reassigned_without_wage_reduction: false,
            reassignment_completed_months: 0,
            early_termination_triggered: false,
            clawback_escrow_balance_micros: 0,
        };
        let result = evaluate_aaufsa_chapter56(context);
        assert!(result.is_ok());
        if let Ok(det) = result {
            assert_eq!(det.total_chapter56_tax_micros, 15_300_000_000);
            assert_eq!(det.oasi_di_trust_fund_75_micros, 11_475_000_000);
            assert_eq!(det.workforce_reskilling_25_micros, 3_825_000_000);
        }
    }
}
