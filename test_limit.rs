use bytes::{BufMut, BytesMut};

fn main() {
    let mut dst = BytesMut::with_capacity(1024);
    let mut chunk = dst.limit(10);
}
