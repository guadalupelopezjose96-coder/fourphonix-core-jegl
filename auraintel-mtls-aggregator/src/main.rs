use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use hyper::body::Incoming;
use hyper::Request;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder as ServerBuilder;
use rustls::{
    crypto::ring::default_provider,
    pki_types::{CertificateDer, PrivateKeyDer},
    server::WebPkiClientVerifier,
    RootCertStore, ServerConfig,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::BufReader,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tower_service::Service;
use tracing::{error, info, warn};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Identidad criptográfica extraída directamente de la sesión TLS 1.3 del nodo DER
#[derive(Clone, Debug)]
pub struct VerifiedClientCertIdentity {
    pub cert_sha256_fingerprint: String,
    /// IEEE 2030.5 Long-Form Device Identifier (Primeros 160 bits / 40 caracteres hex del SHA-256)
    pub ieee2030_5_lfdi: String,
    /// IEEE 2030.5 Short-Form Device Identifier (Primeros 36 bits convertidos a decimal con dígito verificador)
    pub ieee2030_5_sfdi: u64,
}

/// Payload de Telemetría IEEE 2030.5 (MirrorUsagePoint / DERStatus) y OpenADR (oadrUpdateReport)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ieee20305TelemetryPayload {
    pub device_lfdi: String,
    pub tax_entity_id: String,
    pub active_power_watts: i64, // Positivo = Inyección a red, Negativo = Carga
    pub reactive_power_var: i64,
    pub available_energy_wh: u64,
    pub soc_basis_points: u16,   // 0 a 10000 (100.00%)
    pub grid_frequency_mhz: u32, // Ej. 60000 = 60.000 Hz
    pub voltage_mv: u32,         // Ej. 240000 = 240.000 V
    pub timestamp_utc: DateTime<Utc>,
}

/// Respuesta de Despacho / Control enviada de vuelta al Inversor/Batería (DERControl / oadrDistributeEvent)
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

pub struct AppState {
    pub hmac_key: Vec<u8>,
    pub last_ledger_hash: Mutex<String>,
}

/// Calcula el LFDI (40 hex) y SFDI (36 bits + Luhn mod 10) según el estándar IEEE 2030.5-2018 Sección 6.3.3
fn derive_ieee2030_5_identities(cert_der: &CertificateDer<'_>) -> VerifiedClientCertIdentity {
    let mut hasher = Sha256::new();
    hasher.update(cert_der.as_ref());
    let digest = hasher.finalize();
    let full_hex = hex::encode_upper(digest);

    // LFDI: Primeros 20 bytes (40 caracteres hexadecimales)
    let lfdi = full_hex[0..40].to_string();

    // SFDI: Primeros 36 bits (9 caracteres hexadecimales)
    let top_36_bits: u64 = ((digest[0] as u64) << 28)
        | ((digest[1] as u64) << 20)
        | ((digest[2] as u64) << 12)
        | ((digest[3] as u64) << 4)
        | ((digest[4] as u64) >> 4);

    // Cálculo del dígito de suma de verificación (Checksum mod 10) para SFDI
    let mut temp = top_36_bits;
    let mut sum = 0u64;
    while temp > 0 {
        sum += temp % 10;
        temp /= 10;
    }
    let check_digit = (10 - (sum % 10)) % 10;
    let sfdi = (top_36_bits * 10) + check_digit;

    VerifiedClientCertIdentity {
        cert_sha256_fingerprint: full_hex,
        ieee2030_5_lfdi: lfdi,
        ieee2030_5_sfdi: sfdi,
    }
}

/// Endpoint de estado de salud mTLS (DeviceCapability - /dcap)
async fn get_device_capability(
    Extension(client_id): Extension<VerifiedClientCertIdentity>,
) -> impl IntoResponse {
    info!(
        lfdi = %client_id.ieee2030_5_lfdi,
        sfdi = client_id.ieee2030_5_sfdi,
        "Consulta IEEE 2030.5 /dcap autenticada por mTLS"
    );

    Json(serde_json::json!({
        "status": "ONLINE_ZERO_TRUST_MTLS_1_3",
        "standard": "IEEE_2030.5_2018_AND_OPENADR_3.0",
        "authenticated_lfdi": client_id.ieee2030_5_lfdi,
        "authenticated_sfdi": client_id.ieee2030_5_sfdi,
        "endpoints": {
            "telemetry_post": "/sep2/mup",
            "openadr_report": "/openadr/v3/reports"
        }
    }))
}

