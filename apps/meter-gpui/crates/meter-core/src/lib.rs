//! Meter core: reads the usage records AI tools already keep on this Mac and turns them into
//! quotas, daily totals, insights and alerts. No UI and no network. Prompt text is never read.

pub mod accounts;
pub mod alerts;
pub mod dashboard;
pub mod fakehome;
pub mod format;
pub mod insights;
pub mod model;
pub mod readers;
pub mod rollup;
pub mod snapshot;
pub mod util;

pub use accounts::{Account, AccountStore};
pub use alerts::{Alert, AlertEngine, AlertKind, AlertPolicy};
pub use dashboard::{AccountRun, AccountStatus, Dashboard};
pub use insights::{MenuSummary, Note, NoteKind, PaceVerdict};
pub use model::{Quota, ReaderOutput, Session, ToolId, UsageReader};
pub use rollup::{DayUsage, RollupStore};
