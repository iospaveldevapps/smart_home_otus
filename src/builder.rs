use crate::{Room, SmartDevice, SmartHome};

/// Builder for [`SmartHome`] with a fluent interface.
///
/// Uses the typestate pattern: devices can only be added after the first
/// room, which is enforced by the compiler:
///
/// ```compile_fail
/// use smart_home::{HomeBuilder, SmartSocket};
///
/// // No `add_device` method before the first `add_room`.
/// HomeBuilder::new().add_device("socket", SmartSocket::new(true, 100.0));
/// ```
#[derive(Debug, Default)]
pub struct HomeBuilder {
    home: SmartHome,
}

impl HomeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the first room and switches the builder into the state
    /// where devices can be added.
    pub fn add_room(self, name: impl Into<String>) -> HomeBuilderWithRooms {
        HomeBuilderWithRooms {
            home: self.home,
            current_name: name.into(),
            current_room: Room::new(),
        }
    }
}

/// Builder state with at least one room: devices go into the room
/// added last.
#[derive(Debug)]
pub struct HomeBuilderWithRooms {
    home: SmartHome,
    current_name: String,
    current_room: Room,
}

impl HomeBuilderWithRooms {
    /// Finishes the current room and starts a new one.
    pub fn add_room(mut self, name: impl Into<String>) -> Self {
        self.home.add_room(self.current_name, self.current_room);

        Self {
            home: self.home,
            current_name: name.into(),
            current_room: Room::new(),
        }
    }

    /// Adds a device to the room added last.
    pub fn add_device(mut self, name: impl Into<String>, device: impl Into<SmartDevice>) -> Self {
        self.current_room.add_device(name, device);
        self
    }

    pub fn build(mut self) -> SmartHome {
        self.home.add_room(self.current_name, self.current_room);
        self.home
    }
}

#[cfg(test)]
mod tests {
    use super::HomeBuilder;
    use crate::{SmartSocket, SmartThermometer};

    #[test]
    fn builds_home_with_rooms_and_devices() {
        let home = HomeBuilder::new()
            .add_room("living room")
            .add_device("socket_1", SmartSocket::new(true, 150.0))
            .add_device("thermo_1", SmartThermometer::new(23.5))
            .add_room("bedroom")
            .add_device("socket_2", SmartSocket::new(false, 60.0))
            .build();

        assert!(home.get_smart_device("living room", "socket_1").is_ok());
        assert!(home.get_smart_device("living room", "thermo_1").is_ok());
        assert!(home.get_smart_device("bedroom", "socket_2").is_ok());
    }

    #[test]
    fn builds_home_with_empty_rooms() {
        let home = HomeBuilder::new()
            .add_room("hall")
            .add_room("attic")
            .build();

        assert!(home.get_room("hall").is_some());
        assert!(home.get_room("attic").is_some());
    }
}
