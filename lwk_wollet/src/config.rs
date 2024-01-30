use crate::elements::{AddressParams, AssetId};
use crate::error::Error;
use electrum_client::{Client, ConfigBuilder};
use std::str::FromStr;

const LIQUID_POLICY_ASSET_STR: &str =
    "6f0279e9ed041c3d710a9f57d0c02928416460c4b722ae3457a11eec381c526d";
const LIQUID_TESTNET_POLICY_ASSET_STR: &str =
    "144c654344aa716d6f3abcc1ca90e5641e4e2a7f633bc09fe3baf64585819a49";

#[derive(Debug, Clone)]
pub enum ElectrumUrl {
    Tls(String, bool), // the bool value indicates if the domain name should be validated
    Plaintext(String),
}

impl ElectrumUrl {
    pub fn new(electrum_url: &str, tls: bool, validate_domain: bool) -> Self {
        match tls {
            true => ElectrumUrl::Tls(electrum_url.into(), validate_domain),
            false => ElectrumUrl::Plaintext(electrum_url.into()),
        }
    }
    pub fn build_client(&self) -> Result<Client, Error> {
        let builder = ConfigBuilder::new();
        let (url, builder) = match self {
            ElectrumUrl::Tls(url, validate) => {
                (format!("ssl://{}", url), builder.validate_domain(*validate))
            }
            ElectrumUrl::Plaintext(url) => (format!("tcp://{}", url), builder),
        };
        Ok(Client::from_config(&url, builder.build())?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ElementsNetwork {
    Liquid,
    LiquidTestnet,
    ElementsRegtest { policy_asset: AssetId },
}

impl ElementsNetwork {
    pub fn policy_asset(&self) -> AssetId {
        match self {
            ElementsNetwork::Liquid => {
                AssetId::from_str(LIQUID_POLICY_ASSET_STR).expect("can't fail on const")
            }
            ElementsNetwork::LiquidTestnet => {
                AssetId::from_str(LIQUID_TESTNET_POLICY_ASSET_STR).expect("can't fail on const")
            }
            ElementsNetwork::ElementsRegtest { policy_asset } => *policy_asset,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ElementsNetwork::Liquid => "liquid",
            ElementsNetwork::LiquidTestnet => "liquid-testnet",
            ElementsNetwork::ElementsRegtest { .. } => "liquid-regtest",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    network: ElementsNetwork,
    data_root: Option<String>,
}

impl Config {
    pub fn new(network: ElementsNetwork, data_root: Option<&str>) -> Result<Self, Error> {
        Ok(Config {
            network,
            data_root: data_root.map(Into::into),
        })
    }

    pub fn address_params(&self) -> &'static AddressParams {
        match self.network {
            ElementsNetwork::Liquid => &AddressParams::LIQUID,
            ElementsNetwork::LiquidTestnet => &AddressParams::LIQUID_TESTNET,
            ElementsNetwork::ElementsRegtest { .. } => &AddressParams::ELEMENTS,
        }
    }

    pub fn policy_asset(&self) -> AssetId {
        self.network.policy_asset()
    }

    pub fn data_root(&self) -> Option<&str> {
        self.data_root.as_deref()
    }
}