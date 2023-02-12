pub mod tcp_request {
    use serde_json::{Map, Value};
    use serde::{Serialize};

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
    use serde::{Deserialize};

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
    use serde::{Deserialize};

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
    use serde::{Serialize};

    #[derive(Serialize)]
    #[serde(tag = "command")]
    pub enum Response {
        PCPReturnStatus(PCPReturnStatus),
        PCPBookResult(PCPBookResult),
    }

    #[derive(Serialize)]
    pub struct PCPReturnStatus {
        pub room: String,
        pub status: Map<String,Value>,
    }

    #[derive(Serialize)]
    pub struct PCPBookResult {
        pub room: String,
        pub result: String,
    }
}