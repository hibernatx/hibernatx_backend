#[macro_use] extern crate rocket;
use rocket::serde::json::Json;
use std::{io, fs, env};
use std::io::{Write,BufRead};
use std::net::TcpStream;
use serde_json;
use serde_json::{Map, Value};
use hibernatx_backend::{tcp_request::TCPRequest, tcp_return::TCPReturn, client_request::Request, client_response::Response};
use hibernatx_backend::{client_request, client_response, tcp_request, tcp_return};
use hibernatx_backend::json_error::{self, JsonError};

use rocket::http::Header;
use rocket::fairing::{Fairing, Info, Kind};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r rocket::Request<'_>, response: &mut rocket::Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello world!"
}

#[options("/")]
fn opts() -> &'static str {
    "Hello world!"
}

#[post("/", data = "<request_json>")]
fn rec_json<'r>(request_json: String) -> Json<String> {
    let request: Request = match serde_json::from_str(&request_json) {
        Ok(json) => json,
        Err(e) => return JsonError::new(&e.to_string()).to_json(),
    };

    match request {
        Request::PCPGetStatus(request_status) => {
            //send request to node
            match get_status(request_status) {
                Ok(json) => json,
                Err(e) => return e.to_json(),
            }
        },
        Request::PCPBookPC(request_pc) => {
            //send request to node
            match book_pc(request_pc) {
                Ok(json) => json,
                Err(e) => return e.to_json(),
            }
        },
    }
}

fn get_status(request: client_request::PCPGetStatus) -> json_error::Result<Json<String>> {
    //let request_data: Request::PCPGetStatus = serde_json::from_value(request.data)?;

    let request_tcp = TCPRequest::Get( tcp_request::Get { nodes: String::from("*") });
    let request_json = Json(serde_json::to_string(&request_tcp)?);
    let address = match get_address(&request.room) {
        Ok(addr) => addr,
        Err(_) =>  {
            let return_status = Response::PCPReturnStatus(client_response::PCPReturnStatus { room: request.room, result: String::from("not_found"), status: Map::new() });
            return Ok(Json(serde_json::to_string(&return_status)?));
        },
    };
    let return_string = match request_node(&address, request_json) {
        Ok(string) => string,
        Err(_) => {
            let return_status = Response::PCPReturnStatus(client_response::PCPReturnStatus { room: request.room, result: String::from("node_communication_failed"), status: Map::new() });
            return Ok(Json(serde_json::to_string(&return_status)?));
        },
    };
    let return_json: TCPReturn = serde_json::from_str(&return_string)?;

    let return_status = match return_json {
        TCPReturn::NodeList(node_list) => Response::PCPReturnStatus(client_response::PCPReturnStatus { room: node_list.device_id, result: String::from("success"), status: node_list.nodes }),
        TCPReturn::Status(status) => return Err(JsonError::new(status.status.as_str())),
    };

    // TODO : Sanity check device_id against
    let return_status_json = Json(serde_json::to_string(&return_status)?);
    
    Ok(return_status_json)
}

fn book_pc(request: client_request::PCPBookPC) -> json_error::Result<Json<String>> {    
    // TODO : Check if PC already booked
    let mut node_map = Map::new();
    node_map.insert(String::from(&request.pc), serde_json::to_value(String::from("on"))?);
    let request_tcp = TCPRequest::Set(tcp_request::Set { nodes: node_map });
    let request_json = Json(serde_json::to_string(&request_tcp)?);
    let address = match get_address(&request.room) {
        Ok(addr) => addr,
        Err(_) =>  {
            let return_status = Response::PCPBookResult(client_response::PCPBookResult { room: request.room, result: String::from("not_found") });
            return Ok(Json(serde_json::to_string(&return_status)?));
        },
    };
    let return_string = match request_node(&address, request_json) {
        Ok(string) => string,
        Err(_) => {
            let return_status = Response::PCPBookResult(client_response::PCPBookResult { room: request.room, result: String::from("node_communication_failed") });
            return Ok(Json(serde_json::to_string(&return_status)?));
        },
    };
    let return_json: TCPReturn = serde_json::from_str(&return_string)?;

    let return_status = match return_json {
        TCPReturn::NodeList(mut node_list) => match node_list.nodes.remove(&request.pc) {
            Some(state) => match serde_json::from_value::<String>(state)?.as_str() {
                "on" => Response::PCPBookResult(client_response::PCPBookResult { room: node_list.device_id, result: String::from("success") }),
                "off" => Response::PCPBookResult(client_response::PCPBookResult { room: node_list.device_id, result: String::from("node_error") }),
                "already_on" => Response::PCPBookResult(client_response::PCPBookResult { room: node_list.device_id, result: String::from("already_booked") }),
                _ => Response::PCPBookResult(client_response::PCPBookResult { room: node_list.device_id, result: String::from("not_found") }),
            },
            _ => Response::PCPBookResult(client_response::PCPBookResult { room: node_list.device_id, result: String::from("not_found") }),
        }
        TCPReturn::Status(status) => Response::PCPBookResult(client_response::PCPBookResult { room: status.device_id, result: String::from("not_found") }),
    };

    // TODO : Generate result return type from data by checking what return is

    let return_status_json = Json(serde_json::to_string(&return_status)?);
    
    Ok(return_status_json)
}

