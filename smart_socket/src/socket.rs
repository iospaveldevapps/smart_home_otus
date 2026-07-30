/// Emulated smart socket: can be turned on and off and reports power draw.
#[derive(Debug)]
pub struct SmartSocket {
    is_on: bool,
    power: f32,
}

impl SmartSocket {
    /// Creates a turned off socket that draws `power` watts while on.
    pub fn new(power: f32) -> Self {
        Self {
            is_on: false,
            power,
        }
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

    /// Current power draw: zero while the socket is off.
    pub fn power(&self) -> f32 {
        if self.is_on { self.power } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::SmartSocket;

    #[test]
    fn new_socket_is_off_and_draws_no_power() {
        let socket = SmartSocket::new(1500.0);

        assert!(!socket.is_on());
        assert_eq!(socket.power(), 0.0);
    }

    #[test]
    fn changes_state_with_turn_on_and_turn_off() {
        let mut socket = SmartSocket::new(1500.0);

        socket.turn_on();
        assert!(socket.is_on());
        assert_eq!(socket.power(), 1500.0);

        socket.turn_off();
        assert!(!socket.is_on());
        assert_eq!(socket.power(), 0.0);
    }
}
