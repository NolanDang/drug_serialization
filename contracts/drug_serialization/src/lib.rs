#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol};

/// On-chain pharmaceutical unit serialization. Every drug pack is bound to a
/// unique GTIN-like serial at production time. Custody hops
/// (manufacturer -> distributor -> pharmacist) are recorded on-chain, and the
/// final dispense consumes the serial, making a second dispense attempt on the
/// same serial a reliable on-chain counterfeit signal.
#[contract]
pub struct DrugSerialization;

/// On-chain record for a single drug pack identified by its GTIN-like serial.
#[contracttype]
#[derive(Clone)]
pub struct DrugPack {
    /// Human-readable drug name, e.g. "Paracetamol-500mg".
    pub drug: String,
    /// Regulation category stored as a `Symbol` (e.g. "OTC", "RX", "VAX").
    pub drug_category: Symbol,
    /// Manufacturing batch identifier.
    pub batch: String,
    /// Expiry as a unix timestamp in seconds.
    pub expiry: u64,
    /// Address of the original manufacturer.
    pub manufacturer: Address,
    /// Address of the most recent custodian.
    pub last_handler: Address,
    /// Latest known storage or distribution location label.
    pub last_location: String,
    /// Number of custody transfers since production.
    pub custody_count: u32,
    /// True once a pharmacist has dispensed the pack to a patient.
    pub dispensed: bool,
    /// True if a regulatory authority has recalled the pack.
    pub recalled: bool,
    /// Reason supplied by the authority at recall time.
    pub recall_reason: String,
    /// Pseudonymous patient identifier set at dispense time.
    pub patient_hash: String,
    /// Ledger sequence number when production was recorded.
    pub produced_at: u32,
}

/// Storage keys for the contract.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Singleton slot holding the regulatory `Authority` address.
    Authority,
    /// Per-serial pack record.
    Pack(String),
}

#[contractimpl]
impl DrugSerialization {
    /// Initialize the contract and register the regulatory `authority` that
    /// is allowed to recall packs. Must be called exactly once before any
    /// other function. Panics if the contract is already initialized.
    pub fn init(env: Env, authority: Address) {
        if env.storage().instance().has(&DataKey::Authority) {
            panic!("contract already initialized");
        }
        env.storage().instance().set(&DataKey::Authority, &authority);
    }

    /// Manufacturer records production of a new drug pack with a unique
    /// `serial` (a GTIN-like identifier). The `expiry` must be a future
    /// unix timestamp. After this call the manufacturer is the first
    /// custodian (`custody_count == 0`) and the pack lives at "FACTORY".
    /// Panics if the serial already exists or the expiry is not in the
    /// future.
    pub fn produce(
        env: Env,
        manufacturer: Address,
        serial: String,
        drug: String,
        drug_category: Symbol,
        batch: String,
        expiry: u64,
    ) {
        manufacturer.require_auth();

        let key = DataKey::Pack(serial.clone());
        if env.storage().instance().has(&key) {
            panic!("serial already exists");
        }
        if expiry <= env.ledger().timestamp() {
            panic!("expiry must be in the future");
        }

        let factory = String::from_str(&env, "FACTORY");
        let empty = String::from_str(&env, "");

        let pack = DrugPack {
            drug,
            drug_category,
            batch,
            expiry,
            manufacturer: manufacturer.clone(),
            last_handler: manufacturer,
            last_location: factory,
            custody_count: 0,
            dispensed: false,
            recalled: false,
            recall_reason: empty.clone(),
            patient_hash: empty,
            produced_at: env.ledger().sequence(),
        };
        env.storage().instance().set(&key, &pack);
    }

    /// Distributor (or any downstream custodian) takes custody of `serial`
    /// and records its current `location`. Each successful call increments
    /// `custody_count` so verifiers can count hops along the supply chain.
    /// Panics if the serial is unknown, already dispensed, or recalled.
    pub fn transfer(env: Env, distributor: Address, serial: String, location: String) {
        distributor.require_auth();

        let key = DataKey::Pack(serial);
        let mut pack: DrugPack = env
            .storage()
            .instance()
            .get(&key)
            .expect("unknown serial");

        if pack.dispensed {
            panic!("pack already dispensed");
        }
        if pack.recalled {
            panic!("pack has been recalled");
        }

        pack.last_handler = distributor;
        pack.last_location = location;
        pack.custody_count = pack.custody_count.saturating_add(1);

        env.storage().instance().set(&key, &pack);
    }

