const COST: u32 = 12;

pub fn hash(pw: &str) -> anyhow::Result<String> {
    Ok(bcrypt::hash(pw, COST)?)
}

pub fn verify(pw: &str, hash: &str) -> bool {
    bcrypt::verify(pw, hash).unwrap_or(false)
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
    fn own_hash_roundtrips() {
        let h = hash("abcd1234").unwrap();
        assert!(h.starts_with("$2"));
        assert!(verify("abcd1234", &h));
    }
}
