use bytes::{BufMut, BytesMut};

fn main() {
    let mut dst = BytesMut::with_capacity(1024);
    {
        let mut chunk = (&mut dst).limit(10);
        chunk.put_slice(b"hello");
    }
    println!("dst len: {}", dst.len());
}
