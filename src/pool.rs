#![allow(dead_code)]
#![allow(unused_variables)]

use futures::lock::Mutex;
use std::sync::Arc;

use crate::{
    rollup::OutputResponse,
    L1Client,
    OutputOracle,
};

use ethers_core::types::{
    Transaction,
    TransactionReceipt,
    H256,
};
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

    /// Constructs a [Transaction] for a given [OutputResponse] proposal.
    ///
    /// Returns an [eyre::Result] containing the constructed [Transaction].
    pub async fn construct_proposal_tx(
        proposal: OutputResponse,
        l1_client: Arc<Mutex<L1Client>>,
        private_key: &H256,
    ) -> Result<Transaction> {
        tracing::debug!(target: "varro=pool", "Constructing proposal: {:?}", proposal);
        // TODO:
        Err(eyre::eyre!("unimplemented"))
    }

    /// Sends a [Transaction] to the L1 network through an [L1Client].
    ///
    /// Returns an [eyre::Result] containing the [TransactionReceipt] for the sent transaction.
    pub async fn send_transaction(
        l1_client: Arc<Mutex<L1Client>>,
        private_key: &H256,
        tx: Transaction,
    ) -> Result<TransactionReceipt> {
        tracing::debug!(target: "varro=pool", "Sending transaction: {:?}", tx);
        // TODO:
        Err(eyre::eyre!("unimplemented"))
    }
}
