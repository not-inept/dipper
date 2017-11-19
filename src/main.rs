// #[macro_use] extern crate serenity;
//
// extern crate config;
// extern crate coinnect;

// use serenity::client::Client;
// use serenity::framework::standard::StandardFramework;
// use serenity::prelude::*;
// use serenity::model::*;
// use serenity::utils::*;
// use serenity::model::{ChannelType, GuildId};
// use std::env;
//
// struct Handler;
//
// fn handle_primary_function(type : String, msg : String, channel_id :  ChannelId) {
//     println!("{}", msg);
// }
//
// impl EventHandler for Handler {
//     fn on_message(&self, context : Context, msg: serenity::model::Message) {
//         let content_str : &str = &msg.content;
//         if content_str[0] == "!" {
//             handle_primary_function(content_str[0], content_str[1..]);
//             if let Err(why) = msg.channel_id.say("Pong!") {
//                 println!("Error sending message: {:?}", why);
//             }
//         }
//     }
//
//     fn on_ready(&self, context : Context, ready: Ready) {
//         let res = context.edit_profile(|profile| {
//             profile.username("ShirleyAnnBotson")
//         });
//     }
// }
//
// fn main() {
//     // Login with a bot token
//     let mut client = Client::new(&discord["token"], Handler);
//     let _ = client.start();
// }

#[macro_use(bson, doc)]
extern crate bson;
extern crate mongodb;
extern crate coinnect;
extern crate config;
extern crate serde_json;
extern crate chrono;

// Database
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;
use chrono::Local;


// Poloniex
use coinnect::poloniex::credentials::PoloniexCreds;
use coinnect::poloniex::api::PoloniexApi;

//// config
use std::collections::HashMap;
use config::File;

enum ExApi {
    Poloniex(PoloniexApi),
}

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

fn parse_polo_ticker(ticker : serde_json::Map<std::string::String, serde_json::Value>
) -> HashMap<String, HashMap<String, MarketData>> {
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

fn get_snapshot(exchanges : Vec<ExApi>)
-> HashMap<String, HashMap<String, HashMap<String, MarketData> > > {
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

fn store_snapshot(db_client : mongodb::Client, exchanges : Vec<ExApi>) {
    let snapshot = get_snapshot(exchanges);
    let coin_db = db_client.db("coins");
    for (_, ex_data) in snapshot {
        for (coin, market_data) in ex_data {
            let coll = coin_db.collection(&coin);
            let date = Local::now();
            let doc = doc! {
                "time" : date.timestamp(),
                "data" : market_data
            };

            coll.insert_one(doc.clone(), None)
            .ok().expect("Failed to insert document.");
        }
    }
}

fn main() {
    // Load settings file with api keys
    let mut settings_raw = config::Config::default();
    settings_raw
        .merge(File::with_name("conf/dipper.toml")).unwrap();

    let settings = settings_raw.deserialize::<HashMap<String, HashMap<String, String>>>().unwrap();

    let db_client = Client::connect(
        &settings["database"]["url"],
        settings["database"]["port"].parse().unwrap_or(27017)
    ).expect("Failed to initialize standalone client.");

    let poloniex_creds = PoloniexCreds::new(
        &settings["poloniex"]["name"],
        &settings["poloniex"]["api_key"],
        &settings["poloniex"]["api_secret"]
    );

    let poloniex_ex = ExApi::Poloniex(PoloniexApi::new(poloniex_creds).unwrap());

    let exchanges = vec![poloniex_ex];

    // get_snapshot(exchanges);
    store_snapshot(db_client, exchanges);

}
