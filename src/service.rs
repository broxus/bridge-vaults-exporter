use std::collections::{HashMap, HashSet};
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
    token_decimals: String,
}

impl Service {
    pub async fn new(networks: Vec<NetworkVaults>) -> Result<Self> {
        let mut listeners = Vec::with_capacity(networks.len());

        let ctx = Arc::new(InitializationContext::default());

        let mut futures = FuturesUnordered::new();
        for network in networks {
            futures.push(Listener::new(ctx.clone(), network));
        }

        while let Some(listener) = futures.next().await {
            listeners.push(listener?);
        }

        let token_decimals = ctx.prepare_decimals_info(&listeners);

        Ok(Self {
            listeners,
            token_decimals,
        })
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
        Metrics {
            listeners: &self.listeners,
            token_decimals: &self.token_decimals,
        }
    }
}

struct Listener {
    chain_id: u32,
    vaults: Vec<Arc<VaultListener>>,
}

impl Listener {
    pub async fn new(ctx: Arc<InitializationContext>, config: NetworkVaults) -> Result<Arc<Self>> {
        let api = Api::new(config.endpoint.as_str())
            .await
            .context("Failed to initialize api")?;

        let mut vaults = Vec::with_capacity(config.vaults.len());

        let mut futures = FuturesUnordered::new();
        for vault in config.vaults {
            ctx.add_vault(api.chain_id, vault.address)?;
            futures.push(VaultListener::new(ctx.clone(), api.clone(), vault));
        }

        while let Some(vault) = futures.next().await {
            vaults.push(vault?)
        }

        Ok(Arc::new(Self {
            chain_id: api.chain_id,
            vaults,
        }))
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
    async fn new(
        ctx: Arc<InitializationContext>,
        api: Api,
        vault: VaultsEntry,
    ) -> Result<Arc<Self>> {
        let token = api.get_vault_token(vault.address).await?;
        let token_info = api.get_token_info(token).await?;

        if let Some(group) = vault.group {
            ctx.add_token_group(api.chain_id, token, group)?;
        }

        log::info!(
            "Created listener for vault {:x} ({} / {})",
            vault.address,
            token_info.symbol,
            token_info.decimals
        );

        Ok(Arc::new(VaultListener {
            listening: AtomicBool::new(false),
            api,
            vault: vault.address,
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
        let balance = self.api.get_vault_balance(self.token, self.vault).await?;
        let total_assets = self.api.get_vault_total_assets(self.vault).await?;

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

#[derive(Default)]
struct InitializationContext {
    /// Set of unique vaults (chain id + vault address)
    unique_vaults: parking_lot::Mutex<HashSet<(u32, ethabi::Address)>>,
    /// Map of token groups (chain id + token address => group)
    token_groups: parking_lot::Mutex<HashMap<(u32, ethabi::Address), String>>,
}

impl InitializationContext {
    fn add_vault(&self, chain_id: u32, vault: ethabi::Address) -> Result<()> {
        if self.unique_vaults.lock().insert((chain_id, vault)) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Duplicate vault entry"))
        }
    }

    fn add_token_group(&self, chain_id: u32, token: ethabi::Address, group: String) -> Result<()> {
        use std::collections::hash_map::Entry;

        let mut token_group = self.token_groups.lock();
        match token_group.entry((chain_id, token)) {
            Entry::Vacant(entry) => {
                entry.insert(group);
                Ok(())
            }
            Entry::Occupied(entry) if *entry.get() == group => Ok(()),
            Entry::Occupied(_) => Err(anyhow::anyhow!("Inconsistent token group")),
        }
    }

    fn prepare_decimals_info(&self, listeners: &[Arc<Listener>]) -> String {
        TokenDecimals {
            listeners,
            groups: &*self.token_groups.lock(),
        }
        .to_string()
    }
}

type EthHttpApi = web3::api::Eth<web3::transports::Http>;

#[derive(Clone)]
struct Api {
    chain_id: u32,
    api: EthHttpApi,
}

impl Api {
    async fn new(endpoint: &str) -> Result<Self> {
        let transport =
            web3::transports::Http::new(endpoint).context("Failed to create http transport")?;
        let api = EthHttpApi::new(transport);
        let chain_id = api
            .chain_id()
            .await
            .context("Failed to get chain id")?
            .as_u32();

        Ok(Api { chain_id, api })
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
            .call(token, contracts::erc_20::symbol(), &[])
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
            .api
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

struct Metrics<'a> {
    listeners: &'a [Arc<Listener>],
    token_decimals: &'a str,
}

impl std::fmt::Display for Metrics<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.token_decimals)?;

        for listener in self.listeners {
            for vault in &listener.vaults {
                let state = vault.state.read();
                if state.updated_at == 0 {
                    continue;
                }

                f.begin_metric("balance")
                    .label(LABEL_CHAIN_ID, listener.chain_id)
                    .label(LABEL_VAULT, FullAddress(&vault.vault))
                    .label(LABEL_TOKEN, FullAddress(&vault.token))
                    .value(PrintedNum(&state.balance))?;

                f.begin_metric("total_assets")
                    .label(LABEL_CHAIN_ID, listener.chain_id)
                    .label(LABEL_VAULT, FullAddress(&vault.vault))
                    .label(LABEL_TOKEN, FullAddress(&vault.token))
                    .value(PrintedNum(&state.total_assets))?;

                f.begin_metric("updated_at")
                    .label(LABEL_CHAIN_ID, listener.chain_id)
                    .label(LABEL_VAULT, FullAddress(&vault.vault))
                    .value(state.updated_at)?;
            }
        }

        Ok(())
    }
}

struct TokenDecimals<'a> {
    listeners: &'a [Arc<Listener>],
    groups: &'a HashMap<(u32, ethabi::Address), String>,
}

impl std::fmt::Display for TokenDecimals<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        struct TokensEntry<'a> {
            info: &'a TokenInfo,
            group: Option<&'a String>,
        }

        let mut tokens = HashMap::<(u32, ethabi::Address), TokensEntry>::new();
        for listener in self.listeners {
            for vault in &listener.vaults {
                tokens.insert(
                    (listener.chain_id, vault.token),
                    TokensEntry {
                        info: &vault.token_info,
                        group: self.groups.get(&(listener.chain_id, vault.token)),
                    },
                );
            }
        }

        for ((chain_id, token), TokensEntry { info, group }) in tokens {
            f.begin_metric("token_decimals")
                .label(LABEL_CHAIN_ID, chain_id)
                .label(LABEL_TOKEN, FullAddress(&token))
                .label_opt(LABEL_TOKEN_GROUP, group)
                .label(LABEL_SYMBOL, &info.symbol)
                .value(info.decimals)?;
        }

        Ok(())
    }
}

struct FullAddress<'a>(&'a ethabi::Address);

impl std::fmt::Display for FullAddress<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!("0x{:x}", self.0))
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

const LABEL_CHAIN_ID: &str = "chain_id";
const LABEL_VAULT: &str = "vault";
const LABEL_TOKEN: &str = "token";
const LABEL_TOKEN_GROUP: &str = "token_group";
const LABEL_SYMBOL: &str = "symbol";
