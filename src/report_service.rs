use smart_home::Report;

pub fn print_report(report: &impl Report) {
    println!();
    println!("{}", report.report());
}
