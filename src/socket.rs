pub struct SmartSocket {
    is_on: bool,
    power: f32,
}

impl SmartSocket {
    pub fn new(is_on: bool, power: f32) -> Self {
        Self { is_on, power }
    }

    pub fn turn_on(&mut self) {
        self.is_on = true;
    }

    pub fn turn_off(&mut self) {
        self.is_on = false;
    }

    pub fn is_on(&self) -> bool {
        self.is_on
    }

    pub fn get_power(&self) -> f32 {
        if self.is_on { self.power } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::SmartSocket;

    #[test]
    fn reports_zero_power_when_turned_off() {
        let socket = SmartSocket::new(false, 150.0);

        assert!(!socket.is_on());
        assert_eq!(socket.get_power(), 0.0);
    }

    #[test]
    fn changes_state_with_turn_on_and_turn_off() {
        let mut socket = SmartSocket::new(false, 200.0);

        socket.turn_on();
        assert!(socket.is_on());
        assert_eq!(socket.get_power(), 200.0);

        socket.turn_off();
        assert!(!socket.is_on());
        assert_eq!(socket.get_power(), 0.0);
    }
}
