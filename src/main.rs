#[macro_use(bson, doc)]
extern crate bson;
extern crate chrono;
extern crate coinnect;
extern crate config;
extern crate mongodb;
extern crate serde_json;
extern crate serenity;
extern crate typemap;
extern crate gnuplot;

// Discord
use serenity::client::Client as SerenityClient;
//use serenity::framework::standard::StandardFramework;
use serenity::prelude::*;
use serenity::model::*;
use typemap::Key;
//use serenity::utils::*;
//use serenity::model::{ChannelType, GuildId};

// Expression Parsing
extern crate parser;
use std::collections::HashSet;
use gnuplot::{Figure, Caption, Color};
// Snapshot 'Photographer' Thread
use std::{thread, time};

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
use std::fs::File as rsFile;
use std::io::prelude::*;

// Exchange Structs
#[derive(Debug)]
enum ExApi {
    Poloniex(PoloniexApi),
}

#[derive(Debug, Clone)]
enum ExType {
    Poloniex,
}

#[derive(Debug, Clone)]
struct ExCreds {
    ex_type: ExType,
    name: String,
    key: String,
    secret: String,
}
#[derive(Debug)]
enum MarketPropertyType {
    Last,
    LowestAsk,
    HighestBid,
    PercentChange,
    BaseVolume,
    QuoteVolume,
    IsFrozen,
    High24hr,
    Low24hr,
}
#[derive(Debug)]
struct MarketProperty {
    property_type: MarketPropertyType,
    var_string: String,
    coin: String,
}
impl MarketProperty {
    fn new(v: String, c: String, p: String) -> MarketProperty {
        let p_type = match p.to_lowercase().as_str() {
            "lowest_ask" => MarketPropertyType::LowestAsk,
            "highest_bid" => MarketPropertyType::HighestBid,
            "percent_change" => MarketPropertyType::PercentChange,
            "base_volume" => MarketPropertyType::BaseVolume,
            "quote_volume" => MarketPropertyType::QuoteVolume,
            "is_frozen" => MarketPropertyType::IsFrozen,
            "high24hr" => MarketPropertyType::High24hr,
            "low24hr" => MarketPropertyType::Low24hr,
            _ => MarketPropertyType::Last,
        };
        return MarketProperty {
            property_type: p_type,
            var_string: v,
            coin: c,
        };
    }
    fn val(&self, m: &MarketData) -> f64 {
        match self.property_type {
            MarketPropertyType::Last => m.last,
            MarketPropertyType::LowestAsk => m.lowest_ask,
            MarketPropertyType::HighestBid => m.highest_bid,
            MarketPropertyType::PercentChange => m.percent_change,
            MarketPropertyType::BaseVolume => m.base_volume,
            MarketPropertyType::QuoteVolume => m.quote_volume,
            MarketPropertyType::IsFrozen => m.is_frozen as f64,
            MarketPropertyType::High24hr => m.high24hr,
            MarketPropertyType::Low24hr => m.low24hr,
        }
    }
    fn var(&self) -> String {
        self.var_string.clone()
    }
}
// Data struct
#[derive(Debug, Clone)]
struct MarketData {
    last: f64,
    lowest_ask: f64,
    highest_bid: f64,
    percent_change: f64,
    base_volume: f64,
    quote_volume: f64,
    is_frozen: u64,
    high24hr: f64,
    low24hr: f64,
}

