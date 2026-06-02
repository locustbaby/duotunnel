use quinn::RecvStream;
use bytes::BytesMut;

async fn test(mut recv: RecvStream, mut dst: BytesMut) {
    let chunk = recv.read_chunk(1024, true).await.unwrap();
}
fn main() {}
