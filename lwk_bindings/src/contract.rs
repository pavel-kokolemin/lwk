use std::sync::Arc;

use elements::hashes::hex::FromHex;

use crate::LwkError;

/// Wrapper over [`lwk_wollet::Contract`]
#[derive(uniffi::Object)]
#[uniffi::export(Display)]
pub struct Contract {
    inner: lwk_wollet::Contract,
}

impl From<lwk_wollet::Contract> for Contract {
    fn from(inner: lwk_wollet::Contract) -> Self {
        Self { inner }
    }
}

impl From<Contract> for lwk_wollet::Contract {
    fn from(contract: Contract) -> Self {
        contract.inner
    }
}

impl From<&Contract> for lwk_wollet::Contract {
    fn from(contract: &Contract) -> Self {
        contract.inner.clone()
    }
}

impl std::fmt::Display for Contract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(&self.inner).expect("contain simple types");
        write!(f, "{}", &json)
    }
}

#[uniffi::export]
impl Contract {
    /// Construct a Contract object
    #[uniffi::constructor]
    pub fn new(
        domain: String,
        issuer_pubkey: &str,
        name: String,
        precision: u8,
        ticker: String,
        version: u8,
    ) -> Result<Arc<Self>, LwkError> {
        let inner = lwk_wollet::Contract {
            entity: lwk_wollet::Entity::Domain(domain),
            issuer_pubkey: Vec::<u8>::from_hex(issuer_pubkey)
                .map_err(|e| format!("invalid issuer pubkey: {e}"))?,
            name,
            precision,
            ticker,
            version,
        };
        inner.validate()?; // TODO validate should be the constructor
        Ok(Arc::new(Self { inner }))
    }
}
