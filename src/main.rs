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

// Database Shim
mod database;
use database::{DatabaseType, DatabaseConnInfo, Database};

// Poloniex
use coinnect::poloniex::credentials::PoloniexCreds;
use coinnect::poloniex::api::PoloniexApi;

//// config
use std::collections::HashMap;
use config::File;

enum ExApi {
    Poloniex(PoloniexApi),
}

fn get_snapshot(exchanges : Vec<ExApi>) {

    for ex in exchanges {
        match ex {
            ExApi::Poloniex(mut p) => {
                let ticker = p.return_ticker().unwrap();
                println!("{:?}", ticker);
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

    let db = Database::new(
        DatabaseType::Mongodb,
        DatabaseConnInfo::new(
            settings["database"]["url"].clone(),
            settings["database"]["port"].parse().unwrap_or(27017)
        )
    );

    let poloniex_creds = PoloniexCreds::new(
        &settings["poloniex"]["name"],
        &settings["poloniex"]["api_key"],
        &settings["poloniex"]["api_secret"]
    );

    
    let poloniex_ex = ExApi::Poloniex(PoloniexApi::new(poloniex_creds).unwrap());

    let exchanges = vec![poloniex_ex];

    get_snapshot(exchanges);


}
