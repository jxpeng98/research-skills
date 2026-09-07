fn main() {
    println!(
        "{}",
        qiongli::candidate_activation_completed_contract_json()
            .expect("activation contract must serialize")
    );
}
