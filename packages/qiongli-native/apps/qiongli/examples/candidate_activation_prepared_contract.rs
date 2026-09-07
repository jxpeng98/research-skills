fn main() {
    println!(
        "{}",
        qiongli::candidate_activation_prepared_contract_json()
            .expect("activation prepared contract must serialize")
    );
}
