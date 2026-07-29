use crate::Report;

/// Composite that gathers reports from heterogeneous objects.
///
/// Uses static polymorphism: every `add` call wraps the current chain
/// into a new tuple type, so no trait objects are involved.
#[derive(Debug, Default)]
pub struct Reporter<T = ()> {
    items: T,
}

impl Reporter<()> {
    pub fn new() -> Self {
        Self::default()
    }
}

/// A statically typed chain of report sources collected by [`Reporter`].
pub trait ReportSources {
    fn collect_reports(&self, reports: &mut Vec<String>);
}

impl ReportSources for () {
    fn collect_reports(&self, _reports: &mut Vec<String>) {}
}

impl<Chain: ReportSources, Item: Report> ReportSources for (Chain, Item) {
    fn collect_reports(&self, reports: &mut Vec<String>) {
        let (chain, item) = self;
        chain.collect_reports(reports);
        reports.push(item.report());
    }
}

impl<T: ReportSources> Reporter<T> {
    // The name is part of the required fluent API, not arithmetic addition.
    #[allow(clippy::should_implement_trait)]
    pub fn add<R: Report>(self, item: R) -> Reporter<(T, R)> {
        Reporter {
            items: (self.items, item),
        }
    }

    /// Prints the report about every added object to the terminal.
    pub fn report(&self) {
        println!("{}", self.report_string());
    }

    pub fn report_string(&self) -> String {
        let mut reports = Vec::new();
        self.items.collect_reports(&mut reports);
        reports.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::Reporter;
    use crate::{SmartSocket, SmartThermometer, smart_room};

    #[test]
    fn collects_reports_from_all_added_objects() {
        let room = smart_room!("socket" => SmartSocket::new(false, 50.0));
        let socket = SmartSocket::new(true, 100.0);
        let thermometer = SmartThermometer::new(21.0);

        let report = Reporter::new()
            .add(&room)
            .add(&socket)
            .add(&thermometer)
            .report_string();

        assert!(report.contains("Устройство `socket`"));
        assert!(report.contains("Умная розетка. Включена: true"));
        assert!(report.contains("Умный термометр. Температура: 21"));
    }

    #[test]
    fn empty_reporter_produces_empty_report() {
        assert_eq!(Reporter::new().report_string(), "");
    }
}