    /// Pharmacist dispenses `serial` to a patient identified by a
    /// pseudonymous `patient_hash` (e.g. a hash of an off-chain patient
    /// record). One-time only: a second call on the same serial panics,
    /// which is how the on-chain ledger surfaces counterfeit or
    /// re-packaged units. Panics if the pack is unknown, recalled, or
    /// already past its expiry.
    pub fn dispense(
        env: Env,
        pharmacist: Address,
        serial: String,
        patient_hash: String,
    ) {
        pharmacist.require_auth();

        let key = DataKey::Pack(serial);
        let mut pack: DrugPack = env
            .storage()
            .instance()
            .get(&key)
            .expect("unknown serial");

        if pack.dispensed {
            panic!("pack already dispensed - possible counterfeit");
        }
        if pack.recalled {
            panic!("pack has been recalled");
        }
        if env.ledger().timestamp() >= pack.expiry {
            panic!("pack expired");
        }

        pack.dispensed = true;
        pack.patient_hash = patient_hash;
        pack.last_handler = pharmacist;
        env.storage().instance().set(&key, &pack);
    }

    /// Verify `serial` and return the number of custody hops it has
    /// travelled since production. Returns `0` for an unknown serial,
    /// which also serves as a "not registered / counterfeit" signal.
    pub fn verify(env: Env, serial: String) -> u32 {
        let key = DataKey::Pack(serial);
        match env.storage().instance().get::<DataKey, DrugPack>(&key) {
            Some(pack) => pack.custody_count,
            None => 0u32,
        }
    }

    /// Returns true if `serial` has already been dispensed. Because
    /// `dispense` rejects a second call on the same serial, reading this
    /// flag is the canonical on-chain counterfeit check used by
    /// pharmacies, distributors, and patients.
    pub fn is_dispensed(env: Env, serial: String) -> bool {
        let key = DataKey::Pack(serial);
        match env.storage().instance().get::<DataKey, DrugPack>(&key) {
            Some(pack) => pack.dispensed,
            None => false,
        }
    }

    /// Returns true if the regulatory authority has recalled `serial`.
    /// Recalled packs can no longer be transferred or dispensed.
    pub fn is_recalled(env: Env, serial: String) -> bool {
        let key = DataKey::Pack(serial);
        match env.storage().instance().get::<DataKey, DrugPack>(&key) {
            Some(pack) => pack.recalled,
            None => false,
        }
    }

    /// Return the full `DrugPack` record for `serial`, or `None` if the
    /// serial was never produced. Useful for inspectors and consumers
    /// that want to inspect the full provenance of a pack.
    pub fn get_pack(env: Env, serial: String) -> Option<DrugPack> {
        env.storage().instance().get(&DataKey::Pack(serial))
    }

    /// Regulatory `authority` recalls `serial` with a human-readable
    /// `reason`. The pack can no longer be transferred or dispensed.
    /// Panics if the caller is not the registered authority, the serial
    /// is unknown, or the pack has already been dispensed to a patient.
    pub fn recall(env: Env, authority: Address, serial: String, reason: String) {
        authority.require_auth();

        let stored: Address = env
            .storage()
            .instance()
            .get(&DataKey::Authority)
            .expect("contract not initialized");
        if stored != authority {
            panic!("caller is not the regulatory authority");
        }

        let key = DataKey::Pack(serial);
        let mut pack: DrugPack = env
            .storage()
            .instance()
            .get(&key)
            .expect("unknown serial");

        if pack.dispensed {
            panic!("cannot recall a pack that has already been dispensed");
        }

        pack.recalled = true;
        pack.recall_reason = reason;
        env.storage().instance().set(&key, &pack);
    }
}