// Data fetching/storing util functions
fn parse_polo_ticker(
    ticker: serde_json::Map<std::string::String, serde_json::Value>,
) -> HashMap<String, HashMap<String, MarketData>> {
    let mut data: HashMap<String, HashMap<String, MarketData>> = HashMap::new();
    for (key, value) in ticker {
        let s: Vec<&str> = key.split("_").collect();
        let market = String::from(s[0]);
        let coin = String::from(s[1]);
        let m = MarketData {
            last: String::from(value["last"].as_str().unwrap())
                .parse()
                .unwrap(),
            lowest_ask: String::from(value["lowestAsk"].as_str().unwrap())
                .parse()
                .unwrap(),
            highest_bid: String::from(value["highestBid"].as_str().unwrap())
                .parse()
                .unwrap(),
            percent_change: String::from(value["percentChange"].as_str().unwrap())
                .parse()
                .unwrap(),
            base_volume: String::from(value["baseVolume"].as_str().unwrap())
                .parse()
                .unwrap(),
            quote_volume: String::from(value["quoteVolume"].as_str().unwrap())
                .parse()
                .unwrap(),
            is_frozen: String::from(value["isFrozen"].as_str().unwrap())
                .parse()
                .unwrap(),
            high24hr: String::from(value["high24hr"].as_str().unwrap())
                .parse()
                .unwrap(),
            low24hr: String::from(value["low24hr"].as_str().unwrap())
                .parse()
                .unwrap(),
        };
        data.entry(coin).or_insert(HashMap::new()).insert(market, m);
    }
    return data;
}

fn get_snapshot(
    exchange_creds: Vec<ExCreds>,
) -> HashMap<String, HashMap<String, HashMap<String, MarketData>>> {
    let exchanges = get_exchanges(exchange_creds);
    let mut ex_data: HashMap<String, HashMap<String, HashMap<String, MarketData>>> = HashMap::new();
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

fn get_exchanges(exchange_creds: Vec<ExCreds>) -> Vec<ExApi> {
    let mut exchanges = Vec::new();
    for creds in exchange_creds {
        match creds.ex_type {
            ExType::Poloniex => {
                let poloniex_creds = PoloniexCreds::new(&creds.name, &creds.key, &creds.secret);
                let poloniex_ex = ExApi::Poloniex(PoloniexApi::new(poloniex_creds).unwrap());
                exchanges.push(poloniex_ex);
            }
        }
    }
    return exchanges;
}

fn store_snapshot(
    db_client: mongodb::Client,
    exchange_creds: Vec<ExCreds>,
) -> HashMap<String, HashMap<String, HashMap<String, MarketData>>> {
    let snapshot = get_snapshot(exchange_creds);
    let date = Local::now();
    let time : f64 = date.timestamp() as f64;
    let coin_db = db_client.db("coins");
    let snap_clone = snapshot.clone();
    for (_, ex_data) in snap_clone {
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
                    .ok()
                    .expect("Failed to insert document.");
            }
        }
    }
    return snapshot;
}
fn time_to_seconds(time_str_ : String) -> i64 {
    let mut time_str = time_str_.clone();
    let mut factor : i64 = 0;
    if time_str.ends_with("s") {
        factor = 1;
    } else if time_str.ends_with("m") {
        factor = 60;
    } else if time_str.ends_with("h") {
        factor = 360;
    }
    time_str.pop();
    let time_val : i64 = time_str.parse().unwrap();
    return time_val*factor;
}

// returns map(market, map(time, market_data))
fn fetch_relative_range(
    db_client: mongodb::Client,
    relative_time : String,
    coin : String) {

    let date = Local::now();
    let time : i64 = date.timestamp();


    let min_val = time - time_to_seconds(relative_time);
    let max_val = time;
    let doc = doc! { "value" => { "$gt" => min_val, "$lte" => max_val } };
    
    let coin_db = db_client.db("coins");
    let coll = coin_db.collection(&coin);
    let mut cursor = coll.find(Some(doc.clone()), None)
        .ok().expect("Failed to execute find.");
    while let Some(Ok(result)) = cursor.next() {
        let market = result.get_str("market").unwrap();
        let time = result.get_str("time").unwrap();

        let market_data = MarketData {
            last: result.get_f64("last").unwrap(),
            lowest_ask: result.get_f64("lowest_ask").unwrap(),
            highest_bid: result.get_f64("highest_bid").unwrap(),
            percent_change: result.get_f64("percent_change").unwrap(),
            base_volume: result.get_f64("base_volume").unwrap(),
            quote_volume: result.get_f64("quote_volume").unwrap(),
            is_frozen: result.get_f64("is_frozen").unwrap() as u64,
            high24hr: result.get_f64("high24hr").unwrap(),
            low24hr: result.get_f64("low24hr").unwrap(),
        };
    }
}

fn evaluate_variables() {

}

