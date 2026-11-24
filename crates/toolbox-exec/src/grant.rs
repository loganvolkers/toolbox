//! Approval grants (docs/design/federated-actions.md §5.3, T5): how a served parent passes the
//! approvals it recorded to a child that runs the action. The parent signs one statement with its
//! Ed25519 key — which run, which child action and version, entity, inputs, effects, requester and
//! approvals, and until when — and the child counts the approvals only when the signature verifies
//! with the key it pinned in `federation.acceptApprovalsFrom` and every field matches the run it
//! is about to start. A flag or workflow input alone approves nothing.

use aws_lc_rs::signature::{ED25519, Ed25519KeyPair, KeyPair, UnparsedPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

/// How long a grant is valid after the parent signs it, just before dispatch.
pub const GRANT_TTL_SECS: u64 = 600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Grant {
    pub parent: String,
    pub parent_run: String,
    pub action: String,
    pub action_version: String,
    pub entity: String,
    pub inputs: Map<String, Value>,
    /// `effects_hash` of the child's preview the approvers' run was re-prepared against.
    pub effects: String,
    pub requested_by: String,
    /// `<approver entry>=<user>`, as the parent recorded them.
    pub approvals: Vec<String>,
    pub expires: u64,
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    s.len().is_multiple_of(2)
        .then(|| {
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
                .collect()
        })
        .flatten()
}

/// The digest a grant names a preview's effects by.
pub fn effects_hash(effects: &[Value]) -> String {
    let mut h = Sha256::new();
    h.update(Value::Array(effects.to_vec()).to_string().as_bytes());
    hex(&h.finalize())[..32].to_owned()
}

fn keypair(seed_hex: &str) -> Result<Ed25519KeyPair, String> {
    let seed = unhex(seed_hex.trim())
        .filter(|s| s.len() == 32)
        .ok_or("the signing key must be 64 hex characters (32 random bytes)")?;
    Ed25519KeyPair::from_seed_unchecked(&seed).map_err(|e| format!("signing key: {e}"))
}

/// `ed25519:<hex>`: the public key a child pins for this seed.
pub fn public_key(seed_hex: &str) -> Result<String, String> {
    Ok(format!(
        "ed25519:{}",
        hex(keypair(seed_hex)?.public_key().as_ref())
    ))
}

/// `<hex statement>.<hex signature>`.
pub fn sign(g: &Grant, seed_hex: &str) -> Result<String, String> {
    let kp = keypair(seed_hex)?;
    let body = serde_json::to_vec(g).map_err(|e| e.to_string())?;
    Ok(format!("{}.{}", hex(&body), hex(kp.sign(&body).as_ref())))
}

/// The grant, if `token` is signed by `key` (`ed25519:<hex>`) and has not expired.
pub fn open(token: &str, key: &str, now: u64) -> Result<Grant, String> {
    let (body, sig) = token
        .split_once('.')
        .and_then(|(b, s)| Some((unhex(b)?, unhex(s)?)))
        .ok_or("the approval grant is malformed")?;
    let pk = key
        .strip_prefix("ed25519:")
        .and_then(unhex)
        .ok_or("the pinned key must be ed25519:<64 hex>")?;
    UnparsedPublicKey::new(&ED25519, pk)
        .verify(&body, &sig)
        .map_err(|_| "the approval grant's signature does not verify with the pinned key")?;
    let g: Grant = serde_json::from_slice(&body).map_err(|e| format!("approval grant: {e}"))?;
    if now > g.expires {
        return Err("the approval grant expired; approve through the parent again".into());
    }
    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    fn grant() -> Grant {
        Grant {
            parent: "org".into(),
            parent_run: "run_1".into(),
            action: "set-owners".into(),
            action_version: "sha256:x".into(),
            entity: "repository:gamma".into(),
            inputs: Map::new(),
            effects: effects_hash(&[]),
            requested_by: "user:ann".into(),
            approvals: vec!["user:bob=user:bob".into()],
            expires: 100,
        }
    }

    #[test]
    fn a_grant_opens_only_with_its_key_and_before_it_expires() {
        let key = public_key(SEED).unwrap();
        let t = sign(&grant(), SEED).unwrap();
        assert_eq!(open(&t, &key, 50).unwrap(), grant());
        assert!(open(&t, &key, 101).unwrap_err().contains("expired"));
        let other = public_key(&"ab".repeat(32)).unwrap();
        assert!(
            open(&t, &other, 50)
                .unwrap_err()
                .contains("does not verify")
        );
        let (body, sig) = t.split_once('.').unwrap();
        let mut forged = grant();
        forged.approvals = vec!["user:bob=user:mallory".into()];
        let forged = format!("{}.{sig}", hex(&serde_json::to_vec(&forged).unwrap()));
        assert_ne!(forged.split_once('.').unwrap().0, body);
        assert!(open(&forged, &key, 50).is_err());
        assert!(public_key("short").is_err());
    }
}
