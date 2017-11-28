#[macro_use] extern crate serenity;
#[macro_use(bson, doc)] extern crate bson;
extern crate mongodb;
extern crate coinnect;
extern crate config;
extern crate serde_json;
extern crate chrono;

// Discord
use serenity::client::Client as SerenityClient;
use serenity::framework::standard::StandardFramework;
use serenity::prelude::*;
use serenity::model::*;
use serenity::utils::*;
use serenity::model::{ChannelType, GuildId};

// Expression Parsing
extern crate parser;
use std::collections::HashSet;

// Snapshot 'Photographer' Thread
use std::{time, thread};

// Database
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use chrono::Local;

// Poloniex
use coinnect::poloniex::credentials::PoloniexCreds;
use coinnect::poloniex::api::PoloniexApi;

// config
use std::collections::HashMap;
use config::File;

// Exchange Structs
#[derive(Debug)]
enum ExApi {
    Poloniex(PoloniexApi),
}

#[derive(Debug, Clone)]
enum ExType {
    Poloniex
}

#[derive(Debug, Clone)]
struct ExCreds {
    ex_type : ExType,
    name : String,
    key : String,
    secret : String
}

// Data struct
#[derive(Debug)]
struct MarketData {
    last : f64,
    lowest_ask : f64,
    highest_bid : f64,
    percent_change : f64,
    base_volume : f64,
    quote_volume : f64,
    is_frozen : u64,
    high24hr : f64,
    low24hr : f64,
}

// Data fetching/storing util functions
fn parse_polo_ticker(ticker : serde_json::Map<std::string::String, serde_json::Value>)
-> HashMap<String, HashMap<String, MarketData>> {
    let mut data : HashMap<String, HashMap<String, MarketData>> = HashMap::new();
    for (key, value) in ticker {
        let s : Vec<&str> = key.split("_").collect();
        let market = String::from(s[0]);
        let coin = String::from(s[1]);
        let m = MarketData {
            last: String::from(value["last"].as_str().unwrap()).parse().unwrap(),
            lowest_ask: String::from(value["lowestAsk"].as_str().unwrap()).parse().unwrap(),
            highest_bid: String::from(value["highestBid"].as_str().unwrap()).parse().unwrap(),
            percent_change: String::from(value["percentChange"].as_str().unwrap()).parse().unwrap(),
            base_volume: String::from(value["baseVolume"].as_str().unwrap()).parse().unwrap(),
            quote_volume: String::from(value["quoteVolume"].as_str().unwrap()).parse().unwrap(),
            is_frozen: String::from(value["isFrozen"].as_str().unwrap()).parse().unwrap(),
            high24hr: String::from(value["high24hr"].as_str().unwrap()).parse().unwrap(),
            low24hr: String::from(value["low24hr"].as_str().unwrap()).parse().unwrap(),
        };
        data.entry(coin)
            .or_insert(HashMap::new())
            .insert(market, m);
    }
    return data;
}

fn get_snapshot(exchange_creds : Vec<ExCreds>) 
-> HashMap<String, HashMap<String, HashMap<String, MarketData> > > {
    let exchanges = get_exchanges(exchange_creds);
    let mut ex_data : HashMap<String, HashMap<String, HashMap<String, MarketData> > > = HashMap::new();
    for ex in exchanges {
        match ex {
            ExApi::Poloniex(mut p) => {
                let ticker = p.return_ticker().unwrap();
                let polo_data = parse_polo_ticker(ticker);
                ex_data.insert(String::from("poloniex"), polo_data);
            }
        }
    }
    return ex_data;
}

fn get_exchanges(exchange_creds : Vec<ExCreds>) -> Vec<ExApi> {
    let mut exchanges = Vec::new();
    for creds in exchange_creds {
        match creds.ex_type {
            ExType::Poloniex => {
                let poloniex_creds = PoloniexCreds::new(
                    &creds.name,
                    &creds.key,
                    &creds.secret
                );
                let poloniex_ex = ExApi::Poloniex(PoloniexApi::new(poloniex_creds).unwrap());
                exchanges.push(poloniex_ex);
            }
        }
    }
    return exchanges;
}

fn store_snapshot(db_client : mongodb::Client, exchange_creds : Vec<ExCreds>) {
    let snapshot = get_snapshot(exchange_creds);
    let date = Local::now();
    let time = date.timestamp();
    let coin_db = db_client.db("coins");
    for (_, ex_data) in snapshot {
        for (coin, market_data) in ex_data {
            let coll = coin_db.collection(&coin);
            for (market, data) in market_data {
                let doc = doc! {
                    "time" : time,
                    "coin" : coin.clone(),
                    "market" : market,
                    "last" : data.last,
                    "lowest_ask" : data.lowest_ask,
                    "highest_bid" : data.highest_bid,
                    "percent_change" : data.percent_change,
                    "base_volume" : data.base_volume,
                    "quote_volume" : data.quote_volume,
                    "is_frozen" : data.is_frozen,
                    "high24hr" : data.high24hr,
                    "low24hr" : data.low24hr
                };
                coll.insert_one(doc.clone(), None)
                    .ok().expect("Failed to insert document.");
            }

        }
    }
}

