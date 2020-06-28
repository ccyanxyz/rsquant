use crate::errors::*;
use crate::models::*;
use url::Url;

use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use tungstenite::client::AutoStream;
use tungstenite::handshake::client::Response;
use tungstenite::protocol::WebSocket;
use tungstenite::{connect, Message};

static WEBSOCKET_URL: &'static str = "wss://stream.binance.com:9443/ws/";

static OUTBOUND_ACCOUNT_INFO: &'static str = "outboundAccountInfo";
static EXECUTION_REPORT: &'static str = "executionReport";

static KLINE: &'static str = "kline";
static AGGREGATED_TRADE: &'static str = "aggTrade";
static DEPTH_ORDERBOOK: &'static str = "depthUpdate";
static PARTIAL_ORDERBOOK: &'static str = "lastUpdateId";

static DAYTICKER: &'static str = "24hrTicker";

pub enum WebsocketEvent {
    AccountUpdate(AccountUpdateEvent),
    OrderTrade(OrderTradeEvent),
    Trade(TradesEvent),
    OrderBook(OrderBook),
    DayTicker(Vec<DayTickerEvent>),
    Kline(KlineEvent),
    DepthOrderBook(DepthOrderBookEvent),
    BookTicker(BookTickerEvent),
}

pub struct BinanceWs<'a> {
    pub socket: Option<(WebSocket<AutoStream>, Response)>,
    handler: Box<dyn FnMut(WebsocketEvent) -> Result<()> + 'a>,
}

impl<'a> BinanceWs<'a> {
    pub fn new<Callback>(handler: Callback) -> BinanceWs<'a>
    where
        Callback: FnMut(WebsocketEvent) -> Result<()> + 'a,
    {
        BinanceWs {
            socket: None,
            handler: Box::new(handler),
        }
    }

    pub fn connect(&mut self, endpoint: &str) -> Result<()> {
        let wss: String = format!("{}{}", WEBSOCKET_URL, endpoint);
        let url = Url::parse(&wss)?;

        match connect(url) {
            Ok(resp) => {
                self.socket = Some(resp);
                Ok(())
            }
            Err(e) => {
                bail!(format!("Error during handshake: {}", e));
            }
        }
    }

    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(ref mut socket) = self.socket {
            socket.0.close(None)?;
            Ok(())
        } else {
            bail!("Connection close failed");
        }
    }

    pub fn event_loop(&mut self, running: &AtomicBool) -> Result<()> {
        while running.load(Ordering::Relaxed) {
            if let Some(ref mut socket) = self.socket {
                let msg = socket.0.read_message()?;
                let val: Value = serde_json::from_str(msg.to_text()?)?;
                match msg {
                    Message::Text(m) => {
                        if val["u"] != Value::Null
                            && val["s"] != Value::Null
                            && val["b"] != serde_json::Value::Null
                            && val["B"] != serde_json::Value::Null
                            && val["a"] != serde_json::Value::Null
                            && val["A"] != serde_json::Value::Null
                        {
                            let book_ticker: BookTickerEvent = serde_json::from_str(m.as_str())?;
                            (self.handler)(WebsocketEvent::BookTicker(book_ticker))?;
                        } else if m.find(OUTBOUND_ACCOUNT_INFO) != None {
                            let account_update: AccountUpdateEvent =
                                serde_json::from_str(m.as_str())?;
                            (self.handler)(WebsocketEvent::AccountUpdate(account_update))?;
                        } else if m.find(EXECUTION_REPORT) != None {
                            let order_trade: OrderTradeEvent = serde_json::from_str(m.as_str())?;
                            (self.handler)(WebsocketEvent::OrderTrade(order_trade))?;
                        } else if m.find(AGGREGATED_TRADE) != None {
                            let trade: TradesEvent = serde_json::from_str(m.as_str())?;
                            (self.handler)(WebsocketEvent::Trade(trade))?;
                        } else if m.find(DAYTICKER) != None {
                            let trades: Vec<DayTickerEvent> = serde_json::from_str(m.as_str())?;
                            (self.handler)(WebsocketEvent::DayTicker(trades))?;
                        } else if m.find(KLINE) != None {
                            let kline: KlineEvent = serde_json::from_str(m.as_str())?;
                            (self.handler)(WebsocketEvent::Kline(kline))?;
                        } else if m.find(PARTIAL_ORDERBOOK) != None {
                            let partial_orderbook: OrderBook = serde_json::from_str(m.as_str())?;
                            (self.handler)(WebsocketEvent::OrderBook(partial_orderbook))?;
                        } else if m.find(DEPTH_ORDERBOOK) != None {
                            let depth_orderbook: DepthOrderBookEvent =
                                serde_json::from_str(m.as_str())?;
                            (self.handler)(WebsocketEvent::DepthOrderBook(depth_orderbook))?;
                        }
                    }
                    Message::Ping(_) | Message::Pong(_) | Message::Binary(_) => {}
                    Message::Close(e) => {
                        bail!(format!("Disconnected: {:?}", e));
                    }
                }
            }
        }
        Ok(())
    }
}

static USER_DATA_STREAM: &'static str = "/api/v3/userDataStream";
use super::binance::Binance;

pub struct BinanceUserStream {
    pub client: Binance,
}

impl BinanceUserStream {
    pub fn start(&self) -> Result<String> {
        let ret = self.client.post(USER_DATA_STREAM)?;
        Ok(ret)
    }

    pub fn keep_alive(&self, listen_key: &str) -> Result<()> {
        self.client.put(USER_DATA_STREAM, listen_key)?;
        Ok(())
    }

    pub fn close(&self, listen_key: &str) -> Result<()> {
        self.client.delete(USER_DATA_STREAM, listen_key)?;
        Ok(())
    }
}
