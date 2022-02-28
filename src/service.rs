use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use pomfrit::formatter::*;
use web3::api::Namespace;
use web3::ethabi;

use crate::config::*;
use crate::contracts;
use crate::printed_num::*;

pub struct Service {
    listeners: Vec<Arc<Listener>>,
}

impl Service {
    pub async fn new(networks: Vec<NetworkVaults>) -> Result<Self> {
        let mut listeners = Vec::with_capacity(networks.len());

        let mut futures = FuturesUnordered::new();
        for network in networks {
            futures.push(Listener::new(network));
        }

        while let Some(listener) = futures.next().await {
            listeners.push(listener?);
        }

        Ok(Self { listeners })
    }

    pub async fn start_listening(&self, interval: Duration) -> Result<()> {
        let mut futures = FuturesUnordered::new();
        for listener in &self.listeners {
            for vault in &listener.vaults {
                futures.push(vault.start_listening(interval));
            }
        }

        while let Some(result) = futures.next().await {
            result.context("Failed to start listener")?
        }

        Ok(())
    }

    pub fn metrics(&'_ self) -> impl std::fmt::Display + '_ {
        Metrics(&self.listeners)
    }
}

struct Listener {
    chain_id: u32,
    vaults: Vec<Arc<VaultListener>>,
}

impl Listener {
    pub async fn new(config: NetworkVaults) -> Result<Arc<Self>> {
        let api = Api::new(config.endpoint.as_str())?;
        let chain_id = api.get_chain_id().await.context("Failed to get chain id")?;

        let mut vaults = Vec::with_capacity(config.vaults.len());

        let mut futures = FuturesUnordered::new();
        for vault in config.vaults {
            futures.push(VaultListener::new(api.clone(), vault));
        }

        while let Some(vault) = futures.next().await {
            vaults.push(vault?)
        }

        Ok(Arc::new(Self { chain_id, vaults }))
    }
}

struct VaultListener {
    listening: AtomicBool,
    api: Api,
    vault: ethabi::Address,
    token: ethabi::Address,
    token_info: TokenInfo,
    state: parking_lot::RwLock<VaultState>,
}

impl VaultListener {
    async fn new(api: Api, vault: ethabi::Address) -> Result<Arc<Self>> {
        let token = api.get_vault_token(vault.clone()).await?;
        let token_info = api.get_token_info(token.clone()).await?;

        log::info!(
            "Created listener for vault {:x} ({} / {})",
            vault,
            token_info.symbol,
            token_info.decimals
        );

        Ok(Arc::new(VaultListener {
            listening: AtomicBool::new(false),
            api,
            vault,
            token,
            token_info,
            state: Default::default(),
        }))
    }

    async fn start_listening(self: &Arc<Self>, interval: Duration) -> Result<()> {
        if self.listening.swap(true, Ordering::AcqRel) {
            return Ok(());
        }

        self.update().await?;

        log::info!(
            "Started listening {:x} ({} / {})",
            self.vault,
            self.token_info.symbol,
            self.token_info.decimals
        );

        let this = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;

                if let Err(e) = this.update().await {
                    log::error!("Failed to update vault balance {:x}: {}", this.vault, e);
                }
            }
        });

        Ok(())
    }

    async fn update(&self) -> Result<()> {
        let balance = self
            .api
            .get_vault_balance(self.token.clone(), self.vault.clone())
            .await?;
        let total_assets = self.api.get_vault_total_assets(self.vault.clone()).await?;

        let updated_at = now();

        *self.state.write() = VaultState {
            updated_at,
            balance: balance.to_string(),
            total_assets: total_assets.to_string(),
        };

        Ok(())
    }
}

#[derive(Default)]
struct VaultState {
    updated_at: u32,
    balance: String,
    total_assets: String,
}

type EthHttpApi = web3::api::Eth<web3::transports::Http>;

#[derive(Clone)]
struct Api(EthHttpApi);

impl Api {
    fn new(endpoint: &str) -> Result<Self> {
        let transport =
            web3::transports::Http::new(endpoint).context("Failed to create http transport")?;
        Ok(Api(EthHttpApi::new(transport)))
    }

    async fn get_chain_id(&self) -> Result<u32> {
        let chain_id = self.0.chain_id().await?;
        Ok(chain_id.as_u32())
    }

