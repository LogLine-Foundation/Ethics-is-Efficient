
use anyhow::{Result, anyhow};
use serde_json::Value;

pub struct ExecOutcome {
    pub decision: String,
    pub fuel_consumed: u64,
}

pub fn run(bytes: &[u8], _input: &Value, fuel_limit: u64) -> Result<ExecOutcome> {
    // Deterministic, import-free sandbox. If instantiation fails, return ASK.
    let mut config = wasmtime::Config::new();
    config.consume_fuel(true);
    // Disable all non-deterministic features by default
    let engine = wasmtime::Engine::new(&config)?;
    let store = wasmtime::Store::new(&engine);
    store.add_fuel(fuel_limit).ok();
    let module = wasmtime::Module::from_binary(&engine, bytes)
        .map_err(|e| anyhow!("module_err: {e}"))?;
    let mut linker = wasmtime::Linker::new(&engine);
    // No imports linked
    let instance = linker.instantiate(&store, &module)
        .map_err(|e| anyhow!("link_err: {e}"))?;

    // Try call exported "run" fn with () -> i32 mapping to 0=ACK,1=ASK,2=NACK
    let func = instance.get_func("run").ok_or_else(|| anyhow!("missing_export: run"))?;
    let typed = func.typed::<(), i32>().map_err(|e| anyhow!("type_err: {e}"))?;
    let rc = typed.call(()).map_err(|e| anyhow!("call_err: {e}"))?;

    let decision = match rc {
        0 => "ACK",
        1 => "ASK",
        2 => "NACK",
        _ => "ASK"
    }.to_string();

    let fuel_left = store.fuel().unwrap_or(0);
    let consumed = fuel_limit.saturating_sub(fuel_left);
    Ok(ExecOutcome{ decision, fuel_consumed: consumed })
}
