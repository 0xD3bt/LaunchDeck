pub fn provider_required_tip_lamports(provider: &str) -> Option<u64> {
    match provider.trim() {
        "helius-sender" | "jito-bundle" => Some(200_000),
        "hellomoon" => Some(1_000_000),
        _ => None,
    }
}

pub fn provider_min_tip_sol_label(provider: &str) -> &'static str {
    match provider.trim() {
        "hellomoon" => "0.001",
        _ => "0.0002",
    }
}