    async fn get_vault_token(&self, vault: ethabi::Address) -> Result<ethabi::Address> {
        match self
            .call(vault, contracts::vault::token(), &[])
            .await?
            .next()
        {
            Some(ethabi::Token::Address(address)) => Ok(address),
            _ => Err(ListenerError::InvalidOutput.into()),
        }
    }

    async fn get_token_info(&self, token: ethabi::Address) -> Result<TokenInfo> {
        let symbol = match self
            .call(token.clone(), contracts::erc_20::symbol(), &[])
            .await?
            .next()
        {
            Some(ethabi::Token::String(symbol)) => symbol,
            _ => return Err(ListenerError::InvalidOutput.into()),
        };

        let decimals = match self
            .call(token, contracts::erc_20::decimals(), &[])
            .await?
            .next()
        {
            Some(ethabi::Token::Uint(uint)) => uint.as_u32() as u8,
            _ => return Err(ListenerError::InvalidOutput.into()),
        };

        Ok(TokenInfo { symbol, decimals })
    }

    async fn get_vault_balance(
        &self,
        token: ethabi::Address,
        vault: ethabi::Address,
    ) -> Result<ethabi::Uint> {
        match self
            .call(
                token,
                contracts::erc_20::balance_of(),
                &[ethabi::Token::Address(vault)],
            )
            .await?
            .next()
        {
            Some(ethabi::Token::Uint(uint)) => Ok(uint),
            _ => Err(ListenerError::InvalidOutput.into()),
        }
    }

    async fn get_vault_total_assets(&self, vault: ethabi::Address) -> Result<ethabi::Uint> {
        match self
            .call(vault, contracts::vault::total_assets(), &[])
            .await?
            .next()
        {
            Some(ethabi::Token::Uint(uint)) => Ok(uint),
            _ => Err(ListenerError::InvalidOutput.into()),
        }
    }

    async fn call(
        &self,
        address: ethabi::Address,
        method: &ethabi::Function,
        tokens: &[ethabi::Token],
    ) -> Result<impl Iterator<Item = ethabi::Token>> {
        let data = method
            .encode_input(tokens)
            .with_context(|| format!("Failed to encode method input: {}", method.name))?;

        let output = self
            .0
            .call(
                web3::types::CallRequest {
                    to: Some(address),
                    data: Some(data.into()),
                    ..Default::default()
                },
                None,
            )
            .await
            .with_context(|| format!("Failed to execute call method: {}", method.name))?;

        Ok(method
            .decode_output(&output.0)
            .with_context(|| format!("Failed to decode method output: {}", method.name))?
            .into_iter())
    }
}

struct TokenInfo {
    symbol: String,
    decimals: u8,
}

struct Metrics<'a>(&'a [Arc<Listener>]);

impl std::fmt::Display for Metrics<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const LABEL_CHAIN_ID: &str = "chain_id";
        const LABEL_SYMBOL: &str = "symbol";
        const LABEL_DECIMALS: &str = "decimals";
        const LABEL_UPDATED_AT: &str = "updated_at";

        for listener in self.0 {
            for vault in &listener.vaults {
                let state = vault.state.read();
                if state.updated_at == 0 {
                    continue;
                }

                f.begin_metric("balance")
                    .label(LABEL_CHAIN_ID, listener.chain_id)
                    .label(LABEL_SYMBOL, &vault.token_info.symbol)
                    .label(LABEL_DECIMALS, &vault.token_info.decimals)
                    .label(LABEL_UPDATED_AT, &state.updated_at)
                    .value(PrintedNum(&state.balance))?;

                f.begin_metric("total_assets")
                    .label(LABEL_CHAIN_ID, listener.chain_id)
                    .label(LABEL_SYMBOL, &vault.token_info.symbol)
                    .label(LABEL_DECIMALS, &vault.token_info.decimals)
                    .label(LABEL_UPDATED_AT, &state.updated_at)
                    .value(PrintedNum(&state.total_assets))?;
            }
        }

        Ok(())
    }
}

fn now() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Shouldn't fail")
        .as_secs() as u32
}

#[derive(Debug, thiserror::Error)]
enum ListenerError {
    #[error("Invalid getter output")]
    InvalidOutput,
}
