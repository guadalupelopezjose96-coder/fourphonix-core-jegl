<!--
SPDX-License-Identifier: LicenseRef-AuraIntelCore-Proprietary
Copyright (c) 2026 José Emanuel Guadalupe López (JEGL) - AuraIntelCore / Standard 1979. All Rights Reserved.
-->

# MARCO REGULATORIO DE INDICADORES, RECAPITULACIONES, CONTRATO B2B Y LIQUIDACIÓN (100% WORM)

**Titular y Autor:** José Emanuel Guadalupe López (JEGL)  
**Ecosistema:** AuraIntelCore.com | Standard1979.com | Fourphonix-com (964521086569) | AuraTetrax-com (877771073297)  
**Estado de Cumplimiento:** 100% Legal | 100% Lícito | 100% Determinista | Zero-Float Drift (i64/i128)  

---

## 1. ESTÁNDAR METROLÓGICO Y NORMATIVO DE INDICADORES (SI / IEEE / IEC / FERC)

| Indicador / Métrica | Unidad SI / Escala Entera | Rango / Invariante | Norma Técnica y Regulatoria |
| :--- | :--- | :--- | :--- |
| `vpp_bess_discharge_headroom_kw` | `mW` / `uW` (`i64`) | `0` a `500 kW` | IEEE 1547-2018 / FERC Order 2222 |
| `vpp_active_power_kw` | `mW` / `uW` (`i64`) | Medición Fiscal 0.2s | ANSI C12.20 / IEC 62053-22 / IEEE 1588 PTP |
| `vpp_predicted_solar_kw` | `mW` (`i64`) | `>= 0 kW` (SHA-256 IA) | FastAPI Pydantic Strict Schema Gate |
| `vpp_weather_irradiance_wm2` | `mW/m2` (`i64`) | `0` a `1361 W/m2` | ISO 9060 / WMO Pyranometer Standard |
| `vpp_breaker_status` | Estado Booleano (`u8`) | `1=CLOSED`, `0=OPEN` | IEC 61850 (`XCBR1`) / IEC 62351 mTLS |
| `vpp_market_price_usd_mwh` | `micro-USD` (`i128`) | `1_000_000_u128` scale | Wholesale ISO/TSO Locational Marginal Price |
| `vpp_wholesale_revenue_usd_h` | `micro-USD/h` (`i128`) | Partida Doble ACID | IRC Ch. 56 / PEPPOL BIS Billing 3.0 |
| `vpp_saas_client_savings_usd_h` | `micro-USD/h` (`i128`) | Split `2100/7900 bps` | IPMVP Option C / ASHRAE Guideline 14 |

---

## 2. CONTRATO MAESTRO DE DESEMPEÑO Y SUSCRIPCIÓN B2B (MSA / SLA)

1. **Validez Metrológica y Bóveda WORM:** Todo cargo por suscripción (Tiers 1 a 4: `S0d`, `S0e`, `S0f`, `S0c`) o por desempeño compartido (21% VPP Shared Savings) se sustenta en registros inmutables sellados bajo permisos `0400` (Write-Once-Read-Many) y verificados mediante `SOVEREIGN_COMPLIANCE_MANIFEST.sha256`.
2. **Partición Exacta de Ahorros (21% / 79%):** En el modelo de ahorros compartidos, el motor determinista aplica aritmética entera de punto fijo (`i128`) asignando `2100 bps` (21.00%) a la plataforma y `7900 bps` (79.00%) al cliente industrial. Todo residuo de división entera (`rem < 10_000`) se adjudica a favor del cliente para garantizar cero cobros excedentes.
3. **Interbloqueo de Seguridad Operativa (`XCBR1`):** Cuando `vpp_breaker_status == 0` (`OPEN/TRIP`), el optimizador bloquea de forma determinista cualquier despacho activo hacia la subestación.

---

## 3. FLUJO DETERMINISTA DE FACTURACIÓN Y LIQUIDACIÓN (STRIPE LIVE & PEPPOL)

- **Paso 1 (Ingestión Edge):** Telemetría `:8000` / IEC 61850 `:10102` con mTLS, IEC 62351 y Timestamp UTC RFC 3339.
- **Paso 2 (Motor M&V Zero-Float `i128` | `MICRO_SCALE = 1_000_000_u128`):**
  - `Ahorro_Total_uUSD = Baseline_uUSD - Actual_uUSD`
  - `Cargo_Plataforma_21pct = (Ahorro_Total_uUSD * 2100) / 10000`
  - `Beneficio_Cliente_79pct = Ahorro_Total_uUSD - Cargo_Plataforma_21pct`
- **Paso 3 (Liquidación y Bóveda Forense):** Asiento contable ACID de partida doble, escritura WORM en `~/ledger_audit_vault/*.jsonl` (`chmod 0400`), emisión PEPPOL BIS Billing 3.0 / UBL 2.1 y cobro automatizado vía Stripe Live.

---

## 4. ESTÁNDAR DE RECAPITULACIONES DETERMINISTAS Y CIERRES CONTABLES (ZERO-FINDINGS)

1. **Ventanas Temporales Semiabiertas:** Todo corte horario, diario o mensual se evalúa sobre el intervalo semiabierto `[t_inicio, t_fin)` en UTC RFC 3339 sincronizado por IEEE 1588 PTP para impedir doble conteo u omisión de eventos.
2. **Agregación Sin Deriva (1:1):** El total de toda recapitulación es la suma entera exacta (`i128`) de los asientos primarios en `~/ledger_audit_vault/*.jsonl`, prohibiendo el re-redondeo en coma flotante (`#![deny(clippy::float_arithmetic)]`).
3. **Conciliación Trilateral Inmutable:** Toda recapitulación emitida mantiene equivalencia matemática 1:1 entre (a) Telemetría M&V verificada, (b) Libro Mayor de Partida Doble WORM y (c) Facturación PEPPOL / Stripe Live, sellada criptográficamente en `SOVEREIGN_COMPLIANCE_MANIFEST.sha256` bajo permisos `0400`.
