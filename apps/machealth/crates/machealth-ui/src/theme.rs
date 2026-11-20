use gpui::{Rgba, WindowAppearance, rgb};
use machealth_core::cleanup::FileKind;
use machealth_core::processes::descriptions::ImpactLabel;
use machealth_core::{AuditStatus, HealthGrade, Severity};

/// Native macOS Sequoia / Sonoma Human Interface Guidelines color tokens.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub window: Rgba,
    pub sidebar: Rgba,
    pub toolbar: Rgba,
    pub card: Rgba,
    pub card_elevated: Rgba,
    pub row_hover: Rgba,
    pub text: Rgba,
    pub secondary: Rgba,
    pub muted: Rgba,
    pub line: Rgba,
    pub track: Rgba,
    pub accent: Rgba,
    pub selected: Rgba,
    pub selected_text: Rgba,
    pub green: Rgba,
    pub green_bg: Rgba,
    pub amber: Rgba,
    pub amber_bg: Rgba,
    pub red: Rgba,
    pub red_bg: Rgba,
    pub purple: Rgba,
    pub pink: Rgba,
    pub teal: Rgba,
    pub indigo: Rgba,
}

impl Theme {
    pub fn for_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self {
                window: rgb(0x1C1C1E),
                sidebar: rgb(0x232326),
                toolbar: rgb(0x232326),
                card: rgb(0x28282C),
                card_elevated: rgb(0x323237),
                row_hover: rgb(0x303036),
                text: rgb(0xF5F5F7),
                secondary: rgb(0x98989D),
                muted: rgb(0x6E6E73),
                line: rgb(0x3A3A3F),
                track: rgb(0x38383D),
                accent: rgb(0x0A84FF),
                selected: rgb(0x0A84FF),
                selected_text: rgb(0xFFFFFF),
                green: rgb(0x30D158),
                green_bg: rgb(0x163320),
                amber: rgb(0xFF9F0A),
                amber_bg: rgb(0x382810),
                red: rgb(0xFF453A),
                red_bg: rgb(0x3B1818),
                purple: rgb(0xBF5AF2),
                pink: rgb(0xFF375F),
                teal: rgb(0x64D2FF),
                indigo: rgb(0x5E5CE6),
            },
            _ => Self {
                window: rgb(0xF2F2F7),
                sidebar: rgb(0xE8E8ED),
                toolbar: rgb(0xF9F9FB),
                card: rgb(0xFFFFFF),
                card_elevated: rgb(0xF2F2F7),
                row_hover: rgb(0xF5F5FA),
                text: rgb(0x1D1D1F),
                secondary: rgb(0x6E6E73),
                muted: rgb(0x8E8E93),
                line: rgb(0xE0E0E5),
                track: rgb(0xE5E5EA),
                accent: rgb(0x007AFF),
                selected: rgb(0x007AFF),
                selected_text: rgb(0xFFFFFF),
                green: rgb(0x1D9A38),
                green_bg: rgb(0xE5F6E8),
                amber: rgb(0xB56A00),
                amber_bg: rgb(0xFFF1D6),
                red: rgb(0xD70015),
                red_bg: rgb(0xFDE7E9),
                purple: rgb(0xAF52DE),
                pink: rgb(0xFF2D55),
                teal: rgb(0x32ADE6),
                indigo: rgb(0x5856D6),
            },
        }
    }

    pub fn severity_colors(&self, severity: Severity) -> (Rgba, Rgba) {
        match severity {
            Severity::Good => (self.green, self.green_bg),
            Severity::Attention => (self.amber, self.amber_bg),
            Severity::Critical => (self.red, self.red_bg),
        }
    }

    pub fn grade_colors(&self, grade: HealthGrade) -> (Rgba, Rgba) {
        match grade {
            HealthGrade::Great => (self.green, self.green_bg),
            HealthGrade::Good => (self.accent, self.card_elevated),
            HealthGrade::Fair => (self.amber, self.amber_bg),
            HealthGrade::NeedsAttention => (self.red, self.red_bg),
        }
    }

    pub fn impact_colors(&self, impact: ImpactLabel) -> (Rgba, Rgba) {
        match impact {
            ImpactLabel::Light => (self.green, self.green_bg),
            ImpactLabel::Moderate => (self.amber, self.amber_bg),
            ImpactLabel::Heavy => (self.red, self.red_bg),
        }
    }

    pub fn audit_colors(&self, status: AuditStatus) -> (Rgba, Rgba) {
        match status {
            AuditStatus::Pass => (self.green, self.green_bg),
            AuditStatus::Unknown => (self.secondary, self.track),
            AuditStatus::Warn => (self.amber, self.amber_bg),
            AuditStatus::Critical => (self.red, self.red_bg),
        }
    }

    /// macOS System Settings -> Storage palette for file categories.
    pub fn file_kind_color(&self, kind: FileKind) -> Rgba {
        match kind {
            FileKind::Videos => self.purple,
            FileKind::Images => self.pink,
            FileKind::Archives => self.amber,
            FileKind::Documents => self.accent,
            FileKind::AppsAndInstallers => self.red,
            FileKind::DeveloperAndCode => self.teal,
            FileKind::Audio => self.indigo,
            FileKind::Other => self.secondary,
        }
    }
}
