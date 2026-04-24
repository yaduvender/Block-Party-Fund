#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, contractclient,
    token, symbol_short,
    Address, Env, Map, String, Vec,
};

// ─── Storage Keys ──────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    EventName,
    GoalAmount,
    Deadline,
    Token,
    Contributions,
    TotalRaised,
    Finalized,
    Cancelled,
}

// ─── Contract ───────────────────────────────────────────────────────────────

#[contract]
pub struct CommunityPoolContract;

#[contractimpl]
impl CommunityPoolContract {

    /// Initialize the pool. Called once by the admin (organizer).
    ///
    /// * `admin`       – address that can finalize or cancel
    /// * `token`       – Stellar asset contract address (e.g. USDC)
    /// * `event_name`  – human-readable label shown in the UI
    /// * `goal_amount` – target amount in stroops (1 XLM = 10_000_000)
    /// * `deadline`    – Unix timestamp after which contributions close
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        event_name: String,
        goal_amount: i128,
        deadline: u64,
    ) {
        // Ensure this can only be called once
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin,       &admin);
        env.storage().instance().set(&DataKey::Token,       &token);
        env.storage().instance().set(&DataKey::EventName,   &event_name);
        env.storage().instance().set(&DataKey::GoalAmount,  &goal_amount);
        env.storage().instance().set(&DataKey::Deadline,    &deadline);
        env.storage().instance().set(&DataKey::TotalRaised, &0_i128);
        env.storage().instance().set(&DataKey::Finalized,   &false);
        env.storage().instance().set(&DataKey::Cancelled,   &false);

        // Initialize empty contributions map
        let contributions: Map<Address, i128> = Map::new(&env);
        env.storage().instance().set(&DataKey::Contributions, &contributions);

        env.events().publish(
            (symbol_short!("init"), event_name),
            (admin, goal_amount, deadline),
        );
    }

    // ─── Contribute ──────────────────────────────────────────────────────

    /// A neighbor contributes `amount` tokens to the pool.
    /// The tokens are transferred from `contributor` to the contract.
    pub fn contribute(env: Env, contributor: Address, amount: i128) {
        contributor.require_auth();
        Self::assert_active(&env);

        if amount <= 0 {
            panic!("amount must be positive");
        }

        // Pull tokens from contributor → contract
        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&contributor, &env.current_contract_address(), &amount);

        // Record contribution
        let mut contributions: Map<Address, i128> =
            env.storage().instance().get(&DataKey::Contributions).unwrap();
        let previous = contributions.get(contributor.clone()).unwrap_or(0);
        contributions.set(contributor.clone(), previous + amount);
        env.storage().instance().set(&DataKey::Contributions, &contributions);

        // Update total
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        env.storage().instance().set(&DataKey::TotalRaised, &(total + amount));

        env.events().publish(
            (symbol_short!("contrib"), contributor),
            amount,
        );
    }

    // ─── Withdraw Refund ─────────────────────────────────────────────────

    /// If the pool is cancelled, contributors can reclaim their funds.
    pub fn refund(env: Env, contributor: Address) {
        contributor.require_auth();

        let cancelled: bool = env.storage().instance().get(&DataKey::Cancelled).unwrap_or(false);
        if !cancelled {
            panic!("pool is not cancelled");
        }

        let mut contributions: Map<Address, i128> =
            env.storage().instance().get(&DataKey::Contributions).unwrap();
        let amount = contributions.get(contributor.clone()).unwrap_or(0);
        if amount == 0 {
            panic!("nothing to refund");
        }

        // Zero out before transfer (re-entrancy guard)
        contributions.set(contributor.clone(), 0);
        env.storage().instance().set(&DataKey::Contributions, &contributions);

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&env.current_contract_address(), &contributor, &amount);

        env.events().publish(
            (symbol_short!("refund"), contributor),
            amount,
        );
    }

    // ─── Finalize ────────────────────────────────────────────────────────

    /// Admin finalizes the pool and sweeps all funds to `recipient`.
    /// Can be called at any time by the admin (e.g. goal met early, or
    /// deadline passed and funds are used regardless).
    pub fn finalize(env: Env, recipient: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let finalized: bool = env.storage().instance().get(&DataKey::Finalized).unwrap_or(false);
        let cancelled:  bool = env.storage().instance().get(&DataKey::Cancelled).unwrap_or(false);
        if finalized || cancelled {
            panic!("pool already closed");
        }

        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        if total == 0 {
            panic!("no funds raised");
        }

        // Transfer everything to recipient (event organizer / treasury)
        let token_addr: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&env.current_contract_address(), &recipient, &total);

        env.storage().instance().set(&DataKey::Finalized, &true);

        env.events().publish(
            (symbol_short!("finalize"), recipient),
            total,
        );
    }

    // ─── Cancel ──────────────────────────────────────────────────────────

    /// Admin cancels the pool. Contributors may then call `refund`.
    pub fn cancel(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let finalized: bool = env.storage().instance().get(&DataKey::Finalized).unwrap_or(false);
        if finalized {
            panic!("pool already finalized");
        }

        env.storage().instance().set(&DataKey::Cancelled, &true);

        env.events().publish(
            (symbol_short!("cancel"), symbol_short!("pool")),
            (),
        );
    }

    // ─── Read-only helpers ───────────────────────────────────────────────

    /// Returns the total amount raised so far.
    pub fn total_raised(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0)
    }

    /// Returns the fundraising goal.
    pub fn goal(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::GoalAmount).unwrap()
    }

    /// Returns how much a specific address has contributed.
    pub fn contribution_of(env: Env, contributor: Address) -> i128 {
        let contributions: Map<Address, i128> =
            env.storage().instance().get(&DataKey::Contributions).unwrap();
        contributions.get(contributor).unwrap_or(0)
    }

    /// Returns whether the goal has been reached.
    pub fn goal_reached(env: Env) -> bool {
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0);
        let goal:  i128 = env.storage().instance().get(&DataKey::GoalAmount).unwrap();
        total >= goal
    }

    /// Returns pool status: "active" | "finalized" | "cancelled"
    pub fn status(env: Env) -> String {
        if env.storage().instance().get::<_, bool>(&DataKey::Finalized).unwrap_or(false) {
            return String::from_str(&env, "finalized");
        }
        if env.storage().instance().get::<_, bool>(&DataKey::Cancelled).unwrap_or(false) {
            return String::from_str(&env, "cancelled");
        }
        String::from_str(&env, "active")
    }

    // ─── Internal ────────────────────────────────────────────────────────

    fn assert_active(env: &Env) {
        let finalized: bool = env.storage().instance().get(&DataKey::Finalized).unwrap_or(false);
        let cancelled:  bool = env.storage().instance().get(&DataKey::Cancelled).unwrap_or(false);
        if finalized || cancelled {
            panic!("pool is closed");
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() > deadline {
            panic!("contribution deadline has passed");
        }
    }
}