extern crate colorize;

use crate::colorize::AnsiColor;
use futures_util::SinkExt;
use futures_util::{StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EthScanWebSocketMessage {
    pub dashb: Dashb,
    pub blocks: Vec<Block>,
    pub txns: Vec<Txn>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashb {
    pub marketcap: String,
    pub price: String,
    pub lastblock: String,
    pub dec_open_price: String,
    pub dec_current_price: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub b_no: String,
    pub b_time: String,
    pub b_miner: String,
    pub b_miner_tag: String,
    pub b_txns: String,
    pub b_mtime: String,
    pub b_reward: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Txn {
    pub t_hash: String,
    pub t_from: String,
    pub t_to: String,
    #[serde(rename = "t_contractAddress")]
    pub t_contract_address: String,
    pub t_amt: String,
    pub t_time: String,
}

const CONNECTION: &str = "wss://etherscan.io/wshandler";

#[tokio::main]
async fn main() {
    let (mut ws_stream, _) = connect_async(CONNECTION).await.expect("Failed to connect");
    println!(" ✅ WebSocket handshake");

    while let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if msg.is_text() || msg.is_binary() {
            let msg_as_string = String::from_utf8(msg.into_data()).expect("Found invalid UTF-8");

            if &msg_as_string[0..17] == r#"{"event":"welcome"# {
                println!(" ✅ Subscribed");
                ws_stream
                    .send(Message::from(r#"{"event": "gs"}"#))
                    .await
                    .ok();
                continue;
            }

            let _v: EthScanWebSocketMessage = serde_json::from_str(&msg_as_string).expect("bad message");

            for block in _v.blocks {
                let padded_block = format!(" Block: {} ", block.b_no);
                let padded_string_time = format!(
                    " {} ",
                    chrono::offset::Utc::now().format("%Y-%m-%d %H:%M:%S")
                );
                let padded_miner_tag = format!(" Miner Name: {} ", block.b_miner_tag);
                let padded_miner_addy = format!(" Miner Addy: {} ", block.b_miner);

                let trx_count = format!(" Trx Count: {} ", block.b_txns);
                let mint_time = format!(" Mine Time: {}s ", block.b_mtime);

                println!("\n");
                println!("{}", padded_block.black().bold().b_cyanb());
                println!("{}", padded_string_time.black().bold().b_magentab());
                println!("{}", trx_count.black().bold().b_redb());
                println!("{}", mint_time.black().bold().b_blueb());
                println!("{}", padded_miner_tag.black().bold().b_yellowb());
                println!("{}", padded_miner_addy.black().bold().b_greenb());
            }
        }
    }
}
