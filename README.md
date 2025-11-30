# Tunnel

> **架构文档**: 详见 `BEST_PRACTICE_DESIGN.md` - 最佳实践方案

# Tunnel: High-Performance Multi-Protocol Dynamic Tunnel Gateway

## Project Overview
Tunnel is a high-performance tunnel gateway system supporting HTTP multi-port, multi-protocol, group-based dynamic forwarding. It is suitable for scenarios such as multi-cloud, hybrid cloud, intranet and extranet penetration, and microservice gateways. This project currently implements HTTP bidirectional functionality only and is intended for learning purposes.

---

## Architecture and Port Description

| Role    | Port   | Protocol | Purpose/Usage                     | Visitor         |
|---------|--------|----------|-----------------------------------|-----------------|
| client  |   Any  | gRPC     | Tunnel connection to server       | Connects to server |
| client  | 8001   | HTTP     | Local listener for HTTP requests  | External users  |
| client  | 8002   | gRPC     | Local listener for gRPC requests  | External users  |
| server  | 50001  | gRPC     | Tunnel entry receiving client connections | Receives from client |
| server  | 8001   | HTTP     | Local listener for HTTP requests  | External users  |
| server  | 8002   | gRPC     | Local listener for gRPC requests  | External users  |

---

## Core Architecture and Flow Logic

- **Unified Configuration Center**: The server manages all forwarding rules, upstreams, and client groups, supporting hot updates and dynamic distribution.
- **Group Management**: Clients register their group upon startup and periodically fetch exclusive rules from the server. Multiple groups are supported concurrently.
- **Bidirectional Forwarding**: Currently, only HTTP bidirectional functionality is implemented. gRPC functionality is not yet supported.
- **Load Balancing and Circuit Breaking**: The server can configure multiple client groups, supporting multi-protocol load balancing, health checks, and automatic removal.
- **High-Performance Data Channel**: Tunnel message format uses protobuf for efficient multi-protocol forwarding.
- **Configuration Hot Updates**: Server-side rules and upstreams support hot updates, and clients automatically fetch and apply updates.

---

## Typical Scenarios

- External users access server:8001/api, and the server forwards the request to the target client group based on rules. The client processes the request and returns the response.
- External users access client:8001, and the client forwards the request to the server through the tunnel. The server processes the request based on reverse_proxy.rules.
- Multiple groups are supported concurrently, and clients fetch exclusive rules based on their group, adapting flexibly to multi-tenant and multi-environment scenarios.

---

## Configuration Examples

### Server-Side Configuration (Nginx Style)

```toml
[server]
config_version = "v1.0.0"
tunnel_port = 50001
http_entry_port = 8001
grpc_entry_port = 8002
log_level = "info"
trace_enabled = false

[forward.upstreams.backend]
servers = [
    { address = "https://echo.free.beeceptor.com", resolve = true }
]
lb_policy = "round_robin"

[[forward.rules.http]]
match_host = "echo.free.beeceptor.com"
action_upstream = "backend"

[[reverse_proxy.rules.http]]
match_host = "www.google.com"
action_client_group = "group-a"

[reverse_proxy.client_groups.group-a]
config_version = "v1.0.0"

[reverse_proxy.client_groups.group-a.upstreams.local_service]
servers = [
    { address = "https://echo.free.beeceptor.com", resolve = true }
]
lb_policy = "round_robin"

[[reverse_proxy.client_groups.group-a.rules.http]]
match_host = "www.google.com"
action_upstream = "local_service"
```

### Client-Side Group Rules Configuration

```toml
[client]
group_id = "group-a"
server_addr = "tunnel-server.example.com:50001"

[client.local_services]
http_port = 8001
grpc_port = 8002

[client_groups.group-a.rules]

[[client_groups.group-a.rules.http_rules]]
match_host = "www.google.com"
action_upstream = "local_service"

[[client_groups.group-a.rules.grpc_rules]]
match_service = "com.example.Special"
action_upstream = "local_service"
```

---

## Performance Optimization and Architectural Highlights

- Concurrency Optimization: Use DashMap/RwLock for global state to reduce lock contention.
- Efficient Data Forwarding: Use tokio::io::copy_bidirectional and buffer pool for memory reuse.
- Trait Abstraction: Core logic is abstracted into traits for easy extension and mock testing.
- Health Checks and Automatic Removal: Periodic heartbeat checks automatically remove disconnected clients/streams.
- Automatic Reconnection: Implement exponential backoff for reconnection.
- Metrics/Tracing: Add key path metrics and structured logs for distributed tracing.
- Configuration Hot Updates: Use tokio::sync::watch/notify for configuration file hot loading.

---

## TODO
- [ ] Support dynamic registration and health checks for multiple client groups
- [ ] Implement server-side hot updates for forwarding rules
- [ ] Support full-field forwarding for HTTP/gRPC with header/metadata retention
- [ ] Implement multi-protocol load balancing and circuit breaking
- [ ] Provide detailed configuration samples and management tools

---

## Summary

Tunnel achieves "Nginx-level flexible forwarding + dynamic distributed tunnel + unified configuration center," suitable for complex scenarios such as multi-cloud, hybrid cloud, intranet and extranet penetration, and microservice gateways. The architecture is clearly layered, with excellent performance, supporting multi-protocol, multi-group, dynamic configuration, and high availability. Currently, only HTTP bidirectional functionality is implemented, and the project is intended for learning purposes.