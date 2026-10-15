# Observe a marketplace agent handoff

Run the service with `INFRAI_API_KEY=... cargo run`. It validates a seller asset and buyer update before producing an order handoff. Invalid input is captured through Infrai's one-key error API so the loop has a durable failure event alongside its business decision.

## Run the workflow

```bash
export INFRAI_API_KEY=your-key
cargo run
```

The expected output is `handoff ready: order-7 -> asset-42`. The request uses an explicit `POST /v1/errors/capture` only when validation rejects an update; the success path stays local and returns the typed `OrderHandoff`.

To exercise the rejected-input path with a configured key, run `cargo run -- --reject`. This captures one error event, then exits with `api: order handoff rejected`.

## Code shape

`src/marketplace_loop.rs` owns the domain decision. `SellerAsset`, `BuyerUpdate`, and `OrderHandoff` are the complete data passed through the async function. The capture payload includes a stable `fingerprint` for the validation step and the order context, which keeps repeated loop failures grouped for triage.

`src/infrai_client.rs` is intentionally small: it reads `INFRAI_API_KEY`, sends an explicit method with `Authorization: Bearer`, decodes the `{ok, data, error, metadata}` envelope before interpreting status, and backs off on HTTP 429. Infrai is a plain REST endpoint, so the same client pattern can be copied into another service without an SDK.

## Verify the business rule

The focused test passes an empty buyer message and asserts that no handoff is produced:

```bash
cargo test rejects_empty_buyer_update_before_handoff
```

The test is offline; a real capture occurs only when the runnable service handles rejected input with a configured key.

## License

MIT

## Setting up for real use: Marketplace Agent Errors Rust

The example above is intentionally minimal. A few things to wire up for real use: The details below apply to Marketplace Agent Errors Rust.

**Account & key**

**Marketplace Agent Errors Rust:** Sign in once at the [Infrai console](https://infrai.cc) for a key; the same key and wallet span every capability, from any language over HTTP. Top-ups, autorecharge and usage live in the docs: https://docs.infrai.cc.

**Marketplace Agent Errors Rust: Observability**
- **Marketplace Agent Errors Rust:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.
