use crate::{AssetId, Error, Transaction};
use lwk_wollet::elements::pset::PartiallySignedTransaction;
use std::fmt::Display;
use wasm_bindgen::prelude::*;

/// Partially Signed Elements Transaction, wrapper of [`PartiallySignedTransaction`]
#[wasm_bindgen]
#[derive(PartialEq, Debug, Clone)]
pub struct Pset {
    inner: PartiallySignedTransaction,
}

impl From<PartiallySignedTransaction> for Pset {
    fn from(inner: PartiallySignedTransaction) -> Self {
        Self { inner }
    }
}

impl From<Pset> for PartiallySignedTransaction {
    fn from(pset: Pset) -> Self {
        pset.inner
    }
}

impl Display for Pset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[wasm_bindgen]
impl Pset {
    /// Creates a `Pset`
    #[wasm_bindgen(constructor)]
    pub fn new(base64: &str) -> Result<Pset, Error> {
        let pset: PartiallySignedTransaction = base64.trim().parse()?;
        Ok(pset.into())
    }

    #[wasm_bindgen(js_name = extractTx)]
    pub fn extract_tx(&self) -> Result<Transaction, Error> {
        let tx: Transaction = self.inner.extract_tx()?.into();
        Ok(tx)
    }

    #[wasm_bindgen(js_name = issuanceAsset)]
    pub fn issuance_asset(&self, index: u32) -> Option<AssetId> {
        self.issuances_ids(index).map(|e| e.0)
    }

    #[wasm_bindgen(js_name = issuanceToken)]
    pub fn issuance_token(&self, index: u32) -> Option<AssetId> {
        self.issuances_ids(index).map(|e| e.1)
    }
}

impl Pset {
    fn issuances_ids(&self, index: u32) -> Option<(AssetId, AssetId)> {
        let issuance_ids = self.inner.inputs().get(index as usize)?.issuance_ids();
        Some((issuance_ids.0.into(), issuance_ids.1.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::Pset;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn pset_roundtrip() {
        let pset_string =
            include_str!("../../lwk_jade/test_data/pset_to_be_signed.base64").to_string();
        let pset = Pset::new(&pset_string).unwrap();

        let tx_expected =
            include_str!("../../lwk_jade/test_data/pset_to_be_signed_transaction.hex").to_string();
        let tx_string = pset.extract_tx().unwrap().to_string();
        assert_eq!(tx_expected, tx_string);

        assert_eq!(pset_string, pset.to_string());
    }
}