fn help() -> String {
    let mut file = rsFile::open("./conf/usage.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    return contents;
}

fn handle_expression(
    exp: String,
    snapshot: HashMap<String, HashMap<String, HashMap<String, MarketData>>>,
) -> Vec<(String, f64)> {
    let mut result_vec = Vec::new();
    let mut exp_parser = parser::Parser::new(String::from(exp).to_uppercase()).unwrap();
    let vars = exp_parser.vars();
    // get coin values from vars (may be coin.property)
    let mut coin_vars = HashMap::new();
    let copy_vars = vars.clone();

    // Created marked coin variable objects for later binding
    // Marks property to get value from and if
    for v in copy_vars {

        let mut v_clone = v.clone();

        let v_clone = v.clone();
        let var_split: Vec<&str> = v_clone.split(".").collect();
        println!("{:?}", var_split);
        let coin = String::from(var_split[0]).clone();
        let this_coin_vec = coin_vars.entry(coin.clone()).or_insert(Vec::new());
        if var_split.len() == 1 {
            this_coin_vec.push(MarketProperty::new(v.clone(), coin, String::from("last")));
        } else if var_split.len() == 2 {
            println!("Found two!");
            this_coin_vec.push(MarketProperty::new(
                v.clone(),
                coin,
                String::from(var_split[1]),
            ));
        }
        // store trans metadata before pushing
    }
    for (_, exchange_data) in &snapshot {
        let mut coin_vals = HashMap::new();
        let mut market_sets = Vec::new();
        for (coin, coin_data) in exchange_data {
            if coin_vars.contains_key(coin) {
                println!("Coin was var: {}", coin);
                coin_vals.insert(coin.clone(), coin_data);
                let mut my_markets = HashSet::new();
                for (market, _) in coin_data {
                    my_markets.insert(market);
                }
                // translation happens here!
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
        for market in valid_markets {
            for (coin, coin_datum) in &coin_vals {
                let data = coin_datum.get(market).unwrap();
                for coin_var in &coin_vars[coin] {
                    exp_parser.bind(coin_var.var(), coin_var.val(data));
                }
            }
            let result = exp_parser.eval();
            result_vec.push((market.clone(), result));
        }
    }
    return result_vec;
}

fn handle_expressions(content_string: String, ex_creds: Vec<ExCreds>, channel_id: ChannelId) {
    let snapshot = get_snapshot(ex_creds.clone());
    let split = content_string.split(" ");
    let mut i = 0;
    let mut result_message = String::new();
    for exp in split {
        // individual expression handling
        let results = handle_expression(String::from(exp), snapshot.clone());
        if i > 0 { result_message += "\n"; }
        result_message += &format!("{}:", exp.to_uppercase());
        for (market, result) in results {
            result_message += &format!("\n\t{}: {}", market, result);
        }
        i+=1;
    }
    if let Err(why) = channel_id.say(result_message) {
        println!("Error sending message: {:?}", why);
    }
}

#[derive(Debug, Clone)]
struct Watch {
    channel_id: ChannelId,
    expression: String,
    values: Vec<(String, f64)>,
    threshold: f64,
    author: String,
}
struct WatchList;
impl Key for WatchList {
    type Value = HashMap<String, Vec<Watch>>;
}
// Discord API handler and helpers
#[derive(Debug)]
struct Handler {
    name: String,
    ex_creds: Vec<ExCreds>,
    // watches: Vec<Watch>, //db_client : Client
}
impl Handler {
    pub fn new(name: String, _db_client: Client, ex_creds: Vec<ExCreds>) -> Handler {
        return Handler {
            name: name,
            ex_creds: ex_creds,
            // watches: Vec::new(), //db_client: db_client.clone()
        };
    }
    pub fn list_watches(&self, context: Context) -> Vec<Watch> {
        let mut data = context.data.lock();
        let watch_list_cont = data.get_mut::<WatchList>().unwrap();
        let entry = watch_list_cont
            .entry(String::from("watches"))
            .or_insert(Vec::new());
        return entry.clone();
    }
    pub fn remove_watch(&self, context: Context, to_remove : usize) {
        let mut data = context.data.lock();
        let watch_list_cont = data.get_mut::<WatchList>().unwrap();
        let entry = watch_list_cont
            .entry(String::from("watches"))
            .or_insert(Vec::new());
        entry.remove(to_remove);
    }
    pub fn set_watch(&self, context: Context, e: String, t: String, a: String, c: ChannelId) {
        let snapshot = get_snapshot(self.ex_creds.clone());
        let v = handle_expression(e.clone(), snapshot);
        {
            let mut data = context.data.lock();
            let watch_list_cont = data.get_mut::<WatchList>().unwrap();
            let entry = watch_list_cont
                .entry(String::from("watches"))
                .or_insert(Vec::new());
            entry.push(Watch {
                channel_id: c,
                expression: e,
                threshold: t.parse().unwrap(),
                author: a,
                values: v,
            });
        }
    }
}
impl EventHandler for Handler {
    fn on_message(&self, context: Context, msg: serenity::model::Message) {
        let content_str: &str = &msg.content;
        let mut content_string: String = msg.content.clone();
        if content_str.starts_with("$") {
            content_string.drain(..1);
            handle_expressions(content_string, self.ex_creds.clone(), msg.channel_id);
        } else if content_str.starts_with("!") {
            content_string.drain(..1);

            let split: Vec<&str> = content_string.split(" ").collect();
            if split[0] == "help" || split[0] == "h" {
                let result_message = help();
                if let Err(why) = msg.channel_id.say(result_message) {
                    println!("Error sending message: {:?}", why);
                }
            } else if split[0] == "watch" || split[0] == "w" {
                self.set_watch(
                    context,
                    String::from(split[1]),
                    String::from(split[2]),
                    msg.author.mention(),
                    msg.channel_id,
                );
                if let Err(why) = msg.channel_id.say(format!(
                    "Watched expression {} armed over {}%.",
                    split[1].to_uppercase(),
                    split[2]
                )) {
                    println!("Error sending message: {:?}", why);
                }
            } else if split[0] == "ls" || split[0] == "l" || split[0] == "list" {
                    let mut result_message = String::from("Current active watches:");
                    let mut i = 0;
                    let watches = self.list_watches(context);
                    for w in watches {
                        result_message += &format!("\n[{}] {} {}%", i, w.expression, w.threshold);
                        i += 1;
                    }
                if let Err(why) = msg.channel_id.say(result_message
                ) {
                    println!("Error sending message: {:?}", why);
                }
            } else if split[0] == "rm" || split[0] == "remove" || split[0] == "r" {
                let to_remove = split[1].parse().unwrap();
                let watches = self.list_watches(context.clone());
                self.remove_watch(context, to_remove);
                let result_message = format!("Removed watch: {} {}%", watches[to_remove].expression.to_uppercase(), watches[to_remove].threshold);
                if let Err(why) = msg.channel_id.say(result_message
                ) {
                    println!("Error sending message: {:?}", why);
                }
            } else if split[0] == "graph" || split[0] == "g" {
                let to_graph : String = split[1].parse().unwrap();
                let x = [0u32, 1, 2];
                let y = [3u32, 4, 5];
                let mut fg = Figure::new();
                fg.axes2d()
                .lines(&x, &y, &[Caption("A line"), Color("black")]);

                let path : String = format!("./data/graph_{}.png", msg.id.0);
                println!("{}", path);

                let paths = vec![path.as_str()];

                fg.set_terminal("pngcairo", paths[0].clone());
                fg.show();
                if let Err(why) = msg.channel_id.send_files(paths, |m| m.content("Your Graph")) {
                    println!("Error sending message: {:?}", why);
                }
            }
            /*
                !! = show all values
                !last (same as $)
                !lowest_ask exp...
                !highest_bid exp...
                
            */
        }
    }

    fn on_ready(&self, context: Context, _ready: Ready) {
        let _res = context.edit_profile(|profile| profile.username(&self.name));
    }
}

fn check_watches(
    watch_list_cont: &mut HashMap<String, Vec<Watch>>,
    snapshot: HashMap<String, HashMap<String, HashMap<String, MarketData>>>,
) -> Vec<(bool, Vec<(String, f64)>)> {
    let mut result: Vec<(bool, Vec<(String, f64)>)> = Vec::new();
    let mut watches = watch_list_cont
        .entry(String::from("watches"))
        .or_insert(Vec::new())
        .clone();
    let mut i = 0;
    let all_watches = watches.clone();
    for w in all_watches {
        let cur_vals = handle_expression(w.expression.clone(), snapshot.clone());
        let old_vals = w.values.clone();
        let mut changed = false;
        let mut new_string = String::new();
        let mut old_string = String::new();
        let my_cur_vals = cur_vals.clone();
        for (market, val) in my_cur_vals {
            let my_olds = old_vals.clone();
            for (old_market, old_val) in my_olds {
                if market == old_market {
                    if (((val - old_val) / old_val) * 100.0).abs() > w.threshold.abs() {
                        changed = true;
                    }
                }
                if changed {
                    break;
                }
            }
            if changed {
                break;
            }
        }
        if changed {
            let changed_cur_vals = cur_vals.clone();
            for (market, val) in changed_cur_vals {
                new_string += &format!("\n\t\t{}: {}", market, val);
            }
            for (market, val) in old_vals {
                old_string += &format!("\n\t\t{}: {}", market, val);
            }
            let _ = w.channel_id.say(format!(
                "{} your {}% watched expression {} has triggered:\n\tOld:{}\n\tNew:{}",
                w.author,
                w.threshold,
                w.expression.to_uppercase(),
                old_string,
                new_string
            ));
            watches[i].values = cur_vals.clone();
        }
        result.push((changed, cur_vals.clone()));

        i += 1;
    }
    return result;
    //     channel_id: ChannelId,
    // expression: String,
    // values: Vec<(String, f64)>,
    // threshold: f64,
    // author: String,
}
// Dipper
fn main() {
    // Load settings file with api keys
    let mut settings_raw = config::Config::default();
    settings_raw
        .merge(File::with_name("conf/dipper.toml"))
        .unwrap();
    let settings = settings_raw
        .deserialize::<HashMap<String, HashMap<String, String>>>()
        .unwrap();

    // Create DB Client
    let db_client = Client::connect(
        &settings["database"]["url"],
        settings["database"]["port"].parse().unwrap_or(27017),
    ).expect("Failed to initialize standalone client.");

    // Populate cred struct for Poloniex
    let polo_cred_data = ExCreds {
        ex_type: ExType::Poloniex,
        name: settings["poloniex"]["name"].clone(),
        key: settings["poloniex"]["api_key"].clone(),
        secret: settings["poloniex"]["api_secret"].clone(),
    };
    let exchange_creds = vec![polo_cred_data];

    // Initilize 'Photographer' thread for caputring snapshots
    let autoshot_exchange_creds = exchange_creds.clone();
    let autoshot_db_client = db_client.clone();
    let snapshot_frequency = time::Duration::from_millis(60000); // time::Duration::from_secs(60);


    // Create & start Discord Client
    let discord_exchange_creds = exchange_creds.clone();
    let discord_db_client = db_client.clone();
    let handler = Handler::new(
        String::from("Dipper"),
        discord_db_client,
        discord_exchange_creds,
    );
    let mut client = SerenityClient::new(&settings["discord"]["token"], handler);
    {
        let mut data = client.data.lock();
        data.insert::<WatchList>(HashMap::default());
    }
    let context_data = client.data.clone();

    thread::spawn(move || {
        loop {
            {
                let snapshot =
                    store_snapshot(autoshot_db_client.clone(), autoshot_exchange_creds.clone());

                let mut data = context_data.lock();
                let watch_list_cont = data.get_mut::<WatchList>().unwrap();
                let mut watches = watch_list_cont
                    .entry(String::from("watches"))
                    .or_insert(Vec::new())
                    .clone();
                let cur_result = check_watches(watch_list_cont, snapshot);
                for i in 0..watches.len() {
                    if cur_result[i].0 {
                        let cur = &cur_result[i].1;
                        watches[i].values = cur.clone();
                    }
                }
                watch_list_cont.insert(String::from("watches"), watches);
            }
            // println!("Data: {:?}", entry);
            thread::sleep(snapshot_frequency);
        }
    });
    let _ = client.start();
}
