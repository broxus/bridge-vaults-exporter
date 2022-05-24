## Bridge Vaults Exporter

### How to run

```bash
git clone https://github.com/broxus/bridge-vaults-exporter
cd bridge-vaults-exporter
cargo build --release
target/release/bridge-vaults-exporter --config config.yaml
```

### Example output

```
token_decimals{chain_id="1",token="0x6b175474e89094c44da98b954eedeac495271d0f",token_group="DAI",symbol="DAI"} 18
token_decimals{chain_id="250",token="0x8d11ec38a3eb5e956b052f67da8bdc9bef8abf3e",token_group="DAI",symbol="DAI"} 18
token_decimals{chain_id="137",token="0x8f3cf7ad23cd3cadbd9735aff958023239c6a063",token_group="DAI",symbol="DAI"} 18
token_decimals{chain_id="56",token="0x1af3f329e8be154074d8769d1ffa4ee058b1dbc3",token_group="DAI",symbol="DAI"} 18
relay_round{bridge_proxy="0xf4404070f63a7e19be0b1dd89a5fb88e12c0173a"} 2
relay_count{bridge_proxy="0xf4404070f63a7e19be0b1dd89a5fb88e12c0173a"} 22
balance{chain_id="56",vault="0xad4c25634e3818d674ddc07b98135ed6db7ef307",token="0x1af3f329e8be154074d8769d1ffa4ee058b1dbc3"} 11454597652850199020763
total_assets{chain_id="56",vault="0xad4c25634e3818d674ddc07b98135ed6db7ef307",token="0x1af3f329e8be154074d8769d1ffa4ee058b1dbc3"} 11454597652850199020763
withdraw_limit_per_period{chain_id="56",vault="0xad4c25634e3818d674ddc07b98135ed6db7ef307",token="0x1af3f329e8be154074d8769d1ffa4ee058b1dbc3"} 300000000000000000000000
withdrawal_period_total{chain_id="56",vault="0xad4c25634e3818d674ddc07b98135ed6db7ef307",token="0x1af3f329e8be154074d8769d1ffa4ee058b1dbc3",withdrawal_period="19136"} 15000000000000000000000
withdrawal_period_considered{chain_id="56",vault="0xad4c25634e3818d674ddc07b98135ed6db7ef307",token="0x1af3f329e8be154074d8769d1ffa4ee058b1dbc3",withdrawal_period="19136"} 0
updated_at{chain_id="56",vault="0xad4c25634e3818d674ddc07b98135ed6db7ef307"} 1646086133
balance{chain_id="250",vault="0x334d7e33f3b0ac04309b17ca56bcb0f0fa3d0efd",token="0x8d11ec38a3eb5e956b052f67da8bdc9bef8abf3e"} 15775831476917052039837
total_assets{chain_id="250",vault="0x334d7e33f3b0ac04309b17ca56bcb0f0fa3d0efd",token="0x8d11ec38a3eb5e956b052f67da8bdc9bef8abf3e"} 15775831476917052039837
withdraw_limit_per_period{chain_id="250",vault="0x334d7e33f3b0ac04309b17ca56bcb0f0fa3d0efd",token="0x8d11ec38a3eb5e956b052f67da8bdc9bef8abf3e"} 300000000000000000000000
withdrawal_period_total{chain_id="250",vault="0x334d7e33f3b0ac04309b17ca56bcb0f0fa3d0efd",token="0x8d11ec38a3eb5e956b052f67da8bdc9bef8abf3e",withdrawal_period="19136"} 0
withdrawal_period_considered{chain_id="250",vault="0x334d7e33f3b0ac04309b17ca56bcb0f0fa3d0efd",token="0x8d11ec38a3eb5e956b052f67da8bdc9bef8abf3e",withdrawal_period="19136"} 0
updated_at{chain_id="250",vault="0x334d7e33f3b0ac04309b17ca56bcb0f0fa3d0efd"} 1646086104
balance{chain_id="137",vault="0xced734f47613e2484fd9ee6f76afcb866bc4d6fa",token="0x8f3cf7ad23cd3cadbd9735aff958023239c6a063"} 2502623258636882209434
total_assets{chain_id="137",vault="0xced734f47613e2484fd9ee6f76afcb866bc4d6fa",token="0x8f3cf7ad23cd3cadbd9735aff958023239c6a063"} 2502623258636882209434
withdraw_limit_per_period{chain_id="137",vault="0xced734f47613e2484fd9ee6f76afcb866bc4d6fa",token="0x8f3cf7ad23cd3cadbd9735aff958023239c6a063"} 300000000000000000000000
withdrawal_period_total{chain_id="137",vault="0xced734f47613e2484fd9ee6f76afcb866bc4d6fa",token="0x8f3cf7ad23cd3cadbd9735aff958023239c6a063",withdrawal_period="19136"} 0
withdrawal_period_considered{chain_id="137",vault="0xced734f47613e2484fd9ee6f76afcb866bc4d6fa",token="0x8f3cf7ad23cd3cadbd9735aff958023239c6a063",withdrawal_period="19136"} 0
updated_at{chain_id="137",vault="0xced734f47613e2484fd9ee6f76afcb866bc4d6fa"} 1646086104
balance{chain_id="1",vault="0x032d06b4cc8a914b85615acd0131c3e0a7330968",token="0x6b175474e89094c44da98b954eedeac495271d0f"} 346192603472053121587099
total_assets{chain_id="1",vault="0x032d06b4cc8a914b85615acd0131c3e0a7330968",token="0x6b175474e89094c44da98b954eedeac495271d0f"} 346192603472053121587099
withdraw_limit_per_period{chain_id="1",vault="0x032d06b4cc8a914b85615acd0131c3e0a7330968",token="0x6b175474e89094c44da98b954eedeac495271d0f"} 300000000000000000000000
withdrawal_period_total{chain_id="1",vault="0x032d06b4cc8a914b85615acd0131c3e0a7330968",token="0x6b175474e89094c44da98b954eedeac495271d0f",withdrawal_period="19136"} 0
withdrawal_period_considered{chain_id="1",vault="0x032d06b4cc8a914b85615acd0131c3e0a7330968",token="0x6b175474e89094c44da98b954eedeac495271d0f",withdrawal_period="19136"} 0
updated_at{chain_id="1",vault="0x032d06b4cc8a914b85615acd0131c3e0a7330968"} 1646086104
```

