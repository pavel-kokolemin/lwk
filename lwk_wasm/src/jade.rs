use std::{collections::HashMap, str::FromStr};

use crate::{
    serial::{get_jade_serial, WebSerial},
    signer::FakeSigner,
    Error, Network, Pset, WolletDescriptor, Xpub,
};
use lwk_common::DescriptorBlindingKey;
use lwk_jade::{asyncr, protocol::GetXpubParams};
use lwk_jade::{
    derivation_path_to_vec,
    get_receive_address::{GetReceiveAddressParams, SingleOrMulti, Variant},
};
use lwk_wollet::{bitcoin::bip32::DerivationPath, elements::pset::PartiallySignedTransaction};
use wasm_bindgen::prelude::*;

/// Wrapper of [`asyncr::Jade`]
#[wasm_bindgen]
pub struct Jade {
    inner: asyncr::Jade<WebSerial>,
    _port: web_sys::SerialPort,
}

#[wasm_bindgen]
impl Jade {
    /// Creates a Jade from Web Serial for the given network
    ///
    /// When filter is true, it will filter available serial with Blockstream released chips, use
    /// false if you don't see your DYI jade
    #[wasm_bindgen(constructor)]
    pub async fn from_serial(network: &Network, filter: bool) -> Result<Jade, Error> {
        let port = get_jade_serial(filter).await?;
        let web_serial = WebSerial::new(&port)?;

        let inner = asyncr::Jade::new(web_serial, network.clone().into());
        Ok(Jade { inner, _port: port })
    }

    #[wasm_bindgen(js_name = getVersion)]
    pub async fn get_version(&self) -> Result<JsValue, Error> {
        let version = self.inner.version_info().await?;
        Ok(serde_wasm_bindgen::to_value(&version)?)
    }

    #[wasm_bindgen(js_name = getMasterXpub)]
    pub async fn get_master_xpub(&self) -> Result<Xpub, Error> {
        self.inner.unlock().await?;
        let xpub = self.inner.get_master_xpub().await?;
        Ok(xpub.into())
    }

    /// Return a single sig address with the given `variant` and `path` derivation
    #[wasm_bindgen(js_name = getReceiveAddressSingle)]
    pub async fn get_receive_address_single(
        &self,
        variant: Singlesig,
        path: Vec<u32>,
    ) -> Result<String, Error> {
        let network = self.inner.network();
        self.inner.unlock().await?;
        let xpub = self
            .inner
            .get_receive_address(GetReceiveAddressParams {
                network,
                address: SingleOrMulti::Single {
                    variant: variant.into(),
                    path,
                },
            })
            .await?;
        Ok(xpub.to_string())
    }

    /// Return a multisig address of a registered `multisig_name` wallet
    ///
    /// This method accept `path` and `path_n` in place of a single `Vec<Vec<u32>>` because the
    /// latter is not supported by wasm_bindgen (and neither `(u32, Vec<u32>)`). `path` and `path_n`
    /// are converted internally to a `Vec<Vec<u32>>` with the caveat all the paths are the same,
    /// which is almost always the case.
    #[wasm_bindgen(js_name = getReceiveAddressMulti)]
    pub async fn get_receive_address_multi(
        &self,
        multisig_name: String,
        path: Vec<u32>,
        path_n: u32,
    ) -> Result<String, Error> {
        let network = self.inner.network();
        self.inner.unlock().await?;
        let mut paths = vec![];
        for _ in 0..path_n {
            paths.push(path.clone());
        }
        let xpub = self
            .inner
            .get_receive_address(GetReceiveAddressParams {
                network,
                address: SingleOrMulti::Multi {
                    multisig_name,
                    paths,
                },
            })
            .await?;
        Ok(xpub.to_string())
    }

    /// Sign and consume the given PSET, returning the signed one
    pub async fn sign(&self, pset: Pset) -> Result<Pset, Error> {
        let mut pset: PartiallySignedTransaction = pset.into();
        self.inner.sign(&mut pset).await?;
        Ok(pset.into())
    }

    pub async fn wpkh(&self) -> Result<WolletDescriptor, Error> {
        self.desc(lwk_common::Singlesig::Wpkh).await
    }

    #[wasm_bindgen(js_name = shWpkh)]
    pub async fn sh_wpkh(&self) -> Result<WolletDescriptor, Error> {
        self.desc(lwk_common::Singlesig::ShWpkh).await
    }

    // Asks all possible derivation needed for standard singlesig wallets (fist account)
    async fn create_fake_signer(&self) -> Result<FakeSigner, Error> {
        let network = self.inner.network();
        self.inner.unlock().await?;
        let mut paths = HashMap::new();

        for purpose in [49, 84] {
            for coin_type in [1, 1776] {
                let derivation_path_str = format!("m/{purpose}h/{coin_type}h/0h");
                let derivation_path = DerivationPath::from_str(&derivation_path_str)?;
                let path = derivation_path_to_vec(&derivation_path);
                let params = GetXpubParams { network, path };
                let xpub = self.inner.get_cached_xpub(params).await?;
                paths.insert(derivation_path, xpub);
            }
        }
        let xpub = self.inner.get_master_xpub().await?;
        paths.insert(DerivationPath::master(), xpub);
        let slip77 = self.inner.slip77_master_blinding_key().await?;

        Ok(FakeSigner { paths, slip77 })
    }

    async fn desc(&self, script_variant: lwk_common::Singlesig) -> Result<WolletDescriptor, Error> {
        let signer = self.create_fake_signer().await?;
        let is_mainnet = matches!(self.inner.network(), lwk_jade::Network::Liquid);

        let desc_str = lwk_common::singlesig_desc(
            &signer,
            script_variant,
            DescriptorBlindingKey::Slip77,
            is_mainnet,
        )
        .map_err(|s| Error::Generic(s))?;
        WolletDescriptor::new(&desc_str)
    }
}

#[wasm_bindgen]
pub struct Singlesig {
    inner: lwk_common::Singlesig,
}

impl From<Singlesig> for Variant {
    fn from(v: Singlesig) -> Self {
        match v.inner {
            lwk_common::Singlesig::Wpkh => Variant::Wpkh,
            lwk_common::Singlesig::ShWpkh => Variant::ShWpkh,
        }
    }
}

#[wasm_bindgen]
impl Singlesig {
    pub fn from(variant: &str) -> Result<Singlesig, Error> {
        match variant {
            "Wpkh" => Ok(Singlesig {
                inner: lwk_common::Singlesig::Wpkh,
            }),
            "ShWpkh" => Ok(Singlesig {
                inner: lwk_common::Singlesig::ShWpkh,
            }),
            _ => Err(Error::Generic(
                "Unsupported variant, possible values are: Wpkh and ShWpkh".to_string(),
            )),
        }
    }
}
