fn main() {
    println!(
        "{}",
        qiongli::plugin_source_contract_json().expect("source contract must serialize")
    );
}
