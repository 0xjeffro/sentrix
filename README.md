<p align="center">
<img src="https://cdn3.iconfinder.com/data/icons/minecraft-icons/512/enderman.png" alt="tx-parser" width="80">
</p>
<h1 align="center">SENTRIX</h1>

Sentrix (from Sentinel + Metrics) acts as a high-performance reverse proxy that verifies HMAC-signed tokens, 
forwards JSON-RPC requests, and emits rich traceable logs compatible with tools like [Vector](https://vector.dev/) and [Better Stack](https://betterstack.com/).

## 🚀 Features
- ⚡️ **High-performance RPC forwarding** Built with axum + async Rust, optimized for low-latency relay.
- 🔒 **HMAC Token Verification** Supports HMAC token verification for secure communication.
- 📊 **Observability** Capture request payloads, backend latency, error status, response body, etc.
- 🛠️ **Customizable** Easily extendable with custom handlers and middleware.
- 📦 **Ready for production** TOML-configurable, integrates seamlessly with modern observability pipelines.

## 🔧 Configuration
Sentrix uses a `default.toml` file. Example:

```toml
[app]
name = "sentrix"                   # Application name (used for identification and logging)
port = 8080                        # The port on which the gateway listens for incoming requests
secret_key = ""  # HMAC secret key used to verify signed tokens (Base64-encoded)

[backend]
rpc_url = ""  # Target JSON-RPC endpoint for forwarding requests
yellowstone_grpc_url = ""    # Optional Yellowstone gRPC endpoint
yellowstone_grpc_token = ""  # Optional token used to authorize gRPC requests

[http_client]
pool_max_idle_per_host = 32       # Maximum number of idle connections kept alive per host
timeout_secs = 10                 # Total timeout for outbound HTTP requests (in seconds)
connect_timeout_secs = 3          # Timeout for establishing TCP connection (in seconds)
pool_idle_timeout_secs = 90       # Duration to keep idle connections in the pool (in seconds)

[log]
file = "/var/log/sentrix.log"     # Path to the log file.
level = "info"                    # Log verbosity level: one of "error", "warn", "info", "debug", or "trace"
```

## 🔑 Token Format
Sentrix uses custom base64-encoded JSON tokens with an embedded HMAC signature.
Decoded structure:
```json
{
  "user": "jeffro",
  "exp": 1744690570,
  "qps": 100,
  "sig": "<HMAC_SHA256 signature>"
}
```
Token is passed via URL parameter:
`POST /?token=eyJ1c2VyIjoiamVmZnJvIiwiZXhwIjo...`

## ✨ Coming Soon
- Yellowstone gRPC integration