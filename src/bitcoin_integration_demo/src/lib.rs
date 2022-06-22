mod common;
mod types;
use bitcoin::{util::psbt::serialize::Serialize, Address, PrivateKey};
use ic_btc_types::{
    GetBalanceRequest, GetUtxosRequest, GetUtxosResponse, Network, SendTransactionRequest,
};
use ic_cdk::{export::Principal, trap, print};
use hex;
use ic_cdk_macros::{update, query};
use std::cell::RefCell;
use std::str::FromStr;
use crate::common::get_p2pkh_address;

// A private key in WIF (wallet import format). This is only for demonstrational purposes.
// When the Bitcoin integration is released on mainnet, canisters will have the ability
// to securely generate ECDSA keys.
const BTC_PRIVATE_KEY_WIF: &str = "L2C1QgyKqNgfV7BpEPAm6PVn2xW8zpXq6MojSbWdH18nGQF2wGsT";

thread_local! {
    static BTC_PRIVATE_KEY: RefCell<PrivateKey> =
        RefCell::new(PrivateKey::from_wif(BTC_PRIVATE_KEY_WIF).unwrap());
}

#[query(name = "btc_address")]
pub fn btc_address_str() -> String {
    btc_address().to_string()
}

#[update]
async fn get_balance(address: String) -> u64 {
    let balance_res: Result<(u64,), _> = ic_cdk::api::call::call(
        Principal::management_canister(),
        "bitcoin_get_balance",
        (GetBalanceRequest {
            address,
            network: Network::Testnet,
            min_confirmations: None,
        },),
    )
    .await;

    balance_res.unwrap().0
}

#[update]
async fn get_utxos(address: String) -> GetUtxosResponse {
    let utxos_res: Result<(GetUtxosResponse,), _> = ic_cdk::api::call::call(
        Principal::management_canister(),
        "bitcoin_get_utxos",
        (GetUtxosRequest {
            address,
            network: Network::Testnet,
            filter: None,
        },),
    )
    .await;

    utxos_res.unwrap().0
}

#[update]
async fn send_transaction(transaction: Vec<u8>) {
    let res: Result<(), _> = ic_cdk::api::call::call(
        Principal::management_canister(),
        "bitcoin_send_transaction",
        (SendTransactionRequest {
            network: Network::Testnet,
            transaction,
        },),
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

    print(&format!("BTC address: {}", btc_address().to_string()));

    // Fetch our UTXOs.
    let utxos = get_utxos(btc_address().to_string()).await.utxos;

    let spending_transaction = crate::common::build_transaction(utxos, btc_address(), destination, amount, fees)
        .unwrap_or_else(|err| {
            trap(&format!("Error building transaction: {}", err));
        });

    print(&format!(
        "Transaction to sign: {}",
        hex::encode(spending_transaction.serialize())
    ));

    // Sign transaction
    let private_key = BTC_PRIVATE_KEY.with(|private_key| *private_key.borrow());
    let signed_transaction = crate::common::sign_transaction(spending_transaction, private_key, btc_address());

    let signed_transaction_bytes = signed_transaction.serialize();
    print(&format!(
        "Signed transaction: {}",
        hex::encode(signed_transaction_bytes.clone())
    ));

    print("Sending transaction");

    send_transaction(signed_transaction_bytes).await;
    print("Done");
}

fn btc_address() -> Address {
    BTC_PRIVATE_KEY.with(|private_key| get_p2pkh_address(&private_key.borrow(), bitcoin::Network::Regtest))
}