/// Endpoint principal de ingesta de telemetría y despacho en bucle cerrado (/sep2/mup)
async fn receive_telemetry_and_dispatch(
    State(state): State<Arc<AppState>>,
    Extension(client_id): Extension<VerifiedClientCertIdentity>,
    Json(payload): Json<Ieee20305TelemetryPayload>,
) -> Result<Json<DerControlDispatchResponse>, (StatusCode, Json<serde_json::Value>)> {
    // 1. Prevención estricta de Spoofing: El LFDI del payload DEBE coincidir con el certificado TLS 1.3
    if payload.device_lfdi.to_uppercase() != client_id.ieee2030_5_lfdi {
        warn!(
            cert_lfdi = %client_id.ieee2030_5_lfdi,
            payload_lfdi = %payload.device_lfdi,
            "ALERTA_SEGURIDAD_ZERO_TRUST: Intento de suplantacion de LFDI detectado y bloqueado"
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "LFDI_MISMATCH",
                "message": "El LFDI del payload no coincide con el certificado X.509 de la sesion mTLS"
            })),
        ));
    }

    // 2. Validación de Frecuencia de Red (Protección IEEE 1547: Banda operativa 59.5 Hz - 60.5 Hz)
    let mut target_dispatch_watts: i64 = 0;
    if payload.grid_frequency_mhz < 59_500 || payload.grid_frequency_mhz > 60_500 {
        warn!(
            lfdi = %client_id.ieee2030_5_lfdi,
            freq_mhz = payload.grid_frequency_mhz,
            "EMERGENCIA_RED: Frecuencia fuera de banda nominal. Activando control autonomo local (Fail-Safe)"
        );
    } else if payload.soc_basis_points > 2000 {
        // Si SoC > 20%, ordenar inyección óptima del 80% de la energía disponible (hasta 25 kW por intervalo)
        let exportable = (payload.available_energy_wh.saturating_mul(80) / 100).min(25_000);
        target_dispatch_watts = exportable as i64;
    }

    // 3. Encadenamiento Criptográfico Inmutable para Auditoría Fiscal (SAT/IRS) y Regulatoria (FERC/NERC)
    let event_id = Uuid::new_v4();
    let now_utc = Utc::now();

    let (record_hash, hmac_sig) = {
        let mut prev_hash_guard = state
            .last_ledger_hash
            .lock()
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "MUTEX_POISONED"})),
                )
            })?;

        let canonical_record = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            event_id,
            client_id.ieee2030_5_lfdi,
            payload.tax_entity_id,
            payload.active_power_watts,
            target_dispatch_watts,
            payload.grid_frequency_mhz,
            now_utc.to_rfc3339(),
            *prev_hash_guard
        );

        let mut hasher = Sha256::new();
        hasher.update(canonical_record.as_bytes());
        let current_hash = hex::encode(hasher.finalize());

        let mut mac = HmacSha256::new_from_slice(&state.hmac_key).expect("Clave HMAC valida");
        mac.update(current_hash.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        *prev_hash_guard = current_hash.clone();
        (current_hash, signature)
    };

    info!(
        event_id = %event_id,
        lfdi = %client_id.ieee2030_5_lfdi,
        sfdi = client_id.ieee2030_5_sfdi,
        tax_id = %payload.tax_entity_id,
        reported_w = payload.active_power_watts,
        commanded_dispatch_w = target_dispatch_watts,
        audit_hash = %record_hash,
        "Telemetria procesada y orden de despacho firmada"
    );

    Ok(Json(DerControlDispatchResponse {
        event_id,
        verified_lfdi: client_id.ieee2030_5_lfdi,
        target_dispatch_watts,
        autonomous_droop_fallback_enabled: true,
        poll_rate_seconds: 5,
        audit_record_hash: record_hash,
        audit_hmac_signature: hmac_sig,
        server_timestamp_utc: now_utc,
    }))
}

/// Carga certificados X.509 en formato PEM
fn load_certs(path: &str) -> Vec<CertificateDer<'static>> {
    let file = File::open(path).unwrap_or_else(|_| panic!("No se pudo abrir certificado: {}", path));
    let mut reader = BufReader::new(file);
    rustls_pemfile::certs(&mut reader)
        .map(|res| res.expect("Certificado PEM invalido"))
        .collect()
}

