//! Read-only check of the readers against the real home folder. Prints counts and totals only:
//! no project names, no text. Writes nothing except private temp copies it deletes.
//!   meter-probe                    read the real data
//!   meter-probe --fake-home <dir>  write invented tool data into <dir> and exit
use chrono::{Duration, Utc};
use meter_core::{model::ToolId, readers};
use std::collections::BTreeSet;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--fake-home" {
        match meter_core::fakehome::build(std::path::Path::new(&args[2]), Utc::now()) {
            Ok(()) => println!("Fake home written to {}", args[2]),
            Err(e) => {
                eprintln!("could not write fake home: {e}");
                std::process::exit(1);
            }
        }
        return;
    }
    let since = Utc::now() - Duration::days(30);
    for tool in ToolId::ALL {
        let reader = readers::make(tool, None);
        let name = tool.display_name();
        if !reader.is_installed() {
            println!("{name}: not installed");
            continue;
        }
        match reader.read(since) {
            Ok(out) => {
                let sum = |f: fn(&meter_core::Session) -> Option<i64>| out.sessions.iter().filter_map(f).sum::<i64>();
                let models: BTreeSet<_> = out.sessions.iter().filter_map(|s| s.model.clone()).collect();
                println!(
                    "{name}: {} sessions (30d), input {}, output {}, turns {}, models {:?}",
                    out.sessions.len(),
                    sum(|s| s.input_tokens),
                    sum(|s| s.output_tokens),
                    sum(|s| s.turns),
                    models
                );
                for q in out.quotas {
                    println!("  quota {}: {}% resets {:?} observed {}", q.label, q.used_percent, q.resets_at, q.observed_at);
                }
            }
            Err(e) => println!("{name}: read failed: {e}"),
        }
    }
}
