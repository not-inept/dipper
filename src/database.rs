extern crate bson;
extern crate mongodb;

use bson::Bson;
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

// General Utilities
pub enum Error {
    Mongodb(mongodb::Error)
}

// Generic Database Wrapper
enum DatabaseClient {
    Mongodb(mongodb::Client)
}

pub struct DatabaseConnInfo {
    url : String, port : u16
}

impl DatabaseConnInfo {
    pub fn new(url : String, port : u16) -> DatabaseConnInfo {
        DatabaseConnInfo {
            url: url,
            port: port
        }
    }
}
pub enum DatabaseType {
    Mongodb
}

trait GenericDatabaseClient : Sized {
    fn new(conn_info : DatabaseConnInfo) -> Result<Self,  Error>;
    fn select(&self);
    fn insert(&self);
}

enum GenericDatabase {
    Mongodb(Mongodb)
}

pub struct Database {
    db_type : DatabaseType,
    db : GenericDatabase,
}

impl Database {
        pub fn new(db_type : DatabaseType, conn_info : DatabaseConnInfo) -> Result<Database, Error> {
        match db_type {
            DatabaseType::Mongodb => {
                match Mongodb::new(conn_info) {
                    Ok(db) => {
                        Ok((Database {
                            db_type: db_type,
                            db: GenericDatabase::Mongodb(db)
                        }))
                    },
                    Err(e) => Err(e)
                }
            }
        }
    }
}

// Mongodb Generic Database Object
struct Mongodb {
    client : DatabaseClient,
    conn_info : DatabaseConnInfo
}

impl GenericDatabaseClient for Mongodb {
    fn new(conn_info : DatabaseConnInfo) ->  Result<Mongodb, Error> {
        match Client::connect(
            &conn_info.url,
            conn_info.port
        ) {
            Err(e) => Err(Error::Mongodb(e)),
            Ok(c) => Ok((Mongodb {
                client: DatabaseClient::Mongodb(c),
                conn_info: conn_info
            }))
        }
    }
    fn select(&self) {

    }
    fn insert(&self) {

    }
}

// fn main() {
//     let conn_info = DatabaseConnInfo {
//         url: String::from("localhost"),
//         port: 27017
//     };

//     let client = Mongodb::new(conn_info);
//     let client = Client::connect("localhost", 27017)
//         .expect("Failed to initialize standalone client.");

//     let coll = client.db("test").collection("movies");

//     let doc = doc! {
//         "title": "Jaws",
//         "array": [ 1, 2, 3 ],
//     };

//     // Insert document into 'test.movies' collection
//     coll.insert_one(doc.clone(), None)
//         .ok().expect("Failed to insert document.");

//     // Find the document and receive a cursor
//     let mut cursor = coll.find(Some(doc.clone()), None)
//         .ok().expect("Failed to execute find.");

//     let item = cursor.next();

//     // cursor.next() returns an Option<Result<Document>>
//     match item {
//         Some(Ok(doc)) => match doc.get("title") {
//             Some(&Bson::String(ref title)) => println!("{}", title),
//             _ => panic!("Expected title to be a string!"),
//         },
//         Some(Err(_)) => panic!("Failed to get next from server!"),
//         None => panic!("Server returned no results!"),
//     }
// }
