const COST: u32 = 12;

/// bcrypt 는 앞 72바이트만 사용한다. 원본(bcryptjs 3.x)도 72바이트에서 잘라 비교하므로 동일하게 맞춘다.
const MAX_BYTES: usize = 72;

fn truncated(pw: &str) -> &[u8] {
    let bytes = pw.as_bytes();
    &bytes[..bytes.len().min(MAX_BYTES)]
}

pub fn hash(pw: &str) -> anyhow::Result<String> {
    Ok(bcrypt::hash(truncated(pw), COST)?)
}

pub fn verify(pw: &str, hash: &str) -> bool {
    bcrypt::verify(truncated(pw), hash).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    // node: require('bcryptjs').hash('fixture-pw-1234', 12)
    const BCRYPTJS_HASH: &str = "$2b$12$wmrm/h0KccKm2xIVIkxrueqW1bGv6PD66EX4IVTIBlMViAQyfwUxa";

    #[test]
    fn verifies_bcryptjs_hash() {
        assert!(verify("fixture-pw-1234", BCRYPTJS_HASH));
        assert!(!verify("wrong", BCRYPTJS_HASH));
    }

    #[test]
    fn password_over_72_bytes_is_truncated_like_bcryptjs() {
        let pw = "a".repeat(100);
        let h = hash(&pw).unwrap();
        assert!(verify(&pw, &h));
        assert!(verify(&pw[..72], &h));
        assert!(!verify(&pw[..71], &h));
    }

    #[test]
    fn own_hash_roundtrips() {
        let h = hash("abcd1234").unwrap();
        assert!(h.starts_with("$2"));
        assert!(verify("abcd1234", &h));
    }
}
