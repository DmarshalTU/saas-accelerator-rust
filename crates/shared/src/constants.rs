/// String literal constants used for storing string values
pub mod string_literals {
    /// Identifies the `WebNotificationURL` app config key
    pub const WEB_NOTIFICATION_URL: &str = "WebNotificationUrl";
}

/// Claim constants for authentication/authorization
pub mod claim_constants {
    /// Defines the `CLAIM_EMAILADDRESS`
    pub const CLAIM_EMAILADDRESS: &str = "preferred_username";

    /// The claim name
    pub const CLAIM_NAME: &str = "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name";

    /// The claim short name
    pub const CLAIM_SHORT_NAME: &str = "name";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_literals_values() {
        assert_eq!(string_literals::WEB_NOTIFICATION_URL, "WebNotificationUrl");
    }

    #[test]
    fn claim_constants_values() {
        assert_eq!(claim_constants::CLAIM_EMAILADDRESS, "preferred_username");
        assert_eq!(claim_constants::CLAIM_SHORT_NAME, "name");
    }
}

