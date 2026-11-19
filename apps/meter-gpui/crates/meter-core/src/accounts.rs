use crate::model::{ReaderOutput, ToolId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// A tool can have several accounts (for example personal and work). Labels are the user's own;
/// Meter never reads credentials or emails to build them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub tool: ToolId,
    pub label: String,
    /// Alternate data folder for this account; `None` means the tool's default.
    pub data_root: Option<String>,
    pub is_hidden: bool,
}

impl Account {
    pub fn default_for(tool: ToolId) -> Self {
        Account { id: "default".into(), tool, label: "Default".into(), data_root: None, is_hidden: false }
    }
    pub fn key(&self) -> String {
        format!("{}/{}", self.tool.raw(), self.id)
    }
}

pub struct AccountStore {
    pub path: PathBuf,
}

impl AccountStore {
    pub fn new(path: PathBuf) -> Self {
        AccountStore { path }
    }

    pub fn load(&self) -> Vec<Account> {
        std::fs::read(&self.path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }

    fn save(&self, accounts: &[Account]) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec(accounts)?)?;
        std::fs::rename(tmp, &self.path)
    }

    pub fn add(&self, account: Account) -> std::io::Result<Vec<Account>> {
        let mut all = self.load();
        all.push(account);
        self.save(&all)?;
        Ok(all)
    }

    /// Hiding keeps the account and its history; it only leaves the menu, notes and alerts.
    /// A tool's built-in account is created on first use so it can be hidden like any other.
    pub fn set_hidden(&self, hidden: bool, tool: ToolId, id: &str) -> std::io::Result<Vec<Account>> {
        let mut all = self.load();
        match all.iter_mut().find(|a| a.tool == tool && a.id == id) {
            Some(a) => a.is_hidden = hidden,
            None => all.push(Account { is_hidden: hidden, ..Account::default_for(tool) }),
        }
        self.save(&all)?;
        Ok(all)
    }

    pub fn remove(&self, tool: ToolId, id: &str) -> std::io::Result<Vec<Account>> {
        let all: Vec<Account> = self.load().into_iter().filter(|a| !(a.tool == tool && a.id == id)).collect();
        self.save(&all)?;
        Ok(all)
    }

    /// Stored accounts plus a built-in "default" for each tool that has none stored.
    pub fn effective(&self) -> Vec<Account> {
        let mut all = self.load();
        for t in ToolId::ALL {
            if !all.iter().any(|a| a.tool == t && a.id == "default") {
                all.push(Account::default_for(t));
            }
        }
        all.sort_by(|a, b| {
            let ka = (a.tool, a.id != "default", a.label.clone());
            let kb = (b.tool, b.id != "default", b.label.clone());
            ka.cmp(&kb)
        });
        all
    }

    /// "tool/accountID" keys, because every tool's built-in account is called "default".
    pub fn hidden_keys(&self) -> HashSet<String> {
        self.load().into_iter().filter(|a| a.is_hidden).map(|a| a.key()).collect()
    }
}

impl ReaderOutput {
    /// Drops sessions and quotas that belong to hidden accounts.
    pub fn excluding(mut self, hidden: &HashSet<String>) -> Self {
        self.sessions.retain(|s| !hidden.contains(&format!("{}/{}", s.tool.raw(), s.account_id)));
        self.quotas.retain(|q| !hidden.contains(&format!("{}/{}", q.tool.raw(), q.account_id)));
        self
    }
}
