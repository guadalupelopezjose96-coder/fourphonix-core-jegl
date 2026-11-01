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
    evaluate_aaufsa_chapter56, AaufsaDeploymentContext, AaufsaTaxDetermination, CorporateDomain,
    SovereignError, AAUFSA_SAFE_HARBOR_MONTHS, AAUFSA_SMALL_BUSINESS_THRESHOLD_MICROS,
    AAUFSA_TAX_RATE_NUMERATOR, MICRO_SCALE, ORG_HOLDING_ID,
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
pub fn to_hex(bytes: &[u8]) -> String {
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
pub fn format_usd(micros: i128) -> String {
    let abs_val = micros.unsigned_abs();
    let whole = abs_val / 1_000_000_u128;
    let frac = abs_val % 1_000_000_u128;
    format!("{whole}.{frac:06}")
}

struct Form8999ScheduleLine {
    schedule_id: &'static str,
    domain: CorporateDomain,
    domain_name: &'static str,
    statutory_section: &'static str,
    gross_receipts_micros: i128,
    context: AaufsaDeploymentContext,
}

fn build_controlled_group_schedules() -> [Form8999ScheduleLine; 6] {
    [
        // 1. AuraIntelCore.com (incluye $1,236,500.00 VPP + $450,000.00 Gemini API)
        Form8999ScheduleLine {
            schedule_id: "SCH-A-01",
            domain: CorporateDomain::AuraIntelCore,
            domain_name: "AuraIntelCore-com (incl. AuraIntel VPP Prod)",
            statutory_section: "IRC Sec. 5000H(d) Cloud/SaaS Provider Immunity",
            gross_receipts_micros: 1_686_500_000_000,
            context: AaufsaDeploymentContext::ExternalCloudSaasProvider,
        },
        // 2. AuraTetrax.com: Safe Harbor de 36 meses cumplido
        Form8999ScheduleLine {
            schedule_id: "SCH-A-02",
            domain: CorporateDomain::AuraTetrax,
            domain_name: "AuraTetrax-com",
            statutory_section: "IRC Sec. 5000G(a) 36-Month Reassignment Safe Harbor",
            gross_receipts_micros: 820_000_000_000,
            context: AaufsaDeploymentContext::InternalCorporateOperation {
                consolidated_group_revenue_micros: 4_311_500_000_000,
                displaced_fte_annual_bls_wage_micros: 240_000_000_000,
                displacement_duration_days: 180,
                reassigned_without_wage_reduction: true,
                reassignment_completed_months: 36,
                early_termination_triggered: false,
                clawback_escrow_balance_micros: 40_000_000_000,
            },
        },
        // 3. Prophonix.com: Desplazamiento gravable al 15.3% ($45,900.00 USD)
        Form8999ScheduleLine {
            schedule_id: "SCH-A-03",
            domain: CorporateDomain::Prophonix,
            domain_name: "Prophonix-com",
            statutory_section: "IRC Sec. 5000F Competitive Neutrality Tax (15.3%)",
            gross_receipts_micros: 610_000_000_000,
            context: AaufsaDeploymentContext::InternalCorporateOperation {
                consolidated_group_revenue_micros: 18_500_000_000_000,
                displaced_fte_annual_bls_wage_micros: 300_000_000_000,
                displacement_duration_days: 145,
                reassigned_without_wage_reduction: false,
                reassignment_completed_months: 0,
                early_termination_triggered: false,
                clawback_escrow_balance_micros: 0,
            },
        },
        // 4. Fourphonix.com: Recaptura Clawback cubierta por Escrow ($30,600.00 USD)
        Form8999ScheduleLine {
            schedule_id: "SCH-A-04",
            domain: CorporateDomain::Fourphonix,
            domain_name: "Fourphonix-com",
            statutory_section: "IRC Sec. 5000G(c) Safe Harbor Clawback Recapture",
            gross_receipts_micros: 530_000_000_000,
            context: AaufsaDeploymentContext::InternalCorporateOperation {
                consolidated_group_revenue_micros: 18_500_000_000_000,
                displaced_fte_annual_bls_wage_micros: 200_000_000_000,
                displacement_duration_days: 210,
                reassigned_without_wage_reduction: true,
                reassignment_completed_months: 20,
                early_termination_triggered: true,
                clawback_escrow_balance_micros: 35_000_000_000,
            },
        },
        // 5. AuraIntel-Global.com: Exención Grupo Controlado <= $5M
        Form8999ScheduleLine {
            schedule_id: "SCH-A-05",
            domain: CorporateDomain::AuraIntelStudios3d,
            domain_name: "AuraIntel-Global-com",
            statutory_section: "IRC Sec. 5000G(b) Small Business Threshold Exemption",
            gross_receipts_micros: 390_000_000_000,
            context: AaufsaDeploymentContext::InternalCorporateOperation {
                consolidated_group_revenue_micros: 4_850_000_000_000,
                displaced_fte_annual_bls_wage_micros: 150_000_000_000,
                displacement_duration_days: 120,
                reassigned_without_wage_reduction: false,
                reassignment_completed_months: 0,
                early_termination_triggered: false,
                clawback_escrow_balance_micros: 0,
            },
        },
        // 6. Standard1979.com: Guardia Fail-Closed ante Clawback sin fondos
        Form8999ScheduleLine {
            schedule_id: "SCH-A-06",
            domain: CorporateDomain::Standard1979,
            domain_name: "Standard1979-com",
            statutory_section: "IRC Sec. 5000G(c) Fail-Closed Pre-Commit Guard",
            gross_receipts_micros: 275_000_000_000,
            context: AaufsaDeploymentContext::InternalCorporateOperation {
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

fn generate_irs_form8999_xml() -> String {
    let schedules = build_controlled_group_schedules();
    let mut total_receipts: i128 = 0;
    let mut total_tax: i128 = 0;
    let mut total_oasi_75: i128 = 0;
    let mut total_reskilling_25: i128 = 0;

    let mut xml = String::with_capacity(6144);
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(
        "<IRSForm8999Return xmlns=\"urn:us:gov:treasury:irs:aaufsa:ch56\" version=\"2026v1.0\">\n",
    );
    xml.push_str("  <FilerHeader>\n");
    xml.push_str(&format!(
        "    <HoldingOrganizationID>{ORG_HOLDING_ID}</HoldingOrganizationID>\n"
    ));
    xml.push_str(
        "    <LegalName>guadalupelopezjose96-org (Tech Arquitecture Pro-Elite)</LegalName>\n",
    );
    xml.push_str("    <StatutoryAct>Automated Asset Utilization and Fiscal Solvency Act of 2026 (IRC Chapter 56)</StatutoryAct>\n");
    xml.push_str("    <AuthorAttribution>Jose Emanuel Guadalupe Lopez (DMCA 17 U.S.C. 1202 - 0% Alteration)</AuthorAttribution>\n");
    xml.push_str(&format!("    <CompetitiveNeutralityRateBPS>{AAUFSA_TAX_RATE_NUMERATOR}</CompetitiveNeutralityRateBPS>\n"));
    xml.push_str(&format!(
        "    <SafeHarborMonths>{AAUFSA_SAFE_HARBOR_MONTHS}</SafeHarborMonths>\n"
    ));
    xml.push_str(&format!(
        "    <ControlledGroupThresholdUSD>{}</ControlledGroupThresholdUSD>\n",
        format_usd(AAUFSA_SMALL_BUSINESS_THRESHOLD_MICROS)
    ));
    xml.push_str("  </FilerHeader>\n");
    xml.push_str("  <ControlledGroupSchedules>\n");

    for item in &schedules {
        xml.push_str(&format!("    <ScheduleLine id=\"{}\">\n", item.schedule_id));
        xml.push_str(&format!(
            "      <DomainName>{}</DomainName>\n",
            item.domain_name
        ));
        xml.push_str(&format!(
            "      <TenantID>{}</TenantID>\n",
            item.domain.tenant_id()
        ));
        xml.push_str(&format!(
            "      <StatutoryProvision>{}</StatutoryProvision>\n",
            item.statutory_section
        ));
        xml.push_str(&format!(
            "      <GrossReceiptsUSD>{}</GrossReceiptsUSD>\n",
            format_usd(item.gross_receipts_micros)
        ));

        match evaluate_aaufsa_chapter56(item.context) {
            Ok(AaufsaTaxDetermination {
                cloud_saas_exempt,
                small_business_exempt,
                safe_harbor_applied,
                clawback_triggered,
                total_chapter56_tax_micros,
                oasi_di_trust_fund_75_micros,
                workforce_reskilling_25_micros,
            }) => {
                total_receipts = total_receipts.saturating_add(item.gross_receipts_micros);
                total_tax = total_tax.saturating_add(total_chapter56_tax_micros);
                total_oasi_75 = total_oasi_75.saturating_add(oasi_di_trust_fund_75_micros);
                total_reskilling_25 =
                    total_reskilling_25.saturating_add(workforce_reskilling_25_micros);

                xml.push_str("      <FilingStatus>ACCEPTED_SERIALIZABLE</FilingStatus>\n");
                xml.push_str(&format!("      <CloudSaasExemptSec5000Hd>{cloud_saas_exempt}</CloudSaasExemptSec5000Hd>\n"));
                xml.push_str(&format!("      <SmallBusinessExemptSec5000Gb>{small_business_exempt}</SmallBusinessExemptSec5000Gb>\n"));
                xml.push_str(&format!("      <SafeHarbor36mSec5000Ga>{safe_harbor_applied}</SafeHarbor36mSec5000Ga>\n"));
                xml.push_str(&format!("      <ClawbackTriggeredSec5000Gc>{clawback_triggered}</ClawbackTriggeredSec5000Gc>\n"));
                xml.push_str(&format!(
                    "      <Chapter56TaxLiabilityUSD>{}</Chapter56TaxLiabilityUSD>\n",
                    format_usd(total_chapter56_tax_micros)
                ));
                xml.push_str(&format!(
                    "      <TrustFundOasiDi75PctUSD>{}</TrustFundOasiDi75PctUSD>\n",
                    format_usd(oasi_di_trust_fund_75_micros)
                ));
                xml.push_str(&format!(
                    "      <TrustFundReskilling25PctUSD>{}</TrustFundReskilling25PctUSD>\n",
                    format_usd(workforce_reskilling_25_micros)
                ));
            }
            Err(err) => {
                xml.push_str("      <FilingStatus>REJECTED_FAIL_CLOSED</FilingStatus>\n");
                xml.push_str(&format!(
                    "      <FailClosedReason>{err:?}</FailClosedReason>\n"
                ));
            }
        }
        xml.push_str("    </ScheduleLine>\n");
    }

    xml.push_str("  </ControlledGroupSchedules>\n");
    xml.push_str("  <PartIIITrustFundSettlementSummary>\n");
    xml.push_str(&format!(
        "    <TotalAcceptedGrossReceiptsUSD>{}</TotalAcceptedGrossReceiptsUSD>\n",
        format_usd(total_receipts)
    ));
    xml.push_str(&format!(
        "    <TotalChapter56TaxDueUSD>{}</TotalChapter56TaxDueUSD>\n",
        format_usd(total_tax)
    ));
    xml.push_str(&format!(
        "    <AllocationOasiAndDiTrustFund75PctUSD>{}</AllocationOasiAndDiTrustFund75PctUSD>\n",
        format_usd(total_oasi_75)
    ));
    xml.push_str(&format!("    <AllocationWorkforceReskillingTrustFund25PctUSD>{}</AllocationWorkforceReskillingTrustFund25PctUSD>\n", format_usd(total_reskilling_25)));
    xml.push_str("  </PartIIITrustFundSettlementSummary>\n");

    let digest = to_hex(&sha256_digest(xml.as_bytes()));
    xml.push_str(&format!(
        "  <CryptographicSealFIPS180_4>{digest}</CryptographicSealFIPS180_4>\n"
    ));
    xml.push_str("</IRSForm8999Return>\n");
    xml
}

fn main() -> ExitCode {
    let xml_doc = generate_irs_form8999_xml();
    if let Ok(mut f) = File::create("irs_form_8999_aaufsa_ch56_return.xml") {
        if f.write_all(xml_doc.as_bytes()).is_err() {
            return ExitCode::from(1);
        }
    } else {
        return ExitCode::from(1);
    }
    println!("{xml_doc}");
    ExitCode::SUCCESS
}

#[cfg(test)]
mod form8999_tests {
    use super::*;

    #[test]
    fn test_irs_form8999_exact_totals_and_blocking() {
        let xml = generate_irs_form8999_xml();
        assert!(xml.contains(
            "<TotalAcceptedGrossReceiptsUSD>4036500.000000</TotalAcceptedGrossReceiptsUSD>"
        ));
        assert!(xml.contains("<TotalChapter56TaxDueUSD>76500.000000</TotalChapter56TaxDueUSD>"));
        assert!(xml.contains("<AllocationOasiAndDiTrustFund75PctUSD>57375.000000</AllocationOasiAndDiTrustFund75PctUSD>"));
        assert!(xml.contains("<AllocationWorkforceReskillingTrustFund25PctUSD>19125.000000</AllocationWorkforceReskillingTrustFund25PctUSD>"));
        assert!(xml.contains("<FailClosedReason>ClawbackEscrowUnderfunded</FailClosedReason>"));
        assert_eq!(
            SovereignError::ClawbackEscrowUnderfunded,
            SovereignError::ClawbackEscrowUnderfunded
        );
    }
}
