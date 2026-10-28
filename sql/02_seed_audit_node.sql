INSERT INTO der_registered_nodes (
    node_id, owner_legal_name, tax_entity_id, tax_regime_code,
    jurisdiction, iso_substation_node, ieee1547_cert_hash,
    ul1741sb_cert_hash, mtls_client_cert_fingerprint, is_active
) VALUES (
    'a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11',
    'SUNSHINE SOLAR STORAGE HOLDINGS LLC',
    '99-8765432',
    'W9-LLC',
    'UsFercIso',
    'FL-ORLANDO-SUB-01',
    'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    '8f434346648f6b96df89dda901c5176b10a6d83961dd3c1ac88b59b2dc327aa4',
    '2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae',
    TRUE
) ON CONFLICT (mtls_client_cert_fingerprint) DO NOTHING;
