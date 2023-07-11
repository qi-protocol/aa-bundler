use aa_bundler_contracts::entry_point::EntryPointAPI;
use aa_bundler_primitives::{Chain, UserOperation, Wallet};
use ethers::{
    prelude::{SignerMiddleware, LocalWallet},
    providers::{Http, Middleware, Provider},
    signers::Signer,
    types::{
        transaction::eip2718::TypedTransaction, 
        Address, 
        H256
    },
};
use ethers_flashbots::{BundleRequest, FlashbotsMiddleware, PendingBundleError::BundleNotIncluded};
use std::{sync::Arc, time::Duration};
use tracing::{info, trace};
use url::Url;
use tokio::task::JoinHandle;
use std::env;

const RELAY_ENDPOINTS: &[(&str, &str)] = &[
    ("flashbots", "https://relay.flashbots.net"),
    ("flashbots_goerli", "https://relay-goerli.flashbots.net"),
    ("builder0x69", "http://builder0x69.io/"),
    ("edennetwork", "https://api.edennetwork.io/v1/bundle"),
    ("beaverbuild", "https://rpc.beaverbuild.org/"),
    ("lightspeedbuilder", "https://rpc.lightspeedbuilder.info/"),
    ("eth-builder", "https://eth-builder.com/"),
    ("ultrasound", "https://relay.ultrasound.money/"),
    ("agnostic-relay", "https://agnostic-relay.net/"),
    ("relayoor-wtf", "https://relayooor.wtf/"),
    ("rsync-builder", "https://rsync-builder.xyz/"),
];


#[derive(Clone)]
pub struct Bundler {
    pub wallet: Wallet,
    pub eth_client_address: String,
    pub beneficiary: Address,
    pub entry_point: Address,
    pub chain: Chain,
}

impl Bundler {
    pub fn new(
        wallet: Wallet,
        eth_client_address: String,
        beneficiary: Address,
        entry_point: Address,
        chain: Chain,
    ) -> Self {
        Self {
            wallet,
            eth_client_address,
            beneficiary,
            entry_point,
            chain,
        }
    }

    pub async fn send_next_bundle(&self, uos: &Vec<UserOperation>) -> anyhow::Result<H256> {
        if uos.is_empty() {
            info!("Skipping creating a new bundle, no user operations");
            return Ok(H256::default());
        };

        info!("Creating a new bundle with {} user operations", uos.len());
        trace!("Bundle content: {uos:?}");

        let eth_client = Provider::<Http>::try_from(self.eth_client_address.clone())?;
        let client = Arc::new(SignerMiddleware::new(
            eth_client.clone(),
            self.wallet.signer.clone(),
        ));
        let ep = EntryPointAPI::new(self.entry_point, client.clone());

        let nonce = client
            .clone()
            .get_transaction_count(self.wallet.signer.address(), None)
            .await?;
        let mut tx: TypedTransaction = ep
            .handle_ops(
                uos.clone().into_iter().map(Into::into).collect(),
                self.beneficiary,
            )
            .tx
            .clone();
        tx.set_nonce(nonce).set_chain_id(self.chain.id());

        trace!("Sending transaction to the execution client: {tx:?}");

        let tx = client
            .send_transaction(tx, None)
            .await?
            .interval(Duration::from_millis(75));
        let tx_hash = tx.tx_hash();

        let tx_receipt = tx.await?;

        trace!("Transaction receipt: {tx_receipt:?}");

        Ok(tx_hash)
    }

