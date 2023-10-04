use std::{
    collections::HashMap,
    env, fs,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, Ordering},
};

use near_units::parse_near;
use workspaces::{network::Sandbox, Account, Worker};

use crate::interface::{
    claim_contract_interface::ClaimContract,
    ft_contract_interface::{FtContract, FtContractInterface},
};

const EPOCH_BLOCKS_HEIGHT: u64 = 43_200;
const HOURS_PER_EPOCH: u64 = 12;
const ONE_HOUR_BLOCKS_HEIGHT: u64 = EPOCH_BLOCKS_HEIGHT / HOURS_PER_EPOCH;

pub(crate) struct Context {
    worker: Worker<Sandbox>,
    root_account: Account,
    pub accounts: HashMap<String, Account>,
    pub ft_contract: Box<dyn FtContract + Send + Sync>,
    pub claim_contract: Box<dyn ClaimContract + Send + Sync>,
}

impl Context {
    pub(crate) async fn new() -> anyhow::Result<Context> {
        println!("🏭 Initializing context");

        build_contract()?;

        let worker = workspaces::sandbox().await?;
        let root_account = worker.dev_create_account().await?;

        let claim_contract = worker.dev_deploy(&Self::load_wasm("../res/sweat_claim.wasm")).await?;
        let ft_contract = worker.dev_deploy(&Self::load_wasm("../res/sweat.wasm")).await?;

        println!("@@ claim contract deployed to {}", claim_contract.id());
        println!("@@ ft contract deployed to {}", ft_contract.id());

        Ok(Context {
            worker,
            root_account,
            accounts: HashMap::new(),
            ft_contract: Box::new(ft_contract),
            claim_contract: Box::new(claim_contract),
        })
    }

    pub(crate) async fn account(&mut self, name: &str) -> anyhow::Result<Account> {
        if !self.accounts.contains_key(name) {
            let account = self
                .root_account
                .create_subaccount(name)
                .initial_balance(parse_near!("3 N"))
                .transact()
                .await?
                .into_result()?;

            self.accounts.insert(name.to_string(), account);
        }

        Ok(self.accounts.get(name).unwrap().clone())
    }

    fn load_wasm(wasm_path: &str) -> Vec<u8> {
        let current_dir = env::current_dir().expect("Failed to get current dir");
        let wasm_filepath = fs::canonicalize(current_dir.join(wasm_path)).expect("Failed to get wasm file path");
        fs::read(wasm_filepath).expect("Failed to load wasm")
    }

    pub(crate) async fn fast_forward_hours(&self, hours: u64) -> anyhow::Result<()> {
        let blocks_to_advance = ONE_HOUR_BLOCKS_HEIGHT * hours;

        println!("⏳ Fast forward to {hours} hours ({blocks_to_advance} blocks)...");

        self.worker.fast_forward(blocks_to_advance).await?;

        Ok(())
    }

    pub(crate) fn get_signature_material(
        &self,
        receiver_id: &Account,
        product_id: &String,
        valid_until: u64,
        amount: u128,
        last_jar_id: Option<String>,
    ) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.claim_contract.account().id(),
            receiver_id.id(),
            product_id,
            amount,
            last_jar_id.map_or_else(String::new, |value| value,),
            valid_until,
        )
    }
}

static CONTRACT_READY: AtomicBool = AtomicBool::new(false);

/// Compile contract in release mode and prepare it for integration tests usage
pub fn build_contract() -> anyhow::Result<()> {
    if CONTRACT_READY.load(Ordering::Relaxed) {
        return Ok(());
    }

    Command::new("make")
        .arg("build")
        .current_dir("..")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    CONTRACT_READY.store(true, Ordering::Relaxed);

    Ok(())
}

pub(crate) struct Prepared {
    pub(crate) context: Context,
    pub(crate) manager: Account,
    pub(crate) alice: Account,
}

pub(crate) async fn prepare_contract() -> anyhow::Result<Prepared> {
    let mut context = Context::new().await?;

    let manager = context.account("manager").await?;
    let alice = context.account("alice").await?;

    context.ft_contract.init().await?;
    context.claim_contract.init(context.ft_contract.account()).await?;

    context
        .ft_contract
        .storage_deposit(context.claim_contract.account())
        .await?;

    context.ft_contract.storage_deposit(&alice).await?;
    context.ft_contract.mint_for_user(&alice, 100_000_000).await?;

    Ok(Prepared {
        context,
        manager,
        alice,
    })
}
