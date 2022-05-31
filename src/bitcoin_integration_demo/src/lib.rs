use ic_btc_types::{GetBalanceRequest, GetUtxosRequest, GetUtxosResponse, Network};
use ic_cdk::export::Principal;
use ic_cdk_macros::update;

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
