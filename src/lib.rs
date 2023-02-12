pub mod tcp_request {
    use serde_json::{Map, Value};
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(tag = "action_type")]
    pub enum TCPRequest {
        get(Get),
        set(Set),
        add_node(AddNode),
        update_node(UpdateNode),
        remove_node(RemoveNode),
    }

    #[derive(Serialize)]
    pub struct Get {
        pub nodes: String,
    }

    #[derive(Serialize)]
    pub struct Set {
        pub nodes: Map<String,Value>,
    }

    #[derive(Serialize)]
    pub struct AddNode {
        pub node_id: String,
        pub hostname: String,
        pub mac_address: String,
    }

    #[derive(Serialize)]
    pub struct UpdateNode {
        pub node_id: String,
        pub hostname: String,
        pub mac_address: String,
    }

    #[derive(Serialize)]
    pub struct RemoveNode {
        pub node_id: String,
    }
}

pub mod tcp_return {
    use serde_json::{Map, Value};
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(untagged)]
    pub enum TCPReturn {
        NodeList(NodeList),
        Status(Status),
    }

    #[derive(Deserialize)]
    pub struct NodeList {
        pub device_id: String,
        pub nodes: Map<String,Value>,
    }

    #[derive(Deserialize)]
    pub struct Status {
        pub device_id: String,
        pub status: String,
    }
}

pub mod client_request {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(tag = "command")]
    pub enum Request {
        PCPGetStatus(PCPGetStatus),
        PCPBookPC(PCPBookPC),
    }

    #[derive(Deserialize)]
    pub struct PCPGetStatus {
        pub room: String,
    }

    #[derive(Deserialize)]
    pub struct PCPBookPC {
        pub room: String,
        pub pc: String,
    }
}

pub mod client_response {
    use serde_json::{Map, Value};
    use serde::Serialize;

    #[derive(Serialize)]
    #[serde(tag = "command")]
    pub enum Response {
        PCPReturnStatus(PCPReturnStatus),
        PCPBookResult(PCPBookResult),
    }

    #[derive(Serialize)]
    pub struct PCPReturnStatus {
        pub room: String,
        pub result: String,
        pub status: Map<String,Value>,
    }

    #[derive(Serialize)]
    pub struct PCPBookResult {
        pub room: String,
        pub result: String,
    }
}

pub mod json_error {
    use serde::Serialize;
    use rocket::serde::json::Json;

    #[derive(Serialize)]
    pub struct JsonError {
        pub error: String,
    }

    pub type Result<T> = std::result::Result<T, JsonError>;

    impl JsonError {
        pub fn new(err_text: &str) -> JsonError{
            JsonError { error: String::from(err_text) }
        }

        pub fn to_json(&self) -> Json<String> {
            let err_string = match serde_json::to_string(self) {
                Ok(err) => err,
                Err(_) => String::from(r#"{"error": "error parsing error"}"#),
            };
            Json(err_string)
        }

        pub fn to_string(&self) -> String {
            String::from(&self.error)
        }
    }

    impl From<std::io::Error> for JsonError {
        fn from(error: std::io::Error) -> JsonError {
            JsonError::new(&error.to_string())
        }
    }

    impl From<serde_json::Error> for JsonError {
        fn from(error: serde_json::Error) -> JsonError {
            JsonError::new(&error.to_string())
        }
    }
}