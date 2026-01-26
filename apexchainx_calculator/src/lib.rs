#![no_std]

use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Env, Map, Symbol,
};

#[contract]
pub struct SLACalculatorContract;

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const CONFIG_KEY: Symbol = symbol_short!("CONFIG");

#[derive(Clone)]
pub struct SLAConfig {
    pub threshold_minutes: u32,
    pub penalty_per_minute: i128,
    pub reward_base: i128,
}

#[contractimpl]
impl SLACalculatorContract {
   pub fn initialize(env: Env, admin: Address) {
    if env.storage().instance().has(&ADMIN_KEY) {
        panic!("Already initialized");
    }

    env.storage().instance().set(&ADMIN_KEY, &admin);

    let mut configs = Map::<Symbol, SLAConfig>::new(&env);

    
    configs.set(
        symbol_short!("critical"),
        SLAConfig {
            threshold_minutes: 15,
            penalty_per_minute: 100,
            reward_base: 750,
        },
    );

    configs.set(
        symbol_short!("high"),
        SLAConfig {
            threshold_minutes: 30,
            penalty_per_minute: 50,
            reward_base: 750,
        },
    );

    configs.set(
        symbol_short!("medium"),
        SLAConfig {
            threshold_minutes: 60,
            penalty_per_minute: 25,
            reward_base: 750,
        },
    );

    configs.set(
        symbol_short!("low"),
        SLAConfig {
            threshold_minutes: 120,
            penalty_per_minute: 10,
            reward_base: 600,
        },
    );

    env.storage().instance().set(&CONFIG_KEY, &configs);
}

    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&ADMIN_KEY)
            .expect("Not initialized")
    }

    
    pub fn set_config(
        env: Env,
        caller: Address,
        severity: Symbol,
        threshold_minutes: u32,
        penalty_per_minute: i128,
        reward_base: i128,
    ) {
        
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        if caller != admin {
            panic!("Only admin can update config");
        }

        let mut configs: Map<Symbol, SLAConfig> = env
            .storage()
            .instance()
            .get(&CONFIG_KEY)
            .unwrap();

        let cfg = SLAConfig {
            threshold_minutes,
            penalty_per_minute,
            reward_base,
        };

        configs.set(severity, cfg);
        env.storage().instance().set(&CONFIG_KEY, &configs);
    }

    pub fn get_config(env: Env, severity: Symbol) -> SLAConfig {
        let configs: Map<Symbol, SLAConfig> = env
            .storage()
            .instance()
            .get(&CONFIG_KEY)
            .unwrap();

        configs.get(severity).expect("Config not found")
    }
}