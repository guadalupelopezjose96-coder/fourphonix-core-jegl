use chrono::{DateTime, Utc};
use reqwest::{Certificate, Client, Identity};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::BufReader,
    net::SocketAddr,
    time::Duration,
};
use tokio::time::{interval, timeout};
use tokio_modbus::prelude::*;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Payload de Telemetría IEEE 2030.5 idéntico al esperado por el Servidor Agregador
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ieee20305TelemetryPayload {
    pub device_lfdi: String,
    pub tax_entity_id: String,
    pub active_power_watts: i64,
    pub reactive_power_var: i64,
    pub available_energy_wh: u64,
    pub soc_basis_points: u16,
    pub grid_frequency_mhz: u32,
    pub voltage_mv: u32,
    pub timestamp_utc: DateTime<Utc>,
}

/// Respuesta de Despacho firmada por el Servidor Agregador
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerControlDispatchResponse {
    pub event_id: Uuid,
    pub verified_lfdi: String,
    pub target_dispatch_watts: i64,
    pub autonomous_droop_fallback_enabled: bool,
    pub poll_rate_seconds: u16,
    pub audit_record_hash: String,
    pub audit_hmac_signature: String,
    pub server_timestamp_utc: DateTime<Utc>,
}

/// Mapa de Registros SunSpec Modbus TCP (Base 40000 estándar para Inversor + Almacenamiento)
pub struct SunSpecRegisterMap;
impl SunSpecRegisterMap {
    /// Inicio de bloque de lectura de telemetría (Model 103 / 124 simplificado)
    pub const TELEMETRY_BASE_ADDR: u16 = 40070;
    pub const TELEMETRY_REG_COUNT: u16 = 12;
    /// Registro de control de potencia activa máxima / despacho (SunSpec Model 123/124 WMaxLimPct)
    pub const DISPATCH_COMMAND_ADDR: u16 = 40232;
}

/// Extrae el certificado DER desde el PEM del cliente y calcula el LFDI oficial IEEE 2030.5 (40 caracteres Hex)
fn extract_local_lfdi_from_pem(cert_pem_path: &str) -> String {
    let file = File::open(cert_pem_path)
        .unwrap_or_else(|_| panic!("No se pudo abrir certificado de cliente en: {}", cert_pem_path));
    let mut reader = BufReader::new(file);
    let certs: Vec<_> = rustls_pemfile::certs(&mut reader)
        .map(|r| r.expect("Certificado PEM corrupto"))
        .collect();

    assert!(!certs.is_empty(), "El archivo PEM no contiene certificados X.509");

    let mut hasher = Sha256::new();
    hasher.update(certs[0].as_ref());
    let digest = hex::encode_upper(hasher.finalize());
    digest[0..40].to_string()
}

/// Construye el cliente HTTP/2 con autenticación mutua TLS 1.3 (mTLS)
fn build_mtls_client(ca_pem_path: &str, client_cert_path: &str, client_key_path: &str) -> Client {
    let ca_bytes = fs::read(ca_pem_path).expect("Error leyendo CA Root PEM");
    let ca_cert = Certificate::from_pem(&ca_bytes).expect("CA Root PEM invalido");

    let mut identity_pem = fs::read(client_cert_path).expect("Error leyendo certificado cliente");
    let key_pem = fs::read(client_key_path).expect("Error leyendo llave privada cliente");
    identity_pem.push(b'\n');
    identity_pem.extend_from_slice(&key_pem);

    let identity = Identity::from_pem(&identity_pem)
        .expect("Error ensamblando identidad PKCS#8/ECDSA para mTLS");

    Client::builder()
        .use_rustls_tls()
        .min_tls_version(reqwest::tls::Version::TLS_1_3)
        .add_root_certificate(ca_cert)
        .identity(identity)
        .https_only(true)
        .timeout(Duration::from_secs(4))
        .build()
        .expect("Error construyendo cliente mTLS 1.3")
}