// Discord API handler and helpers
struct Handler {
    name : String,
    ex_creds : Vec<ExCreds>,
    db_client : Client
}
impl Handler {
    pub fn new(name : String, db_client : Client, ex_creds : Vec<ExCreds>) -> Handler {
        return Handler {
            name: name,
            ex_creds: ex_creds,
            db_client: db_client.clone()
        }
    }
}
impl EventHandler for Handler {
    fn on_message(&self, context : Context, msg: serenity::model::Message) {
        let content_str : &str = &msg.content;
        let mut content_string : String = msg.content.clone();
        if content_str.starts_with("$") {
            content_string.drain(..1);
            let snapshot = get_snapshot(self.ex_creds.clone());
            let split = content_string.split(" ");
            for exp in split {
                let mut exp_parser = parser::Parser::new(String::from(exp).to_uppercase()).unwrap();
                let vars = exp_parser.vars(); 
                for (exchange,exchange_data) in &snapshot {
                    let mut coin_vals = HashMap::new();
                    let mut market_sets = Vec::new();
                    for (coin, coin_data) in exchange_data {
                        if vars.contains(coin) {
                            coin_vals.insert(coin.clone(), coin_data);
                            let mut my_markets = HashSet::new();
                            for (market, market_data) in coin_data {
                                my_markets.insert(market);
                            }
                            market_sets.push(my_markets);

                        }
                    }
                    let mut valid_markets = market_sets.pop().unwrap();
                    for market_set in market_sets {
                        for market in valid_markets.clone() {
                            if !market_set.contains(market) {
                                valid_markets.remove(market);
                            }
                        }
                    }
                    let mut result_message = format!("Results for {}:", exp.to_uppercase());
                    for market in valid_markets {
                        for (coin, coin_datum) in &coin_vals {
                            let data = coin_datum.get(market).unwrap();
                            exp_parser.bind(coin.clone(), data.last);
                        }
                        let result = exp_parser.eval();
                        result_message += &format!("\n\t{}: {}", market, result);
                    }
                    if let Err(why) = msg.channel_id.say(result_message) {
                        println!("Error sending message: {:?}", why);
                    }
                }
            }

        } else if content_str.starts_with("!") {
            content_string.drain(..1);
            let split = content_string.split(" ");
            /*
                !! = show all values
                !last (same as $)
                !lowest_ask exp...
                !highest_bid exp...
                
            */

        }
    }

    fn on_ready(&self, context : Context, ready: Ready) {
        let res = context.edit_profile(|profile| {
            profile.username(&self.name)
        });
    }
}

// Dipper
fn main() {
    // Load settings file with api keys
    let mut settings_raw = config::Config::default();
    settings_raw
        .merge(File::with_name("conf/dipper.toml")).unwrap();
    let settings = settings_raw.deserialize::<HashMap<String, HashMap<String, String>>>().unwrap();

    // Create DB Client
    let db_client = Client::connect(
        &settings["database"]["url"],
        settings["database"]["port"].parse().unwrap_or(27017)
    ).expect("Failed to initialize standalone client.");

    // Populate cred struct for Poloniex
    let polo_cred_data = ExCreds {
        ex_type: ExType::Poloniex,
        name: settings["poloniex"]["name"].clone(),
        key: settings["poloniex"]["api_key"].clone(),
        secret: settings["poloniex"]["api_secret"].clone()
    };
    let exchange_creds = vec![polo_cred_data];

    // Initilize 'Photographer' thread for caputring snapshots
    let autoshot_exchange_creds = exchange_creds.clone();
    let autoshot_db_client = db_client.clone();
    let snapshot_frequency = time::Duration::from_secs(60);
    thread::spawn(move || {
        loop {
            store_snapshot(autoshot_db_client.clone(), autoshot_exchange_creds.clone());
            thread::sleep(snapshot_frequency);
        }
    });

    // Create & start Discord Client
    let discord_exchange_creds = exchange_creds.clone();
    let discord_db_client = db_client.clone();
    let handler = Handler::new(String::from("Dipper"), discord_db_client, discord_exchange_creds);
    let mut client = SerenityClient::new(&settings["discord"]["token"], handler);
    let _ = client.start();
    
}