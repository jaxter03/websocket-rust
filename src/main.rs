use std::env;

use futures::sink::SinkExt;
use futures::stream::StreamExt;
use websocket_lite::{Message, Opcode, Result};
use serde_json::Value;
use substrate_subxt::{ClientBuilder, PairSigner, NodeTemplateRuntime, Client};
use substrate_subxt::generic_asset::{CreateCall, AssetOptions, PermissionsV1, Owner};
use substrate_subxt::polkadex::{RegisterNewOrderbookCall, OrderType, SubmitOrder};
use substrate_subxt::sp_runtime::testing::H256;
use substrate_subxt::sp_runtime::sp_std::str::FromStr;
use sp_core::{sr25519::Pair, Pair as PairT};

const UNIT: u128 = 1_000_000_000_000;
const UNIT_REP: u128 = 1_000_000_000;

// struct Data {
// e: String,  // Event type
// E: f64,   // Event time
// s: f64,    // Symbol
// a: 12345,       // Aggregate trade ID
// p: "0.001",     // Price
// q: "100",       // Quantity
// f: 100,         // First trade ID
// l: 105,         // Last trade ID
// T: 123456785,   // Trade time
// m: true,        // Is the buyer the market maker?
// M: true         // Ignore
// }

async fn run() -> Result<()> {
    let alice_client = ClientBuilder::<NodeTemplateRuntime>::new()
        .set_url("ws://127.0.0.1:9955")
        .build()
        .await?;
    let mut alice_nonce: u32 = initial_calls(alice_client.clone()).await?;

    let url = env::args().nth(1).unwrap_or_else(|| "wss://stream.binance.com:9443/ws/btcusdt@aggTrade".to_owned());
    let builder = websocket_lite::ClientBuilder::new(&url)?;
    let mut ws_stream = builder.async_connect().await?;

    loop {
        let msg: Option<Result<Message>> = ws_stream.next().await;

        let msg = if let Some(msg) = msg {
            msg
        } else {
            break;
        };

        let msg = if let Ok(msg) = msg {
            msg
        } else {
            //let _ = ws_stream.send(Message::close(None)).await;
            break;
        };

        match msg.opcode() {
            Opcode::Text => {
                let data = msg.as_text().unwrap();
                let v: Value = serde_json::from_str(data)?;
                repetitive_calls(alice_client.clone(), v, alice_nonce).await?;
                alice_nonce = alice_nonce + 1
            }
            Opcode::Binary => {}  // ws_stream.send(msg).await?,
            Opcode::Ping => ws_stream.send(Message::pong(msg.into_data())).await?,
            Opcode::Close => {
                let _ = ws_stream.send(Message::close(None)).await;
                break;
            }
            Opcode::Pong => {}
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

async fn repetitive_calls(client: Client<NodeTemplateRuntime>, v: Value, alice_nonce: u32) -> Result<()> {
    println!("Nonce: {}", alice_nonce);
    let submit_trade_call = SubmitOrder {
        order_type: if v["m"].as_bool().unwrap() { OrderType::BidLimit } else { OrderType::AskLimit },
        trading_pair: H256::from_str("f28a3c76161b8d5723b6b8b092695f418037c747faa2ad8bc33d8871f720aac9").unwrap(),
        price: (1000f64 * v["p"].to_owned().as_str().unwrap().parse::<f64>().unwrap()).round() as u128 * UNIT_REP,
        quantity: (1000f64 * v["q"].to_owned().as_str().unwrap().parse::<f64>().unwrap()).round() as u128 * UNIT_REP,
    };
    let load_key = Pair::from_string("tube soldier vehicle position betray vibrant knife canyon armed accident desk flee",None);
    let mut signer = PairSigner::<NodeTemplateRuntime, _>::new(load_key.unwrap());
    signer.set_nonce(alice_nonce);
    let result = client.submit(submit_trade_call, &signer).await?;
    println!(" Trade Placed #{}", result);
    Ok(())
}


async fn initial_calls(client: Client<NodeTemplateRuntime>) -> Result<u32> {
    let load_key = Pair::from_string("tube soldier vehicle position betray vibrant knife canyon armed accident desk flee",None);
    let mut signer = PairSigner::<NodeTemplateRuntime, _>::new(load_key.unwrap());

    let mut alice_nonce = 0;
    // Register BTC/USD Orderbook
    let register_orderbook_call = RegisterNewOrderbookCall {
        quote_asset_id: 2 as u32,
        base_asset_id: 1 as u32,
    };
    signer.set_nonce(alice_nonce);
    let result = client.submit(register_orderbook_call, &signer).await?;
    println!(" Order book Registered: {}", result);
    alice_nonce = alice_nonce + 1;
    Ok(alice_nonce)
}
