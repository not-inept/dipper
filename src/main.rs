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


extern crate coinnect;
extern crate config;

// Kraken
use coinnect::kraken::KrakenCreds;
use coinnect::kraken::api::KrakenApi;
// Poloniex
use coinnect::poloniex::credentials::PoloniexCreds;
use coinnect::poloniex::api::PoloniexApi;
//// config
use std::collections::HashMap;
use config::File;

enum ExApi {
    Poloniex(PoloniexApi),
    Kraken(KrakenApi)
}

fn get_snapshot(exchanges : Vec<ExApi>) {

    for ex in exchanges {
        match ex {
            ExApi::Poloniex(mut p) => {
                let ticker = p.return_ticker().unwrap();
            },
            ExApi::Kraken(_k) => {
            }
        }
    }
}

fn main() {
    // Load settings file with api keys
    let mut settings_raw = config::Config::default();
    settings_raw
        .merge(File::with_name("conf/dipper.toml")).unwrap();

    let settings = settings_raw.deserialize::<HashMap<String, HashMap<String, String>>>().unwrap();
    println!("{:?}", settings);

    let kraken_creds = KrakenCreds::new(
        &settings["kraken"]["name"],
        &settings["kraken"]["api_key"],
        &settings["kraken"]["api_secret"]
    );

    let poloniex_creds = PoloniexCreds::new(
        &settings["poloniex"]["name"],
        &settings["poloniex"]["api_key"],
        &settings["poloniex"]["api_secret"]
    );

    let mut kraken_api = ExApi::Kraken(KrakenApi::new(kraken_creds).unwrap());
    let mut poloniex_api = ExApi::Poloniex(PoloniexApi::new(poloniex_creds).unwrap());

    let exchanges = vec![kraken_api, poloniex_api];

    get_snapshot(exchanges);
    // We create a Coinnect Generic API
    // Since Kraken does not need customer_id field, we set it to None
    // let kraken_creds = KrakenCreds::new("my_optionnal_name", "api_key", "api_secret");
    // let mut my_api = Coinnect::new(Kraken, my_creds).unwrap();
    //ovol8671
    // let ticker = my_api.ticker(ETC_BTC);
    //
    // println!("ETC_BTC last trade price is {}.",
    //          ticker.unwrap().last_trade_price);

     // Let's look at the ticker!

    //  for coin in list_coins {
    //      // please visit Poloniex API documentation to know how the data is returned
    //      // or look at the coinnect documentation
    //      let name = coin.0;
    //      let price = coin.1.as_object().unwrap().get("last").unwrap().as_str().unwrap();
     //
    //      println!("Coin {} has price : {}", name, price);
    //  }
}
