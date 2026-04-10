#[derive(Clone, PartialEq, Default)]
pub enum RelayDirection {
    #[default]
    Both,
    MqttToReticulum,
    ReticulumToMqtt,
}

impl std::fmt::Debug for RelayDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayDirection::Both => write!(f, "Both"),
            RelayDirection::MqttToReticulum => write!(f, "MQTT → Reticulum"),
            RelayDirection::ReticulumToMqtt => write!(f, "Reticulum → MQTT"),
        }
    }
}