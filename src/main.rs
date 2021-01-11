use std::env;

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use websocket_lite::{Message, Opcode, Result};
use serde_json::Value;
use substrate_subxt::{ClientBuilder, PairSigner, NodeTemplateRuntime, Client};
use substrate_subxt::polkadex::{RegisterNewOrderbookCall, OrderType, SubmitOrder};
use substrate_subxt::sp_runtime::testing::H256;
use substrate_subxt::sp_runtime::sp_std::str::FromStr;
use sp_core::{sr25519::Pair, Pair as PairT};
use sp_keyring::AccountKeyring;
use substrate_subxt::balances::TransferCallExt;
use substrate_subxt::balances::TransferEventExt;
use std::thread::sleep;
use std::time;
use std::time::Duration;

// const UNIT: u128 = 1_000_000_000_000;
const UNIT_REP: u128 = 1_000_000_000;

async fn run() -> Result<()> {
    let alice_client = ClientBuilder::<NodeTemplateRuntime>::new()
        .set_url("ws://127.0.0.1:9955")
        .build()
        .await?;
    let mut alice_nonce = 0;
    const time_delay: Duration = time::Duration::from_secs(1);
    for iteration in 0..100000 { // I am trying to simulate 200 transactions passed every 1 second.
        send_transfer(&alice_client, alice_nonce).await;
        alice_nonce = alice_nonce + 1;
        if iteration % 200 == 0 {
            sleep(time_delay)
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        run().await.unwrap_or_else(|e| {
            eprintln!("{}", e);
        })
    })
        .await
        .unwrap();
}

async fn send_transfer(client: &Client<NodeTemplateRuntime>, alice_nonce: u32) -> Result<()> {
    println!("Nonce: {}", alice_nonce);
    let dest = AccountKeyring::Bob.to_account_id().into();
    let mut signer = PairSigner::<NodeTemplateRuntime, _>::new(AccountKeyring::Alice.pair());
    signer.set_nonce(alice_nonce);
    let result = client.transfer(&signer, &dest, 10_000).await?;
    println!(" Transaction Placed #{}", result);
    Ok(())
}

