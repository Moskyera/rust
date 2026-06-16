# Hacash by Rust
Hacash Fullnode, PoWorker etc.

### Build and use documentation

- [Build_compilation.md](https://github.com/hacash/doc/blob/main/build/build_compilation.md)
- [Configuration description](https://github.com/hacash/doc/blob/main/build/config_description.md)
- [Fullnode API Doc](https://github.com/hacash/doc/blob/main/server/fullnode_api_doc.md)


### Project code engineering architecture

The architecture of Hacash full node code is divided into 7 levels from bottom to top:

X16RS -> Core -> Chain -> Mint -> Node -> Server -> Miner

Each layer of the architecture has independent functions and responsibilities for the upper layer to call, and the implementation of the lower layer to the upper layer is unknown. The responsibilities of each layer are roughly as follows:

1. [X16RS] Basic algorithm - including HAC mining, block diamond mining, GPU version algorithm, etc.
2. [Core] Core - block structure definition, interface definition, data serialization and deserialization, storage object, field format, genesis block definition, etc.
3. [Chain] Chain - underlying database, block and transaction storage, blockchain state storage, logs, etc.
4. [Mint] Mint - block mining difficulty adjustment algorithm, coinbase definition, block construction, transaction execution and status update, etc.
5. [Node] Node - P2P underlying module, Backend blockchain synchronization terminal, point-to-point network message definition and processing, etc.
6. [Server] Server - RPC API interface service, block and transaction and account data query, other services, etc.
7. [Miner] Miner - block construction and mining, diamond mining, transaction memory pool, mining pool server, mining pool worker, etc.


### HIP-25 HACD staking (community review)

**Branch:** [`hip-25-staking`](https://github.com/Moskyera/rust/tree/hip-25-staking) · **Release:** [`v0.1.0-hip25-mainnet`](https://github.com/Moskyera/rust/releases/tag/v0.1.0-hip25-mainnet) · **Audits:** 5/5 PASS @ `a798094`

| Mode | Launcher | Port | Notes |
|------|----------|------|-------|
| Testnet demo | `scripts\START_WALLET.bat` | 8083 | Fresh dev chain, poworker, wipes demo data |
| Mainnet wallet | `scripts\START_MAINNET_WALLET.bat` | 8081 | Release build, synced `data_dir`, no data wipe |

**Mainnet build (once):**
```powershell
powershell -ExecutionPolicy Bypass -File scripts\BUILD_MAINNET_RELEASE.ps1
```

**Community review / voting:** [docs/HIP25_COMMUNITY_REVIEW.md](docs/HIP25_COMMUNITY_REVIEW.md)  
**Upstream PR (copy-paste body):** [docs/UPSTREAM_PR.md](docs/UPSTREAM_PR.md)  
**Open PR to official repo:** https://github.com/hacash/rust/compare/main...Moskyera:rust:hip-25-staking?expand=1

Config template: `hacash_mainnet_hip25.config.ini.example` (`chain_id=0`, loopback RPC, client WASM signing only).

