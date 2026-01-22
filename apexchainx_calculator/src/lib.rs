#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

#[contract]
pub struct SLACalculatorContract;

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");

#[contractimpl]
impl SLACalculatorContract {

    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&ADMIN_KEY, &admin);
    }


    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&ADMIN_KEY)
            .expect("Not initialized")
    }


    pub fn get_config(_env: Env, _severity: Symbol) -> Symbol {
    
        symbol_short!("TODO")
    }


    pub fn calculate_sla(
        _env: Env,
        _outage_id: Symbol,
        _severity: Symbol,
        _mttr_minutes: u32,
    ) -> Symbol {
    
        symbol_short!("TODO")
    }
}