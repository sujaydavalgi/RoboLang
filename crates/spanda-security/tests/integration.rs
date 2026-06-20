//! Integration tests for spanda-security.

use spanda_security::{
    CapabilitySet, PackagePermissions, RobotIdentity, SecretHandle, SecretSource, SecretStore,
    SecurePolicy, SecurityContext, SignedMessage, TrustLevel,
};

#[test]
fn full_security_context_flow() {
    let mut ctx = SecurityContext::with_permissions(&PackagePermissions::from_capabilities([
        "audit.write",
        "identity.sign",
        "identity.verify",
    ]));
    ctx.set_identity(RobotIdentity::new("rover-1", "device-key").with_trust(TrustLevel::Trusted));
    ctx.register_secure_endpoint("/cmd", SecurePolicy::signed_trusted());

    let signed = ctx.sign_outbound("/cmd", "move forward").unwrap().unwrap();
    assert!(signed.verify(ctx.identity.as_ref().unwrap()).unwrap());

    ctx.verify_inbound("/cmd", Some(&signed)).unwrap();
}

#[test]
fn secret_store_env_and_literal() {
    let mut store = SecretStore::new();
    store.register(SecretHandle {
        name: "token".into(),
        source: SecretSource::Literal {
            value: "abc123".into(),
        },
    });
    assert_eq!(store.resolve("token").unwrap(), "abc123");
}

#[test]
fn capability_denial_blocks_audit() {
    let ctx = SecurityContext::new();
    let mut audit = spanda_audit::AuditRuntime::new("test", vec![]);
    assert!(ctx.audit_event(&mut audit, "test", "detail").is_err());
}

#[test]
fn trust_level_blocks_restricted_endpoint() {
    let mut ctx = SecurityContext::new();
    ctx.set_identity(
        RobotIdentity::new("rover-1", "key").with_trust(TrustLevel::Restricted),
    );
    let mut caps = CapabilitySet::new();
    caps.grant("identity.verify");
    ctx.capabilities = caps;
    ctx.register_secure_endpoint(
        "/secure",
        SecurePolicy {
            signed: false,
            min_trust: Some(TrustLevel::Trusted),
            requires: vec![],
        },
    );
    assert!(ctx.verify_inbound("/secure", None).is_err());
}

#[test]
fn signed_message_json_export() {
    let id = RobotIdentity::new("bot", "key");
    let msg = SignedMessage::sign("payload", &id);
    let json = msg.to_json().unwrap();
    assert!(json.contains("signature"));
}
