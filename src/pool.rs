#![allow(dead_code)]
#![allow(unused_variables)]

use ethers_middleware::SignerMiddleware;
use ethers_signers::LocalWallet;
use futures::lock::Mutex;
use std::sync::Arc;

use crate::{
    rollup::OutputResponse,
    L1Client,
    OutputOracle,
    OutputOracleContract,
};
use ethers_providers::Middleware;
use ethers_core::{types::{
    TransactionReceipt,
    H256, Address, U256, transaction::eip2718::TypedTransaction,
}, k256::ecdsa::SigningKey};
use eyre::Result;
use tokio::{
    sync::mpsc::UnboundedReceiver,
    task::JoinHandle,
};

/// TransactionPool
///
/// The transaction pool is responsible for managing and sending [crate::rollup::OutputResponse]
/// proposal transactions to the L1 network.
#[derive(Debug, Default)]
pub struct TransactionPool {
    /// The L1 client
    pub l1_client: Option<L1Client>,
    /// The output oracle
    pub output_oracle: Option<OutputOracle>,
    /// Transaction Construction Join Handles
    pub tx_construction_handles: Arc<Mutex<Vec<JoinHandle<Result<()>>>>>,
    /// The core transaction pool task
    pub core_task: Option<JoinHandle<Result<()>>>,
}

impl TransactionPool {
    /// Constructs a new [TransactionPool].
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts the transaction pool in a new thread.
    pub async fn start(
        &mut self,
        l1_client: Arc<Mutex<L1Client>>,
        private_key: H256,
        receiver: Option<UnboundedReceiver<OutputResponse>>,
    ) -> Result<()> {
        let handles = Arc::clone(&self.tx_construction_handles);
        let mut receiver =
            receiver.ok_or(eyre::eyre!("Missing output response receiver"))?;
        // Spawn transaction pool in a separate task
        let main_handle = tokio::task::spawn(async move {
            while let Some(proposal) = receiver.recv().await {
                let l1_client = Arc::clone(&l1_client);
                let sub_task = tokio::task::spawn(async move {
                    let tx = TransactionPool::construct_proposal_tx(
                        proposal,
                        Arc::clone(&l1_client),
                        &private_key,
                    )
                    .await?;
                    let receipt = TransactionPool::send_transaction(
                        Arc::clone(&l1_client),
                        &private_key,
                        tx,
                    )
                    .await?;
                    tracing::info!(target: "varro=pool", "Sent transaction: {:?}", receipt);
                    Ok(())
                });
                handles.lock().await.push(sub_task);
            }
            Ok(())
        });
        self.core_task = Some(main_handle);

        Ok(())
    }

    /// Constructs a [TypedTransaction] for a given [OutputResponse] proposal.
    ///
    /// Returns an [eyre::Result] containing the constructed [TypedTransaction].
    pub async fn construct_proposal_tx(
        proposal: OutputResponse,
        l1_client: Arc<Mutex<L1Client>>,
        private_key: &H256,
    ) -> Result<TypedTransaction> {
        tracing::debug!(target: "varro=pool", "Constructing proposal: {:?}", proposal);

        let unwrapped_client = l1_client.lock().await;
        let signing_key = SigningKey::from_bytes(&private_key.0)?;
        let wallet = LocalWallet::from(signing_key);
        let client = SignerMiddleware::new(unwrapped_client.to_owned(), wallet);
        let client = Arc::new(client);

        // TODO: get the output oracle address from the config (pass it into this function)
        let output_oracle_address = Address::zero();
        let output_oracle_contract = OutputOracleContract::new(output_oracle_address, client);

        let tx = output_oracle_contract.propose_l2_output(
            H256::from_slice(proposal.output_root.as_slice()).into(),
            U256::from(proposal.block_ref.number),
            // TODO: `current_l1` should contain the l1 hash and the l1 number
            // TODO: this should be the current l1 hash
            // proposal.sync_status.current_l1,
            H256::zero().into(),
            U256::from(proposal.sync_status.current_l1),
        );

        Ok(tx.tx)
    }

    /// Sends a [TypedTransaction] to the L1 network through an [L1Client].
    ///
    /// Returns an [eyre::Result] containing the [TransactionReceipt] for the sent transaction.
    pub async fn send_transaction(
        l1_client: Arc<Mutex<L1Client>>,
        private_key: &H256,
        tx: TypedTransaction,
    ) -> Result<Option<TransactionReceipt>> {
        tracing::debug!(target: "varro=pool", "Sending typed transaction: {:?}", tx);

        let unwrapped_client = l1_client.lock().await;
        let signing_key = SigningKey::from_bytes(&private_key.0)?;
        let wallet = LocalWallet::from(signing_key);
        let client = SignerMiddleware::new(unwrapped_client.to_owned(), wallet);
        let client = Arc::new(client);

        let receipt = client.send_transaction(tx, None).await?;
        let receipt = receipt.confirmations(1).await?;
        Ok(receipt)
    }
}
