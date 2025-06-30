pub mod tunnel {
    tonic::include_proto!("tunnel");
}

pub mod http_forward;
pub mod response; 
pub mod proxy;

use tunnel::StreamType;

impl StreamType {
    pub fn from_rule_type(s: &str) -> StreamType {
        match s.to_ascii_lowercase().as_str() {
            "http" => StreamType::Http,
            "grpc" => StreamType::Grpc,
            "control" => StreamType::Control,
            "websocket" => StreamType::Websocket,
            _ => StreamType::Unspecified,
        }
    }
    pub fn as_rule_type(&self) -> &'static str {
        match self {
            StreamType::Http => "http",
            StreamType::Grpc => "grpc",
            StreamType::Control => "control",
            StreamType::Websocket => "websocket",
            _ => "unspecified",
        }
    }
}