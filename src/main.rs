#[macro_use] extern crate rocket;
use rocket::serde::json::Json;
use std::{io, fs};
use std::io::{Write,BufRead};
use std::net::TcpStream;
use serde_json;
use serde_json::{Map, Value};
use serde::{Serialize, Deserialize};
use hibernatx_backend::{tcp_request::TCPRequest, tcp_return::TCPReturn, client_request::Request, client_response::Response};
use hibernatx_backend::{client_request, client_response, tcp_request};
use hibernatx_backend::json_error::{self, JsonError};

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

    let tcp_request = TCPRequest::get( tcp_request::Get { nodes: String::from("*") });
    let request_json = Json(serde_json::to_string(&tcp_request)?);
    let return_json: TCPReturn = serde_json::from_str(&request_node(&get_address(&request.room)?, request_json)?)?;

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
    let tcp_request = TCPRequest::set(tcp_request::Set { nodes: node_map });
    let request_json = Json(serde_json::to_string(&tcp_request)?);
    let return_json: TCPReturn = serde_json::from_str(&request_node(&get_address(&request.room)?, request_json)?)?;

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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![rec_json])
}