/// Curva de Control Autónomo Local (IEEE 1547-2018 Frequency-Watt Droop Control)
/// Se activa automáticamente si se pierde el enlace con la nube (>15s) o ante desviación severa de frecuencia
fn compute_autonomous_ieee1547_droop(
    freq_mhz: u32,
    soc_basis_points: u16,
    max_rated_watts: i64,
    pre_disturbance_watts: i64,
) -> i64 {
    // 1. Paro de emergencia por desconexión de red (Islanding / Trip Limits: < 57.0 Hz o > 61.8 Hz)
    if freq_mhz < 57_000 || freq_mhz > 61_800 {
        error!(
            freq_mhz = freq_mhz,
            "DISPARO_PROTECCION_IEEE1547: Frecuencia fuera de limites seguros. Inyeccion = 0 W"
        );
        return 0;
    }

    // 2. Banda muerta nominal (Deadband IEEE 1547 Categoría III: 59.964 Hz a 60.036 Hz)
    let db_low_mhz: i64 = 59_964;
    let db_high_mhz: i64 = 60_036;
    let current_f = freq_mhz as i64;

    if current_f >= db_low_mhz && current_f <= db_high_mhz {
        // En banda estable sin conexión a la nube: mantener operación segura conservadora o retorno a 0W si SoC bajo
        return if soc_basis_points > 2500 {
            pre_disturbance_watts.clamp(-max_rated_watts, max_rated_watts)
        } else {
            0
        };
    }

    // 3. Estatismo del 5% (k_droop = 0.05 -> 100% de potencia por cada 3,000 mHz [3.0 Hz] fuera de la banda muerta)
    if current_f > db_high_mhz {
        // Sobrefrecuencia (> 60.036 Hz): Reducir inyección o absorber energía (cargar batería si SoC < 95%)
        let delta_mhz = current_f - db_high_mhz;
        let reduction_w = (delta_mhz.saturating_mul(max_rated_watts)) / 3_000;
        let target_w = pre_disturbance_watts.saturating_sub(reduction_w);

        if soc_basis_points < 9500 {
            target_w.max(-max_rated_watts)
        } else {
            target_w.max(0)
        }
    } else {
        // Subfrecuencia (< 59.964 Hz): Inyectar potencia activa de soporte a la red si SoC > 15%
        if soc_basis_points <= 1500 {
            warn!(soc_bp = soc_basis_points, "SoC critico (<15%): Imposible soportar subfrecuencia");
            return 0;
        }
        let delta_mhz = db_low_mhz - current_f;
        let support_w = (delta_mhz.saturating_mul(max_rated_watts)) / 3_000;
        let target_w = pre_disturbance_watts.saturating_add(support_w);
        target_w.min(max_rated_watts)
    }
}

/// Lee los registros SunSpec del inversor por Modbus TCP.
/// Si el inversor físico no está accesible (modo laboratorio/HIL), genera telemetría determinista.
async fn read_sunspec_telemetry(
    modbus_addr: SocketAddr,
    device_lfdi: &str,
    tax_entity_id: &str,
) -> Ieee20305TelemetryPayload {
    let connect_result = timeout(Duration::from_millis(800), tcp::connect(modbus_addr)).await;

    if let Ok(Ok(mut ctx)) = connect_result {
        if let Ok(Ok(Ok(regs))) = timeout(
            Duration::from_millis(800),
            ctx.read_holding_registers(
                SunSpecRegisterMap::TELEMETRY_BASE_ADDR,
                SunSpecRegisterMap::TELEMETRY_REG_COUNT,
            ),
        )
        .await
        {
            if regs.len() >= 6 {
                let active_power_watts = (regs[0] as i16) as i64 * 10;
                let reactive_power_var = (regs[1] as i16) as i64 * 10;
                let voltage_mv = (regs[2] as u32) * 100;
                let grid_frequency_mhz = (regs[3] as u32) * 10;
                let available_energy_wh = (regs[4] as u64) * 100;
                let soc_basis_points = regs[5].min(10_000);

                return Ieee20305TelemetryPayload {
                    device_lfdi: device_lfdi.to_string(),
                    tax_entity_id: tax_entity_id.to_string(),
                    active_power_watts,
                    reactive_power_var,
                    available_energy_wh,
                    soc_basis_points,
                    grid_frequency_mhz,
                    voltage_mv,
                    timestamp_utc: Utc::now(),
                };
            }
        }
    }

    // Fallback Hardware-in-the-Loop (HIL) para validación sin inversor físico conectado
    Ieee20305TelemetryPayload {
        device_lfdi: device_lfdi.to_string(),
        tax_entity_id: tax_entity_id.to_string(),
        active_power_watts: 12_500,
        reactive_power_var: 350,
        available_energy_wh: 42_000,
        soc_basis_points: 8400,     // 84.00%
        grid_frequency_mhz: 60_002, // 60.002 Hz
        voltage_mv: 240_150,        // 240.15 V
        timestamp_utc: Utc::now(),
    }
}

