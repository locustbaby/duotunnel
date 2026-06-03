use rkyv::util::AlignedVec;
fn main() {
    let mut buf = AlignedVec::<16>::new();
    buf.extend_from_slice(&vec![0; 10]);
}
