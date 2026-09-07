fn main() {
    println!(
        "{}",
        qiongli::update_recovery_contract_json().expect("recovery contract must serialize")
    );
}
