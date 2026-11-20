use std::env;
use std::fs;
use std::path::Path;

#[test]
fn the_typescript_contract_in_the_ui_is_current() {
    let contract_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ui/core/contract.ts");
    let generated_contract = blossom::contract::typescript();
    if env::var_os("BLOSSOM_WRITE_CONTRACT").is_some() {
        fs::write(&contract_file, &generated_contract).unwrap();
    }

    let committed_contract = fs::read_to_string(&contract_file).unwrap_or_default();

    assert!(
        committed_contract == generated_contract,
        "ui/core/contract.ts is stale; regenerate it with BLOSSOM_WRITE_CONTRACT=1 cargo test --package blossom --test contract"
    );
}
