<!--
SPDX-License-Identifier: LicenseRef-AuraIntelCore-Proprietary
Copyright (c) 2026 José Emanuel Guadalupe López (JEGL) - AuraIntelCore / Standard 1979. All Rights Reserved.
-->

# MARCO REGULATORIO DE INDICADORES, RECAPITULACIONES, CONTRATO HÍBRIDO B2B Y LIQUIDACIÓN (100% WORM)

**Titular y Autor:** José Emanuel Guadalupe López (JEGL)  
**Ecosistema:** AuraIntelCore.com | Standard1979.com | Fourphonix-com (964521086569) | AuraTetrax-com (877771073297)  
**Modelo Comercial Activo:** Híbrido Acumulativo (Suscripción Fija Mensual Tiers 1-4 + 21% Comisión por Desempeño M&V)  
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

## 2. CONTRATO MAESTRO B2B DE DOBLE FLUJO: SUSCRIPCIÓN TIERS 1-4 + 21% DESEMPEÑO (MSA / SLA)

1. **Estructura Tarifaria Híbrida Acumulativa:** Toda planta eléctrica, microrred o entidad industrial contratante se obliga al pago conjunto y acumulativo de: (a) La **Suscripción Mensual Fija por Nivel (Tiers 1 a 4: `S0d`, `S0e`, `S0f`, `S0c`)** por concepto de licenciamiento SaaS, telemetría determinista y disponibilidad de plataforma, MÁS (b) La **Comisión por Desempeño del 21.00% (`2100 bps`)** calculada sobre el 100% de los ahorros energéticos e ingresos mayoristas verificados en cada ciclo de facturación.
2. **Partición Exacta en Punto Fijo (21% / 79%):** Sobre el ahorro/beneficio bruto verificado conforme a IPMVP Opción C / ASHRAE Guideline 14, el motor determinista aplica aritmética entera (`i128`) asignando `2100 bps` (21.00%) adicionales a la plataforma y `7900 bps` (79.00%) de beneficio neto retenido por el cliente industrial. Todo residuo entero (`rem < 10_000`) se adjudica a favor del cliente.
3. **Validez Probatoria WORM e Interbloqueo (`XCBR1`):** Todo estado de cuenta y factura recapitulativa se respalda en bitácoras selladas bajo permisos `0400` y verificadas mediante `SOVEREIGN_COMPLIANCE_MANIFEST.sha256`. Cuando `vpp_breaker_status == 0` (`OPEN/TRIP`), el optimizador suspende de forma determinista el despacho activo sin penalización contractual.

---

## 3. ECUACIÓN DETERMINISTA DE LIQUIDACIÓN HÍBRIDA Y RECAPITULACIÓN CONTABLE

- **Paso 1 (Ingestión Metrológica):** Telemetría `:8000` / IEC 61850 `:10102` con mTLS, IEC 62351 y ventana semiabierta `[t_inicio, t_fin)` en UTC RFC 3339.
- **Paso 2 (Motor Zero-Float `i128` | `MICRO_SCALE = 1_000_000_u128`):**
  - `Cuota_Base_uUSD = Tarifa_Mensual_Tier_1_a_4_uUSD`
  - `Ahorro_Total_uUSD = Baseline_IPMVP_uUSD - Consumo_Optimizado_uUSD`
  - `Comision_Desempeno_21pct_uUSD = (Ahorro_Total_uUSD * 2100) / 10000`
  - `Beneficio_Neto_Cliente_79pct_uUSD = Ahorro_Total_uUSD - Comision_Desempeno_21pct_uUSD`
  - `Total_Factura_B2B_uUSD = Cuota_Base_uUSD + Comision_Desempeno_21pct_uUSD + Impuestos_Indirectos_uUSD`
- **Paso 3 (Asiento ACID y Cobro Stripe Live):** Escritura inmutable en `~/ledger_audit_vault/*.jsonl` (`chmod 0400`), emisión de factura PEPPOL BIS Billing 3.0 / UBL 2.1 y ejecución en Stripe Live.
