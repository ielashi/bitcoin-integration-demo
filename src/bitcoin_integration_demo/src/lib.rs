mod common;
mod types;
mod util;
use types::*;
use bitcoin::{util::psbt::serialize::Serialize as _, Address, PrivateKey};
use hex;
use ic_btc_types::{
    GetBalanceRequest, GetUtxosRequest, GetUtxosResponse, Network, SendTransactionRequest,
};
use ic_cdk::{
    api::call::RejectionCode,
    call,
    export::{
        candid::{CandidType, Deserialize},
        serde::Serialize,
        Principal,
    },
    print, trap,
};
use ic_cdk_macros::{query, update};
use sha2::Digest;
use std::cell::RefCell;
use std::str::FromStr;


#[update]
async fn get_p2pkh_address() -> String {
    let public_key = get_public_key().await;
    print(format!("Public Key {:?}", hex::encode(public_key.clone())));
    crate::util::p2pkh_address_from_public_key(public_key)
}

#[update]
async fn get_public_key() -> Vec<u8> {
    let ecdsa_canister_id = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

    #[allow(clippy::type_complexity)]
    let res: (ECDSAPublicKeyReply,) = call(
        ecdsa_canister_id,
        "ecdsa_public_key",
        (ECDSAPublicKey {
            canister_id: None,
            derivation_path: vec![vec![0]],
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: String::from("test"),
            },
        },),
    )
    .await
    .unwrap();

    res.0.public_key
}

#[update]
async fn get_balance(address: String) -> u64 {
    let balance_res: Result<(u64,), _> = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "bitcoin_get_balance",
        (GetBalanceRequest {
            address,
            network: Network::Regtest,
            min_confirmations: None,
        },),
        100_000_000,
    )
    .await;

    balance_res.unwrap().0
}

#[update]
async fn get_utxos(address: String) -> GetUtxosResponse {
    let utxos_res: Result<(GetUtxosResponse,), _> = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "bitcoin_get_utxos",
        (GetUtxosRequest {
            address,
            network: Network::Regtest,
            filter: None,
        },),
        100_000_000,
    )
    .await;

    utxos_res.unwrap().0
}

#[update]
async fn send_transaction(transaction: Vec<u8>) {
    let res: Result<(), _> = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "bitcoin_send_transaction",
        (SendTransactionRequest {
            network: Network::Regtest,
            transaction,
        },),
        1_000_000_000_000,
    )
    .await;

    res.unwrap();
}

#[update]
pub async fn send(destination: String) {
    let amount = 1_0000_0000;
    let fees: u64 = 10_000;

    if amount <= fees {
        trap("Amount must be higher than the fee of 10,000 satoshis")
    }

    let destination = match Address::from_str(&destination) {
        Ok(destination) => destination,
        Err(_) => trap("Invalid destination address"),
    };

    let our_address = get_p2pkh_address().await;

    print(&format!("BTC address: {}", our_address));

    // Fetch our UTXOs.
    let utxos = get_utxos(our_address.clone()).await.utxos;

    let spending_transaction = crate::common::build_transaction(
        utxos,
        Address::from_str(&our_address).unwrap(),
        destination,
        amount,
        fees,
    )
    .unwrap_or_else(|err| {
        trap(&format!("Error building transaction: {}", err));
    });

    let tx_bytes = spending_transaction.serialize();
    print(&format!("Transaction to sign: {}", hex::encode(tx_bytes)));

    // Sign transaction
    let signed_transaction = crate::common::sign_transaction(
        spending_transaction,
        Address::from_str(&our_address).unwrap(),
        get_public_key().await,
    )
    .await;

    let signed_transaction_bytes = signed_transaction.serialize();
    print(&format!(
        "Signed transaction: {}",
        hex::encode(signed_transaction_bytes.clone())
    ));

    print("Sending transaction");

    send_transaction(signed_transaction_bytes).await;
    print("Done");
}
