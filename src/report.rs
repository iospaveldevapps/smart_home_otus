pub trait Report {
    fn report(&self) -> String;

    fn print_report(&self) {
        println!("{}", self.report());
    }
}
