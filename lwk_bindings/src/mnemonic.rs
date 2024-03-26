use std::{fmt::Display, str::FromStr, sync::Arc};

use lwk_signer::bip39;

use crate::LwkError;

/// Wrapper over [`bip39::Mnemonic`]
#[derive(uniffi::Object, PartialEq, Eq, Debug)]
#[uniffi::export(Display)]
pub struct Mnemonic {
    inner: bip39::Mnemonic,
}

impl From<bip39::Mnemonic> for Mnemonic {
    fn from(inner: bip39::Mnemonic) -> Self {
        Self { inner }
    }
}

impl Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[uniffi::export]
impl Mnemonic {
    /// Construct a Mnemonic type
    #[uniffi::constructor]
    pub fn new(s: &str) -> Result<Arc<Self>, LwkError> {
        let inner = bip39::Mnemonic::from_str(s)?;
        Ok(Arc::new(Self { inner }))
    }
}

#[cfg(test)]
mod tests {
    use crate::Mnemonic;
    use lwk_signer::bip39;
    use std::str::FromStr;

    #[test]
    fn mnemonic() {
        let mnemonic_str = lwk_test_util::TEST_MNEMONIC;
        let mnemonic_bip39 = bip39::Mnemonic::from_str(mnemonic_str).unwrap();
        let from_bip39: Mnemonic = mnemonic_bip39.into();
        let mnemonic = Mnemonic::new(mnemonic_str).unwrap();
        assert_eq!(mnemonic_str, mnemonic.to_string());
        assert_eq!(from_bip39, *mnemonic);
    }
}