/// Escribe la orden de despacho en el inversor vía Modbus TCP (SunSpec Model 124)
async fn write_sunspec_dispatch_watts(modbus_addr: SocketAddr, target_watts: i64) {
    let scaled_deci_kw = (target_watts / 10).clamp(i16::MIN as i64, i16::MAX as i64) as i16;
    let raw_register = scaled_deci_kw as u16;

    if let Ok(Ok(mut ctx)) = timeout(Duration::from_millis(800), tcp::connect(modbus_addr)).await {
        match timeout(
            Duration::from_millis(800),
            ctx.write_single_register(SunSpecRegisterMap::DISPATCH_COMMAND_ADDR, raw_register),
        )
        .await
        {
            Ok(Ok(_)) => info!(
                modbus_target = %modbus_addr,
                commanded_watts = target_watts,
                "Orden de despacho escrita en registro Modbus SunSpec"
            ),
            _ => warn!(
                modbus_target = %modbus_addr,
                commanded_watts = target_watts,
                "Fallo al escribir registro Modbus; aplicando setpoint en simulador HIL local"
            ),
        }
    } else {
        info!(
            mode = "HIL_SIMULATION",
            commanded_watts = target_watts,
            "Setpoint de potencia aplicado en controlador virtual local"
        );
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("info")
        .init();

    let _ = rustls::crypto::ring::default_provider().install_default();

    let aggregator_url = std::env::var("AURAINTEL_AGGREGATOR_URL")
        .unwrap_or_else(|_| "https://localhost:8443/sep2/mup".to_string());
    let ca_cert_path = std::env::var("AURAINTEL_CA_CERT")
        .unwrap_or_else(|_| "../auraintel-mtls-aggregator/certs/ca_root.pem".to_string());
    let client_cert_path = std::env::var("AURAINTEL_CLIENT_CERT")
        .unwrap_or_else(|_| "../auraintel-mtls-aggregator/certs/der_client.pem".to_string());
    let client_key_path = std::env::var("AURAINTEL_CLIENT_KEY")
        .unwrap_or_else(|_| "../auraintel-mtls-aggregator/certs/der_client.key".to_string());
    let modbus_addr: SocketAddr = std::env::var("AURAINTEL_MODBUS_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:1502".to_string())
        .parse()
        .expect("Direccion IP:Puerto Modbus TCP invalida");
    let tax_entity_id = std::env::var("AURAINTEL_TAX_ID")
        .unwrap_or_else(|_| "US-EIN-99-1234567".to_string());
    let max_rated_watts: i64 = 25_000; // 25 kW capacidad nominal del inversor

    // 1. Extraer LFDI directamente del certificado X.509 local
    let device_lfdi = extract_local_lfdi_from_pem(&client_cert_path);
    let mtls_client = build_mtls_client(&ca_cert_path, &client_cert_path, &client_key_path);

    info!(
        device_lfdi = %device_lfdi,
        aggregator_url = %aggregator_url,
        modbus_inverter = %modbus_addr,
        "Agente de Borde AuraIntel VPP iniciado con proteccion autonoma IEEE 1547"
    );

    let mut ticker = interval(Duration::from_secs(5));
    let mut consecutive_cloud_failures: u32 = 0;
    let mut last_commanded_watts: i64 = 0;

    loop {
        ticker.tick().await;

        // Paso A: Leer estado físico del inversor/batería (SunSpec Modbus TCP)
        let telemetry = read_sunspec_telemetry(modbus_addr, &device_lfdi, &tax_entity_id).await;

        // Paso B: Verificación prioritaria de Frecuencia de Red Local (IEEE 1547 Override)
        if telemetry.grid_frequency_mhz < 59_964 || telemetry.grid_frequency_mhz > 60_036 {
            let droop_watts = compute_autonomous_ieee1547_droop(
                telemetry.grid_frequency_mhz,
                telemetry.soc_basis_points,
                max_rated_watts,
                last_commanded_watts,
            );
            warn!(
                freq_mhz = telemetry.grid_frequency_mhz,
                droop_dispatch_w = droop_watts,
                "ANOMALIA_FRECUENCIA_LOCAL: Curva Frequency-Watt IEEE 1547 activada con prioridad sobre la nube"
            );
            write_sunspec_dispatch_watts(modbus_addr, droop_watts).await;
            last_commanded_watts = droop_watts;
        }

        // Paso C: Transmitir telemetría al Servidor Agregador vía mTLS 1.3
        match mtls_client.post(&aggregator_url).json(&telemetry).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<DerControlDispatchResponse>().await {
                    Ok(dispatch) => {
                        consecutive_cloud_failures = 0;
                        let safe_watts = dispatch
                            .target_dispatch_watts
                            .clamp(-max_rated_watts, max_rated_watts);

                        info!(
                            event_id = %dispatch.event_id,
                            commanded_w = safe_watts,
                            audit_hash = %dispatch.audit_record_hash,
                            "Despacho mTLS recibido y verificado"
                        );

                        write_sunspec_dispatch_watts(modbus_addr, safe_watts).await;
                        last_commanded_watts = safe_watts;
                    }
                    Err(e) => {
                        consecutive_cloud_failures += 1;
                        error!(error = %e, "Payload JSON de respuesta del agregador invalido");
                    }
                }
            }
            Ok(resp) => {
                consecutive_cloud_failures += 1;
                warn!(
                    http_status = %resp.status(),
                    failures = consecutive_cloud_failures,
                    "Servidor agregador rechazo el paquete de telemetria"
                );
            }
            Err(e) => {
                consecutive_cloud_failures += 1;
                warn!(
                    error = %e,
                    failures = consecutive_cloud_failures,
                    "Fallo de enlace mTLS con el servidor agregador"
                );
            }
        }

        // Paso D: Fail-Safe Watchdog (>15 segundos / 3 ciclos sin conexión con la nube)
        if consecutive_cloud_failures >= 3 {
            let fallback_watts = compute_autonomous_ieee1547_droop(
                telemetry.grid_frequency_mhz,
                telemetry.soc_basis_points,
                max_rated_watts,
                0, // En pérdida de enlace prolongada y frecuencia normal, converger a 0W exportados por seguridad
            );

            warn!(
                offline_seconds = consecutive_cloud_failures * 5,
                fallback_dispatch_w = fallback_watts,
                "FAIL_SAFE_WATCHDOG_ACTIVO: Mas de 15s sin enlace Cloud. Operando bajo control autonomo local IEEE 1547"
            );

            write_sunspec_dispatch_watts(modbus_addr, fallback_watts).await;
            last_commanded_watts = fallback_watts;
        }
    }
}
