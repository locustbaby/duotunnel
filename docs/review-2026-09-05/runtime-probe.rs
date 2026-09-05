use duotunnel_lib::{VhostRouter, canonicalize_egress_host};
fn main() {
    let authority = "[::1]:8080";
    let route_host = authority.split(':').next().unwrap_or(authority).to_ascii_lowercase();
    let router = VhostRouter::new();
    router.add_route("[::1]", "ipv6-backend");
    println!("H2c IPv6: raw={authority}, current_key={route_host:?}, current_match={:?}, canonical_match={:?}", router.get(&route_host), router.get(&canonicalize_egress_host(authority).unwrap()));
    let dir = std::env::temp_dir().join(format!("duotunnel-review-pki-{}", std::process::id()));
    std::fs::create_dir(&dir).expect("probe requires a fresh isolated directory");
    let cert = dir.join("ca.crt");
    let key = dir.join("ca.key");
    std::fs::write(&cert, "corrupt-existing-certificate").unwrap();
    std::fs::write(&key, "corrupt-existing-key").unwrap();
    let params = duotunnel_lib::PkiParams {
        ca_cert_path: Some(cert.to_string_lossy().into_owned()),
        ca_key_path: Some(key.to_string_lossy().into_owned()),
        ..Default::default()
    };
    duotunnel_lib::infra::pki::init_cert_cache(&params);
    println!("Existing invalid CA overwritten: cert={}, key={}", std::fs::read_to_string(&cert).unwrap().contains("BEGIN CERTIFICATE"), std::fs::read_to_string(&key).unwrap().contains("BEGIN PRIVATE KEY"));
    std::fs::remove_dir_all(&dir).unwrap();
}
