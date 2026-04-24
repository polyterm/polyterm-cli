# polyterm

The Bloomberg terminal for Polymarket — a keyboard-native trading terminal for prediction markets.

Built in Rust. Real-time. Cross-platform.

> Early development. Read-only market browsing today; trading features underway.

## Install

No releases yet — build from source:

```bash
git clone https://github.com/polyterm/polyterm-cli
cd polyterm-cli
cargo build --release
./target/release/polyterm
```

Requires Rust 1.88+.

## Usage

Launch the TUI (default):

```bash
polyterm
```

Navigate: `j`/`k` or arrows · `Enter` to open · `Esc` to go back · `q` to quit.

Plain CLI output, for scripting:

```bash
polyterm markets --limit 10
```

## Roadmap

- [x] TUI menu + markets browser
- [x] Gamma API integration (read-only)
- [ ] Real-time orderbook via CLOB WebSocket
- [ ] Positions, fills, PnL view
- [ ] Order entry with keyboard shortcuts
- [ ] Wallet setup flow
- [ ] Homebrew / scoop / apt distribution

## Built on

[`polymarket-client-sdk`](https://crates.io/crates/polymarket-client-sdk) — the official Polymarket Rust SDK.

## Links

- Website: [polyterm.one](https://polyterm.one)
- X: [@PolyTermOne](https://x.com/PolyTermOne)
- Telegram: [@PolyTermOne](https://t.me/PolyTermOne)
