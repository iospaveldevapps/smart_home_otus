pub trait Report {
    fn report(&self) -> String;

    fn print_report(&self) {
        println!("{}", self.report());
    }
}

impl<T: Report + ?Sized> Report for &T {
    fn report(&self) -> String {
        (**self).report()
    }
}
