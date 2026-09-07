fn main() {
    println!(
        "{}",
        qiongli::candidate_stage_contract_json().expect("stage contract must serialize")
    );
}
