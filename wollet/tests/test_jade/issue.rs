use bs_containers::testcontainers::clients::Cli;
use elements::bitcoin::bip32::DerivationPath;
use signer::Signer;
use std::str::FromStr;

use crate::{
    test_jade::init::inner_jade_debug_initialization,
    test_session::{generate_slip77, setup, TestWollet},
    TEST_MNEMONIC,
};

#[cfg(feature = "serial")]
mod serial {
    use crate::test_jade::init::serial;
    use signer::Signer;

    #[test]
    #[ignore = "requires hardware jade: initialized with localtest network, connected via usb/serial"]
    fn jade_issue_asset() {
        let mut jade = serial::unlock();
        let signers = [&Signer::Jade(&jade)];

        super::issue_asset_contract(&signers);

        // refuse the tx on the jade to keep the session logged
        jade.get_mut().unwrap().logout().unwrap();
    }
}

#[test]
fn emul_issue_asset() {
    let docker = Cli::default();
    let jade_init = inner_jade_debug_initialization(&docker, TEST_MNEMONIC.to_string());
    let signers = [&Signer::Jade(&jade_init.jade)];

    issue_asset_contract(&signers);
}

fn issue_asset_contract(signers: &[&Signer]) {
    let path = "84h/1h/0h";
    let master_node = signers[0].xpub().unwrap();
    let fingerprint = master_node.fingerprint();
    let xpub = signers[0]
        .derive_xpub(&DerivationPath::from_str(&format!("m/{path}")).unwrap())
        .unwrap();

    let slip77_key = generate_slip77();

    // m / purpose' / coin_type' / account' / change / address_index
    let desc_str = format!("ct(slip77({slip77_key}),elwpkh([{fingerprint}/{path}]{xpub}/1/*))");

    let server = setup();

    let mut wallet = TestWollet::new(&server.electrs.electrum_url, &desc_str);

    wallet.fund_btc(&server);

    let contract = "{\"entity\":{\"domain\":\"test.com\"},\"issuer_pubkey\":\"0337cceec0beea0232ebe14cba0197a9fbd45fcf2ec946749de920e71434c2b904\",\"name\":\"Test\",\"precision\":2,\"ticker\":\"TEST\",\"version\":0}";

    let (asset, _token) = wallet.issueasset(signers, 1_000, 1, contract, None);
    dbg!(asset); // f56d514
}