CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS der_registered_nodes (
    node_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_legal_name VARCHAR(255) NOT NULL,
    tax_entity_id VARCHAR(64) NOT NULL,
    tax_regime_code VARCHAR(16) NOT NULL,
    jurisdiction VARCHAR(32) NOT NULL CHECK (jurisdiction IN ('UsFercIso', 'MxCenaceMem', 'EuEnstoE')),
    iso_substation_node VARCHAR(64) NOT NULL,
    ieee1547_cert_hash CHAR(64) NOT NULL,
    ul1741sb_cert_hash CHAR(64) NOT NULL,
    mtls_client_cert_fingerprint CHAR(64) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS vpp_dispatch_settlement_ledger (
    record_id UUID PRIMARY KEY,
    node_id UUID NOT NULL REFERENCES der_registered_nodes(node_id),
    tax_entity_id VARCHAR(64) NOT NULL,
    jurisdiction VARCHAR(32) NOT NULL,
    dispatched_wh BIGINT NOT NULL CHECK (dispatched_wh > 0),
    gross_settlement_micro_usd BIGINT NOT NULL CHECK (gross_settlement_micro_usd >= 0),
    tax_withholding_micro_usd BIGINT NOT NULL CHECK (tax_withholding_micro_usd >= 0),
    net_payout_micro_usd BIGINT NOT NULL CHECK (net_payout_micro_usd >= 0),
    cfdi_uuid_or_1099_ref VARCHAR(64),
    timestamp_utc TIMESTAMPTZ NOT NULL,
    previous_record_hash CHAR(64) NOT NULL,
    record_hash CHAR(64) NOT NULL UNIQUE,
    hmac_signature CHAR(64) NOT NULL
);

CREATE OR REPLACE FUNCTION prevent_ledger_tampering()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'OPERACION_PROHIBIDA: Libro mayor inmutable (NIST AU-9 / Materialidad Fiscal).';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_immutable_vpp_ledger ON vpp_dispatch_settlement_ledger;
CREATE TRIGGER trg_immutable_vpp_ledger
BEFORE UPDATE OR DELETE OR TRUNCATE ON vpp_dispatch_settlement_ledger
FOR EACH STATEMENT
EXECUTE FUNCTION prevent_ledger_tampering();
