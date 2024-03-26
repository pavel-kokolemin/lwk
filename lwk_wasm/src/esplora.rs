use crate::{Error, Update, Wollet};
use wasm_bindgen::prelude::*;

/// Wrapper of [`lwk_wollet::EsploraWasmClient`]
#[wasm_bindgen]
pub struct EsploraClient {
    inner: lwk_wollet::EsploraWasmClient,
}

#[wasm_bindgen]
impl EsploraClient {
    /// Creates an `EsploraClient`
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Self {
        let inner = lwk_wollet::EsploraWasmClient::new(url);
        Self { inner }
    }

    #[wasm_bindgen(js_name = fullScan)]
    pub async fn full_scan(&mut self, wollet: &Wollet) -> Result<Option<Update>, Error> {
        let update: Option<lwk_wollet::Update> = self.inner.full_scan(wollet.as_ref()).await?;
        Ok(update.map(Into::into))
    }
}

#[cfg(test)]
mod tests {

    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_sleep() {
        lwk_wollet::async_sleep(1).await;
    }
}