fn request_node(addr: &str, request_json: Json<String>) -> json_error::Result<String> {
    let mut stream = TcpStream::connect(addr)?;
    println!("Writing {} to python server", &request_json.to_string());
    stream.write(&request_json.to_string().as_bytes())?;

    let mut reader = io::BufReader::new(stream.try_clone()?);
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    println!("Recieved {} from python server", &buf);

    Ok(buf)
}

fn get_address(device_id: &str) -> json_error::Result<String> {

    let contents = fs::read_to_string("address_table.json")?;
    let mut address_map: Map<String,Value> = serde_json::from_str(&contents)?;

    let address = match address_map.remove(device_id) {
        Some(addr) => addr,
        None => return Err(JsonError::new("Room not found in address lookup table")),
    };
    let address: String = serde_json::from_value(address)?;
    println!("{}", &address);

    Ok(address)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let args: Vec<String> = env::args().collect();
    if let Some(arg) = args.get(1) {
        if arg == "setup" {
            setup();
            return Ok(());
        }
    }
    let _rocket = rocket::build()
        .attach(CORS)
        .mount("/", routes![rec_json])
        .mount("/", routes![index])
        .mount("/", routes![opts])
        .launch()
        .await?;
    Ok(())
}

fn setup() {
    loop {
        let mut line = String::new();
        print!("> ");
        io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();
        let mut spl_line = line.trim().split(" ");
        match spl_line.next() {
            Some("exit") => return,
            Some("add") => {
                let device_id = String::from(match spl_line.next() {
                    Some(val) => val,
                    None => { println!("Expected node ID"); continue; },
                });
                let node_id = String::from(match spl_line.next() {
                    Some(val) => val,
                    None => { println!("Expected pc ID"); continue; },
                });
                let hostname = String::from(match spl_line.next() {
                    Some(val) => val,
                    None => { println!("Expected hostname"); continue; },
                });
                let mac_address = String::from(match spl_line.next() {
                    Some(val) => val,
                    None => { println!("Expected mac address"); continue; },
                });
                match add_node(device_id, node_id, hostname, mac_address) {
                    Err(e) => println!("Error: {}", e.to_string()),
                    Ok(()) => println!("Node sucessfully added"),
                };
            },
            Some("update") => {
                let device_id = String::from(match spl_line.next() {
                    Some(val) => val,
                    None => { println!("Expected node ID"); continue; },
                });
                let node_id = String::from(match spl_line.next() {
                    Some(val) => val,
                    None => { println!("Expected pc ID"); continue; },
                });
                let hostname = String::from(match spl_line.next() {
                    Some(val) => val,
                    None => { println!("Expected hostname"); continue; },
                });
                let mac_address = String::from(match spl_line.next() {
                    Some(val) => val,
                    None => { println!("Expected mac address"); continue; },
                });
                match update_node(device_id, node_id, hostname, mac_address) {
                    Err(e) => println!("Error: {}", e.to_string()),
                    Ok(()) => println!("Node sucessfully updated"),
                };
            },
            Some("remove") => {
                let device_id = String::from(match spl_line.next() {
                    Some(val) => val,
                    None => { println!("Expected node ID"); continue; },
                });
                let node_id = String::from(match spl_line.next() {
                    Some(val) => val,
                    None => { println!("Expected pc ID"); continue; },
                });
                match remove_node(device_id, node_id) {
                    Err(e) => println!("Error: {}", e.to_string()),
                    Ok(()) => println!("Node sucessfully removed"),
                };
            },
            _ => println!("Command not recognised"),
        };
    }
}

fn add_node(device_id: String, node_id: String, hostname: String, mac_address: String) -> json_error::Result<()>{
    let request_tcp = TCPRequest::AddNode(tcp_request::AddNode { node_id, hostname, mac_address });
    let request_json = Json(serde_json::to_string(&request_tcp)?);
    let return_json: tcp_return::Status = serde_json::from_str(&request_node(&get_address(&device_id)?, request_json)?)?;
    match return_json.status.as_str() {
        "ok" => Ok(()),
        error => Err(JsonError::new(error)),
    }
}

fn update_node(device_id: String, node_id: String, hostname: String, mac_address: String) -> json_error::Result<()>{
    let request_tcp = TCPRequest::UpdateNode(tcp_request::UpdateNode { node_id, hostname, mac_address });
    let request_json = Json(serde_json::to_string(&request_tcp)?);
    let return_json: tcp_return::Status = serde_json::from_str(&request_node(&get_address(&device_id)?, request_json)?)?;
    match return_json.status.as_str() {
        "ok" => Ok(()),
        error => Err(JsonError::new(error)),
    }
}

fn remove_node(device_id: String, node_id: String) -> json_error::Result<()>{
    let request_tcp = TCPRequest::RemoveNode(tcp_request::RemoveNode { node_id });
    let request_json = Json(serde_json::to_string(&request_tcp)?);
    let return_json: tcp_return::Status = serde_json::from_str(&request_node(&get_address(&device_id)?, request_json)?)?;
    match return_json.status.as_str() {
        "ok" => Ok(()),
        error => Err(JsonError::new(error)),
    }
}