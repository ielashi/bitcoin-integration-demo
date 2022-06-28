mod common;
mod types;
use crate::common::get_p2pkh_address;
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

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
struct X {
    public_key: Vec<u8>,
    chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
struct EcdsaKeyId {
    pub curve: EcdsaCurve,
    pub name: String,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
pub enum EcdsaCurve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

#[derive(CandidType, Deserialize, Debug)]
struct SignWithECDSAReply {
    pub signature: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
struct ECDSAPublicKey {
    pub canister_id: Option<Principal>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Serialize, Debug)]
struct SignWithECDSA {
    pub message_hash: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

// TODO: cache the address.
#[update]
async fn get_address() -> String {
    let ecdsa_canister_id = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

    #[allow(clippy::type_complexity)]
    let res: (X,) = call(
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
    print(format!(
        "Public Key {:?}",
        hex::encode(res.0.public_key.clone())
    ));

    // sha256 + ripmd160
    let mut hasher = ripemd::Ripemd160::new();
    hasher.update(sha256(res.0.public_key));
    let result = hasher.finalize();

    // mainnet: 0x00, testnet: 0x6f
    let mut data_with_prefix = vec![0x6f];
    data_with_prefix.extend(result);

    //let data_with_prefix_b58 = bs58::encode(data_with_prefix);
    // TODO: get rid of clone?
    let checksum = &sha256(sha256(data_with_prefix.clone()))[..4];

    let mut full_address = data_with_prefix;
    full_address.extend(checksum);

    bs58::encode(full_address).into_string()
}

async fn get_public_key() -> Vec<u8> {
    let ecdsa_canister_id = Principal::from_text("rwlgt-iiaaa-aaaaa-aaaaa-cai").unwrap();

    #[allow(clippy::type_complexity)]
    let res: (X,) = call(
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
    print(format!(
        "Public Key {:?}",
        hex::encode(res.0.public_key.clone())
    ));

    res.0.public_key
}

fn sha256(data: Vec<u8>) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/*
#[query(name = "btc_address")]
pub fn btc_address_str() -> String {
    btc_address().to_string()
}*/

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
        100_000_000
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
        1_000_000_000_000
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

    let our_address = get_address().await;

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
