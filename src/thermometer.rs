pub struct SmartThermometer {
    temperature: f32,
}

impl SmartThermometer {
    pub fn new(temperature: f32) -> Self {
        Self { temperature }
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature
    }
}

#[cfg(test)]
mod tests {
    use super::SmartThermometer;

    #[test]
    fn returns_stored_temperature() {
        let thermometer = SmartThermometer::new(21.5);

        assert_eq!(thermometer.get_temperature(), 21.5);
    }
}
