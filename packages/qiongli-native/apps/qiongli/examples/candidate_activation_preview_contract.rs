fn main() {
    println!(
        "{}",
        qiongli::candidate_activation_preview_contract_json()
            .expect("activation preview contract must serialize")
    );
}
