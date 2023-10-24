use bs_containers::testcontainers::clients::Cli;
use elements::bitcoin::bip32::DerivationPath;
use jade::{
    get_receive_address::Variant,
    mutex_jade::MutexJade,
    register_multisig::{JadeDescriptor, RegisterMultisigParams},
};
use signer::Signer;
use std::{convert::TryInto, str::FromStr};
use wollet::WolletDescriptor;

use crate::{
    jade_emulator::inner_jade_debug_initialization,
    test_session::{generate_signer, generate_slip77, setup, TestWollet},
    TEST_MNEMONIC,
};

#[cfg(feature = "serial")]
mod serial {
    use jade::{
        get_receive_address::Variant, mutex_jade::MutexJade, protocol::JadeState, serialport, Jade,
    };
    use signer::Signer;
    use std::time::Duration;

    use crate::change_detection::{send_lbtc_detect_change, send_lbtc_detect_change_multisig};

    #[test]
    #[ignore = "requires hardware jade: initialized with localtest network, connected via usb/serial"]
    fn jade_send_lbtc_detect_change() {
        let network = jade::Network::LocaltestLiquid;

        let ports = serialport::available_ports().unwrap();
        assert!(!ports.is_empty());
        let path = &ports[0].port_name;
        let port = serialport::new(path, 115_200)
            .timeout(Duration::from_secs(60))
            .open()
            .unwrap();

        let jade = Jade::new(port.into(), network);
        let mut jade = MutexJade::new(jade);

        let mut jade_state = jade.get_mut().unwrap().version_info().unwrap().jade_state;
        assert_ne!(jade_state, JadeState::Uninit);
        assert_ne!(jade_state, JadeState::Unsaved);
        if jade_state == JadeState::Locked {
            jade.unlock().unwrap();
            jade_state = jade.get_mut().unwrap().version_info().unwrap().jade_state;
        }
        assert_eq!(jade_state, JadeState::Ready);
        let signers = [&Signer::Jade(&jade)];

        send_lbtc_detect_change(&signers, Variant::Wpkh);
        send_lbtc_detect_change(&signers, Variant::ShWpkh);

        // refuse the tx on the jade to keep the session logged
        jade.get_mut().unwrap().logout().unwrap();
    }

    #[test]
    fn jade_send_lbtc_detect_change_multisig() {
        let network = jade::Network::LocaltestLiquid;

        let ports = serialport::available_ports().unwrap();
        assert!(!ports.is_empty());
        let path = &ports[0].port_name;
        let port = serialport::new(path, 115_200)
            .timeout(Duration::from_secs(60))
            .open()
            .unwrap();

        let jade = Jade::new(port.into(), network);
        let mut jade = MutexJade::new(jade);

        let mut jade_state = jade.get_mut().unwrap().version_info().unwrap().jade_state;
        assert_ne!(jade_state, JadeState::Uninit);
        assert_ne!(jade_state, JadeState::Unsaved);
        if jade_state == JadeState::Locked {
            jade.unlock().unwrap();
            jade_state = jade.get_mut().unwrap().version_info().unwrap().jade_state;
        }
        assert_eq!(jade_state, JadeState::Ready);

        send_lbtc_detect_change_multisig(jade);
    }
}

#[test]
fn emul_send_lbtc_detect_change() {
    let docker = Cli::default();
    let jade_init = inner_jade_debug_initialization(&docker, TEST_MNEMONIC.to_string());
    let signers = [&Signer::Jade(&jade_init.jade)];

    send_lbtc_detect_change(&signers, Variant::Wpkh);
    send_lbtc_detect_change(&signers, Variant::ShWpkh);
}

fn send_lbtc_detect_change(signers: &[&Signer], variant: Variant) {
    let (variant, path, closing) = match variant {
        Variant::Wpkh => ("elwpkh", "84h/1h/0h", ""),
        Variant::ShWpkh => ("elsh(wpkh", "49h/1h/0h", ")"),
    };
    let master_node = signers[0].xpub().unwrap();
    let fingerprint = master_node.fingerprint();
    let xpub = signers[0]
        .derive_xpub(&DerivationPath::from_str(&format!("m/{path}")).unwrap())
        .unwrap();

    let slip77_key = generate_slip77();

    // m / purpose' / coin_type' / account' / change / address_index
    let desc_str =
        format!("ct(slip77({slip77_key}),{variant}([{fingerprint}/{path}]{xpub}/1/*){closing})");

    let server = setup();

    let mut wallet = TestWollet::new(&server.electrs.electrum_url, &desc_str);

    wallet.fund_btc(&server);

    let node_address = server.node_getnewaddress();
    wallet.send_btc(signers, None, Some((node_address, 10_000)));
}

#[test]
fn emul_send_lbtc_detect_change_multisig() {
    let docker = Cli::default();
    let jade_init = inner_jade_debug_initialization(&docker, TEST_MNEMONIC.to_string());
    send_lbtc_detect_change_multisig(jade_init.jade)
}

fn send_lbtc_detect_change_multisig(mut s1: MutexJade) {
    let s1_xpub = s1.get_mut().unwrap().get_master_xpub().unwrap();
    let s1_fingerprint = s1_xpub.fingerprint();

    let s2 = generate_signer();
    let s2_xpub = s2.xpub();
    let s2_fingerprint = s2_xpub.fingerprint();

    let slip77 = generate_slip77();

    let desc_str = format!(
        "ct(slip77({slip77}),elwsh(multi(2,[{s1_fingerprint}]{s1_xpub}/1/*,[{s2_fingerprint}]{s2_xpub}/1/*)))"
    );
    let wollet_desc: WolletDescriptor = desc_str.parse().unwrap();
    let descriptor: JadeDescriptor = wollet_desc.as_ref().try_into().unwrap();

    s1.get_mut()
        .unwrap()
        .register_multisig(RegisterMultisigParams {
            network: jade::Network::LocaltestLiquid,
            multisig_name: "peppino".to_string(),
            descriptor,
        })
        .unwrap();

    let server = setup();

    let mut wallet = TestWollet::new(&server.electrs.electrum_url, &desc_str);

    wallet.fund_btc(&server);

    let signers = [&Signer::Jade(&s1), &Signer::Software(s2)];
    let node_address = server.node_getnewaddress();
    wallet.send_btc(&signers, None, Some((node_address, 10_000)));
}