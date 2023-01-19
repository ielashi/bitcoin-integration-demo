use ic_btc_types::{
    GetBalanceRequest, GetCurrentFeePercentilesRequest, GetUtxosRequest, GetUtxosResponse,
    MillisatoshiPerByte, Network, Page, SendTransactionRequest, UtxosFilter,
};
use ic_cdk::{api::call::call_with_payment, export::Principal};
use ic_cdk_macros::update;

const M: u64 = 1_000_000; // One million
const B: u64 = 1_000_000_000; // One billion

// Fees for the various bitcoin endpoints.
const GET_BALANCE_FEE: u64 = 100 * M;
const GET_UTXOS_FEE: u64 = 10 * B;
const GET_CURRENT_FEE_PERCENTILES_FEE: u64 = 10 * M;
const SEND_TRANSACTION_BASE_FEE: u64 = 5 * B;
const SEND_TRANSACTION_FEE_PER_BYTE: u64 = 20 * M;

#[update]
async fn get_balance(address: String) -> u64 {
    let balance_res: Result<(u64,), _> = call_with_payment(
        Principal::management_canister(),
        "bitcoin_get_balance",
        (GetBalanceRequest {
            address,
            network: Network::Testnet,
            min_confirmations: None,
        },),
        GET_BALANCE_FEE,
    )
    .await;

    balance_res.unwrap().0
}

#[update]
async fn get_utxos(address: String, page: Option<Page>) -> GetUtxosResponse {
    let utxos_res: Result<(GetUtxosResponse,), _> = call_with_payment(
        Principal::management_canister(),
        "bitcoin_get_utxos",
        (GetUtxosRequest {
            address,
            network: Network::Testnet,
            filter: page.map(UtxosFilter::Page),
        },),
        GET_UTXOS_FEE,
    )
    .await;

    utxos_res.unwrap().0
}

#[update]
async fn get_current_fee_percentiles() -> Vec<MillisatoshiPerByte> {
    let res: Result<(Vec<MillisatoshiPerByte>,), _> = call_with_payment(
        Principal::management_canister(),
        "bitcoin_get_current_fee_percentiles",
        (GetCurrentFeePercentilesRequest {
            network: Network::Testnet,
        },),
        GET_CURRENT_FEE_PERCENTILES_FEE,
    )
    .await;

    res.unwrap().0
}

#[update]
async fn send_transaction(transaction: Vec<u8>) {
    // A crude check so that we don't spend too many cycles.
    if transaction.len() > 1000 {
        panic!("Transaction too large.");
    }

    let fee =
        SEND_TRANSACTION_BASE_FEE + (transaction.len() as u64) * SEND_TRANSACTION_FEE_PER_BYTE;

    let res: Result<(), _> = call_with_payment(
        Principal::management_canister(),
        "bitcoin_send_transaction",
        (SendTransactionRequest {
            network: Network::Testnet,
            transaction,
        },),
        fee,
    )
    .await;

    res.unwrap()
}
