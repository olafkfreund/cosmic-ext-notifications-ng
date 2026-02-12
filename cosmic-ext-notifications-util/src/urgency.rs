/// Notification urgency level as defined by the freedesktop.org specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NotificationUrgency {
    /// Low urgency notification
    Low = 0,
    /// Normal urgency notification (default)
    #[default]
    Normal = 1,
    /// Critical urgency notification
    Critical = 2,
}

impl From<u8> for NotificationUrgency {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Low,
            2 => Self::Critical,
            _ => Self::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urgency_from_u8_low() {
        let urgency = NotificationUrgency::from(0);
        assert_eq!(urgency, NotificationUrgency::Low);
    }

    #[test]
    fn test_urgency_from_u8_normal() {
        let urgency = NotificationUrgency::from(1);
        assert_eq!(urgency, NotificationUrgency::Normal);
    }

    #[test]
    fn test_urgency_from_u8_critical() {
        let urgency = NotificationUrgency::from(2);
        assert_eq!(urgency, NotificationUrgency::Critical);
    }

    #[test]
    fn test_urgency_from_u8_invalid_defaults_to_normal() {
        let urgency = NotificationUrgency::from(3);
        assert_eq!(urgency, NotificationUrgency::Normal);

        let urgency = NotificationUrgency::from(255);
        assert_eq!(urgency, NotificationUrgency::Normal);
    }

    #[test]
    fn test_urgency_default() {
        let urgency: NotificationUrgency = Default::default();
        assert_eq!(urgency, NotificationUrgency::Normal);
    }

    #[test]
    fn test_urgency_clone() {
        let urgency = NotificationUrgency::Critical;
        let cloned = urgency;
        assert_eq!(urgency, cloned);
    }

    #[test]
    fn test_urgency_copy() {
        let urgency = NotificationUrgency::Low;
        let copied = urgency;
        assert_eq!(urgency, copied);
        // Verify original is still usable (copy trait)
        assert_eq!(urgency, NotificationUrgency::Low);
    }

    #[test]
    fn test_urgency_equality() {
        assert_eq!(NotificationUrgency::Low, NotificationUrgency::Low);
        assert_eq!(NotificationUrgency::Normal, NotificationUrgency::Normal);
        assert_eq!(NotificationUrgency::Critical, NotificationUrgency::Critical);

        assert_ne!(NotificationUrgency::Low, NotificationUrgency::Normal);
        assert_ne!(NotificationUrgency::Normal, NotificationUrgency::Critical);
        assert_ne!(NotificationUrgency::Low, NotificationUrgency::Critical);
    }

    #[test]
    fn test_urgency_debug_format() {
        let low = NotificationUrgency::Low;
        let normal = NotificationUrgency::Normal;
        let critical = NotificationUrgency::Critical;

        assert_eq!(format!("{:?}", low), "Low");
        assert_eq!(format!("{:?}", normal), "Normal");
        assert_eq!(format!("{:?}", critical), "Critical");
    }

    #[test]
    fn test_urgency_repr_values() {
        // Verify the repr(u8) values are correct
        assert_eq!(NotificationUrgency::Low as u8, 0);
        assert_eq!(NotificationUrgency::Normal as u8, 1);
        assert_eq!(NotificationUrgency::Critical as u8, 2);
    }

    #[test]
    fn test_urgency_from_conversion_roundtrip() {
        let low = NotificationUrgency::Low;
        let normal = NotificationUrgency::Normal;
        let critical = NotificationUrgency::Critical;

        assert_eq!(NotificationUrgency::from(low as u8), low);
        assert_eq!(NotificationUrgency::from(normal as u8), normal);
        assert_eq!(NotificationUrgency::from(critical as u8), critical);
    }
}