/// Carga llave privada ECDSA/RSA en formato PEM
fn load_private_key(path: &str) -> PrivateKeyDer<'static> {
    let file = File::open(path).unwrap_or_else(|_| panic!("No se pudo abrir llave privada: {}", path));
    let mut reader = BufReader::new(file);
    rustls_pemfile::private_key(&mut reader)
        .expect("Error leyendo llave privada")
        .expect("No se encontro llave privada valida en el archivo PEM")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("info")
        .init();

    // 1. Instalar proveedor criptográfico Ring (FIPS/NIST compatible)
    let _ = default_provider().install_default();

    let ca_cert_path = std::env::var("AURAINTEL_CA_CERT").unwrap_or_else(|_| "./certs/ca_root.pem".to_string());
    let server_cert_path = std::env::var("AURAINTEL_SERVER_CERT").unwrap_or_else(|_| "./certs/server.pem".to_string());
    let server_key_path = std::env::var("AURAINTEL_SERVER_KEY").unwrap_or_else(|_| "./certs/server.key".to_string());
    let bind_addr: SocketAddr = std::env::var("AURAINTEL_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8443".to_string())
        .parse()
        .expect("Direccion IP:Puerto invalida");

    // 2. Configurar RootCertStore para exigir mTLS a todos los nodos DER entrantes
    let mut client_auth_roots = RootCertStore::empty();
    for cert in load_certs(&ca_cert_path) {
        client_auth_roots.add(cert).expect("Error agregando certificado CA al RootStore");
    }

    let client_verifier = WebPkiClientVerifier::builder(Arc::new(client_auth_roots))
        .build()
        .expect("Fallo al construir verificador mTLS WebPki");

    // 3. Configurar TLS 1.3 exclusivo (NIST SP 800-52 Rev. 2 / Zero-Trust)
    let server_certs = load_certs(&server_cert_path);
    let server_key = load_private_key(&server_key_path);

    let mut tls_config = ServerConfig::builder_with_protocol_versions(&[&rustls::version::TLS13])
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(server_certs, server_key)
        .expect("Configuracion de certificado/llave del servidor invalida");

    tls_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

    // 4. Inicializar estado compartido e inmutabilidad HMAC
    let hmac_secret = std::env::var("AURAINTEL_HMAC_KEY")
        .unwrap_or_else(|_| "CLAVE_PRODUCCION_ZERO_TRUST_AURAINTEL_VPP_2026".to_string());

    let app_state = Arc::new(AppState {
        hmac_key: hmac_secret.into_bytes(),
        last_ledger_hash: Mutex::new("0".repeat(64)),
    });

    let app = Router::new()
        .route("/dcap", get(get_device_capability))
        .route("/sep2/mup", post(receive_telemetry_and_dispatch))
        .route("/openadr/v3/reports", post(receive_telemetry_and_dispatch))
        .with_state(app_state);

    let listener = TcpListener::bind(&bind_addr)
        .await
        .expect("No se pudo enlazar el puerto TCP");

    info!(
        listen_addr = %bind_addr,
        tls_version = "TLSv1.3_ONLY",
        client_auth = "MANDATORY_MTLS_X509",
        "Servidor Agregador AuraIntel VPP listo para conexiones IEEE 2030.5 y OpenADR"
    );

    // 5. Bucle de aceptación de conexiones seguras
    loop {
        let (tcp_stream, remote_addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(error = %e, "Error aceptando conexion TCP");
                continue;
            }
        };

        let tls_acceptor = tls_acceptor.clone();
        let app = app.clone();

        tokio::spawn(async move {
            let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                Ok(stream) => stream,
                Err(e) => {
                    warn!(remote_ip = %remote_addr, error = %e, "Handshake mTLS 1.3 rechazado");
                    return;
                }
            };

            // Extraer el certificado X.509 hoja presentado por el nodo cliente
            let (_, server_conn) = tls_stream.get_ref();
            let peer_certs = match server_conn.peer_certificates() {
                Some(certs) if !certs.is_empty() => certs,
                _ => {
                    warn!(remote_ip = %remote_addr, "Conexion sin certificado de cliente rechazada");
                    return;
                }
            };

            let verified_identity = derive_ieee2030_5_identities(&peer_certs[0]);
            let io = TokioIo::new(tls_stream);

            let hyper_service = hyper::service::service_fn(move |mut req: Request<Incoming>| {
                let mut app = app.clone();
                let identity = verified_identity.clone();
                async move {
                    req.extensions_mut().insert(identity);
                    app.call(req).await
                }
            });

            if let Err(err) = ServerBuilder::new(TokioExecutor::new())
                .serve_connection(io, hyper_service)
                .await
            {
                error!(remote_ip = %remote_addr, error = ?err, "Error en sesion HTTP/2 sobre mTLS");
            }
        });
    }
}
