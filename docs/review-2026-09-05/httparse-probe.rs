fn main() {
    for count in [64, 65, 128] {
        let request = format!("GET / HTTP/1.1\r\n{}\r\n", "X-Test: x\r\n".repeat(count));
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut parsed = httparse::Request::new(&mut headers);
        println!("{count} headers: {:?}", parsed.parse(request.as_bytes()));
    }
    println!("Header size: {}", std::mem::size_of::<httparse::Header<'_>>());
}
