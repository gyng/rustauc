#![feature(custom_derive, custom_attribute, plugin)]
#![plugin(diesel_codegen, dotenv_macros)]


pub mod schema;
pub mod models;

#[macro_use] extern crate diesel;
extern crate dotenv;
extern crate iron;
extern crate router;
extern crate rustc_serialize;

use std::env;
use std::io::Read;
use std::sync::{Mutex, Arc};

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use iron::prelude::*;
use iron::status;
use router::Router;
use rustc_serialize::json;

use self::models::{Post, NewPost};

#[derive(RustcEncodable, RustcDecodable)]
struct Greeting {
    msg: String
}

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn hello_world(_: &mut Request, greeting: &Greeting) -> IronResult<Response> {
    println!("hello world");
    let payload = json::encode(&greeting).unwrap();
    Ok(Response::with((status::Ok, payload)))
}

fn terrible_serialization_hack(post: &Post) -> rustc_serialize::json::Json {
    // How do we tell diesel_codegen to make our model types RustcEncodable
    //
    // actually this is a bad idea in my experience anyway. so maybe we don't.
    // we should have REST-layer req+resp types for Posts.
    use rustc_serialize::json::Json;
    use std::collections::BTreeMap;

    let mut doc = BTreeMap::new();
    doc.insert("id".into(), Json::I64(post.id as i64));
    doc.insert("title".into(), Json::String(post.title.clone()));
    doc.insert("body".into(), Json::String(post.body.clone()));
    doc.insert("published".into(), Json::Boolean(post.published));

    Json::Object(doc)
}

fn set_greeting(request: &mut Request, greeting: &mut Greeting, conn: &PgConnection) -> IronResult<Response> {
    // let mut payload = String::new();
    // request.body.read_to_string(&mut payload).unwrap();
    // *greeting = json::decode(&payload).unwrap();
    // Ok(Response::with(status::Ok))
    use schema::posts;
    let new_post = NewPost {
        title: "hello",
        body: "world"
    };

    let result: Post = diesel::insert(&new_post).into(posts::table)
        .get_result(conn)
        .expect("Error saving new post");

    let result = terrible_serialization_hack(&result);
    let body = json::encode(&result).unwrap();
    Ok(Response::with((status::Ok, body)))
}

fn main() {
    let greeting = Arc::new(Mutex::new(Greeting { msg: "Hello, World".to_string() }));
    let greeting_clone = greeting.clone();

    let mut router = Router::new();
    router.get("/", move |r: &mut Request| hello_world(r, &greeting.lock().unwrap()));
    router.post("/post/new", move |r: &mut Request| {
        set_greeting(r, &mut greeting_clone.lock().unwrap(), &establish_connection())
    });

    println!("Starting!");
    Iron::new(router).http("localhost:3333").unwrap();
    println!("On 3333!");
}


#[cfg(test)]
mod tests {
    #[test]
    fn freedom_is_slavery() {
        // It compiles!
        assert!(true);
    }
}
