extern crate colorize;

use crate::colorize::AnsiColor;
use futures_util::SinkExt;
use futures_util::{future, pin_mut, StreamExt};
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
    let (_stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    let (ws_stream, _) = connect_async(CONNECTION).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, read) = ws_stream.split();
    write.send(Message::from(r#"{"event": "gs"}"#)).await.ok();
    let stdin_to_ws = stdin_rx.map(Ok).forward(write);

    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            let data_as_json = std::str::from_utf8(&data).unwrap();

            if &data_as_json[0..17] == r#"{"event":"welcome"# {
                return;
            }

            let _v: EthScanWebSocketMessage =
                serde_json::from_str(data_as_json).expect("bad message");

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
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}
