//! Inspect the exact fictional request or its deterministic evidence report.
use reactcoach_decision_engine::simulation::{demo_request, simulate};

fn main() {
    let request = demo_request();
    if std::env::args().nth(1).as_deref() == Some("request") {
        println!(
            "{}",
            serde_json::to_string_pretty(&request).expect("serialize request")
        );
    } else {
        let report = simulate(request).expect("valid fictional demo");
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("serialize report")
        );
    }
}
