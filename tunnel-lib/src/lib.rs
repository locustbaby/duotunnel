pub mod quic_transport;
pub mod protocol;
pub mod hash;
pub mod frame;

// Include generated protobuf code
// prost generates code based on the package name in the proto file
// Since package is "tunnel", the generated code will be in proto::tunnel module
pub mod proto {
    pub mod tunnel {
        include!(concat!(env!("OUT_DIR"), "/tunnel.rs"));
    }
}