> Exported metrics:
> - `token_decimals` - token decimals (unique for each token in each each network)
> - `relay_round` - current relay round
> - `relay_count` - relay count in current round
> - `balance` - current token balance which is available for withdrawal.
> - `total_assets` - total token balance, including funds which are locked in some strategies.
> - `withdraw_limit_per_period` - maximum amount of tokens which can be withdrawn in one withdrawal period (1 day)
> - `withdrawal_period_total` - total amount of tokens which were withdrawn in current withdrawal period
> - `withdrawal_period_considered` - total amount of tokens which were approved for withdrawal in current withdrawal period
> - `updated_at` - timestamp of the last update

### Example config

> NOTE: The syntax `${VAR}` can also be used everywhere in config. It will be
> replaced by the value of the environment variable `VAR`.

```yaml
---
networks:
  # Ethereum
  - endpoint: https://mainnet.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161
    bridge_proxy: 0xF4404070f63a7E19Be0b1dd89A5fb88E12c0173A
    vaults:
      - group: DAI
        address: 0x032d06b4cc8a914b85615acd0131c3e0a7330968
      - group: USDT
        address: 0x81598d5362eac63310e5719315497c5b8980c579
      - group: USDC
        address: 0xf8a0d53ddc6c92c3c59824f380c0f3d2a3cf521c
      - group: WBTC
        address: 0xf67d8b970a0a955b5ff2a80b8dfd6aff21567633
      - group: WETH
        address: 0x55046f53eb9fa069286969d73432b769f068e1fc
      - group: UNI-V2
        address: 0x8d589f403d5232e37bd30e02260ea6b6ad061f3f

  # Polygon
  - endpoint: https://rpc-mainnet.matic.quiknode.pro
    vaults:
      - group: DAI
        address: 0xced734f47613e2484fd9ee6f76afcb866bc4d6fa
      - group: USDT
        address: 0xd33492080f2d3a89ae500a3b3bc0e076713a3cbb
      - group: USDC
        address: 0xf504e9a7511f1af03f8e8c6800a05fb9d43481f2
      - group: WBTC
        address: 0x1fa28c9cb44d2853afd0d932c3805221fab95a8b
      - group: WETH
        address: 0x356b37e007564fd37b957f946a246871bf827ea2

  # Fantom
  - endpoint: https://rpc.ftm.tools
    vaults:
      - group: DAI
        address: 0x334d7e33f3b0ac04309b17ca56bcb0f0fa3d0efd
      - group: USDC
        address: 0xb05a3640132642e6297980376129354ee21a9fc6
      - group: WBTC
        address: 0x8f9d8cfd0b018b1939bb24e2ce48a9e4040fb68a
      - group: WETH
        address: 0x5115cd7e0dd0886c11e28e54ad2422f61544f314

  # BNB
  - endpoint: https://bsc-dataseed.binance.org
    vaults:
      - group: DAI
        address: 0xad4c25634e3818d674ddc07b98135ed6db7ef307
      - group: USDT
        address: 0x5d767d4e250b5c8640cb2bf7e7cd3acaeb7768e1
      - group: USDC
        address: 0x65950dd2a3d8316c197bda1a353aed046035b1c9
      - group: WBTC
        address: 0x0cc7096480e78409aec14795a96efeaf3e0b4b38
      - group: WETH
        address: 0x5b1e3e9f24455debd6f3a0c4b8bc6b46ca57f68c

metrics_settings:
  # Listen address of metrics. Used by the client to gather prometheus metrics.
  # Default: "127.0.0.1:10000"
  listen_address: "127.0.0.1:10000"
  # URL path to the metrics. Default: "/"
  # Example: `curl http://127.0.0.1:10000/`
  metrics_path: "/"
  # Metrics update interval in seconds. Default: 10
  collection_interval_sec: 30

# log4rs settings.
# See https://docs.rs/log4rs/1.0.0/log4rs/ for more details
logger_settings:
  appenders:
    stdout:
      kind: console
      encoder:
        pattern: "{h({l})} {M} = {m} {n}"
  root:
    level: error
    appenders:
      - stdout
  loggers:
    bridge_vaults_exporter:
      level: info
      appenders:
        - stdout
      additive: false
```
