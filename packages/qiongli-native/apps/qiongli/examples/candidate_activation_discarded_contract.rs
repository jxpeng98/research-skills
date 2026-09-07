fn main() {
    println!(
        "{}",
        qiongli::candidate_activation_discarded_contract_json()
            .expect("discard contract must serialize")
    );
}