    // TODO: add more relay endpoints support
    /// Send a bundle as Flashbots bundles
    #[allow(clippy::needless_return)]
    #[allow(clippy::clone_double_ref)]
    pub async fn send_next_bundle_flashbots(&self, uos: &Vec<UserOperation>, test: Option<bool>) -> anyhow::Result<H256> {

        let mut relay_endpoint: &str = RELAY_ENDPOINTS[0].1;
        if Some(true) == test {
            relay_endpoint = RELAY_ENDPOINTS[1].1; 
        };
        

        if uos.is_empty() {
            info!("Skipping creating a new bundle, no user operations");
            return Ok(H256::default());
        };

        info!("Creating a new bundle with {} user operations", uos.len());
        trace!("Bundle content: {uos:?}");

        let provider = Provider::<Http>::try_from(self.eth_provider_address.clone())?;

        let _bundle_signer = env::var("FLASHBOTS_IDENTIFIER").expect("FLASHBOTS_IDENTIFIER environment variable is not set");

        let bundle_signer = _bundle_signer.parse::<LocalWallet>()?;

        let mut fb_middleware = FlashbotsMiddleware::new(
                    provider.clone(),
                    Url::parse(relay_endpoint.clone())?,
                    bundle_signer.clone(),
                );
        fb_middleware.set_simulation_relay(Url::parse(relay_endpoint.clone()).unwrap());

        let client = Arc::new(
            SignerMiddleware::new(
                fb_middleware,
                self.wallet.signer.clone(),
            )
        );

        let ep = EntryPointAPI::new(self.entry_point, client.clone());

        let nonce = client
            .clone()
            .get_transaction_count(self.wallet.signer.address(), None)
            .await?;
        let mut tx: TypedTransaction = ep
            .handle_ops(
                uos.clone().into_iter().map(Into::into).collect(),
                self.beneficiary,
            )
            .tx
            .clone();
        tx.set_nonce(nonce).set_chain_id(self.chain.id());

        trace!("Sending transaction to the execution client: {tx:?}");

        // Sign the tx
        let typed_tx = TypedTransaction::Eip1559(tx.clone().into());
        let raw_signed_tx = match client.signer().sign_transaction(&typed_tx).await {
            Ok(tx) => typed_tx.rlp_signed(&tx),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to sign transaction: {:?}",
                    e
                ))
            }
        };

        // Add tx to Flashbots bundle
        let mut bundle_req = BundleRequest::new();
        bundle_req = bundle_req.push_transaction(raw_signed_tx);

        // Simulate the Flashbots bundle
        let block_num = client.get_block_number().await?;
        bundle_req = bundle_req
            .set_block(block_num + 1)
            .set_simulation_block(block_num)
            .set_simulation_timestamp(0);
        let simulated_bundle = client.inner().simulate_bundle(&bundle_req).await?;

        // Currently there's only 1 tx per bundle 
        for tx in simulated_bundle.transactions {
            trace!("Simulate bundle: {:?}", tx);
            if let Some(err) = &tx.error { 
                return Err(anyhow::anyhow!(
                    "Transaction failed simulation with error: {:?}",
                    err
                ));
            }
            if let Some(revert) = &tx.revert { 
                return Err(anyhow::anyhow!(
                    "Transaction failed simulation with revert: {:?}",
                    revert
                ));
            }
        };

        // Send the Flashbots bundle and check for status
        let handle: JoinHandle<Result<(bool, H256), anyhow::Error>> = tokio::spawn(
            async move {
                let pending_bundle = match client.inner().send_bundle(&bundle_req).await {
                    Ok(bundle) => bundle,
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "Failed to send bundle: {:?}",
                            e
                        ))
                    }
                };

                let bundle_hash = pending_bundle.bundle_hash;

                match pending_bundle.await {
                    Ok(_) => return Ok((true, bundle_hash)),
                    Err(BundleNotIncluded) => {
                        return Err(anyhow::anyhow!("Bundle not included in the target block"));
                    },
                    Err(e) => {
                        return Err(anyhow::anyhow!("Bundle rejected: {:?}", e));
                    }
                };
            }
        );

        match handle.await {
            Ok(Ok((_, bundle_hash))) => {
                info!("Bundle included");
                Ok(bundle_hash)
            },
            Ok(Err(e)) => Err(e),
            Err(e) => Err(anyhow::anyhow!(
                    "Task panicked: {:?}",
                    e
                )),
        }
    }
}
