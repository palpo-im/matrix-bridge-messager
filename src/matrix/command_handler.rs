use crate::matrix::MatrixEvent;

const DEFAULT_PROVISIONING_POWER_LEVEL: i64 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatrixCommandPermission {
    pub required_level: i64,
    pub category: &'static str,
    pub subcategory: &'static str,
    pub self_service: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatrixCommandOutcome {
    Ignored,
    Reply(String),
    BridgeRequested { phone_number: String },
    UnbridgeRequested,
}

#[derive(Debug, Clone)]
pub struct MatrixCommandHandler {
    prefix: &'static str,
    self_service_enabled: bool,
    provisioning_power_level: i64,
}

impl Default for MatrixCommandHandler {
    fn default() -> Self {
        Self {
            prefix: "!message",
            self_service_enabled: true,
            provisioning_power_level: DEFAULT_PROVISIONING_POWER_LEVEL,
        }
    }
}

impl MatrixCommandHandler {
    pub fn new(self_service_enabled: bool, provisioning_power_level: Option<i64>) -> Self {
        Self {
            self_service_enabled,
            provisioning_power_level: provisioning_power_level
                .unwrap_or(DEFAULT_PROVISIONING_POWER_LEVEL),
            ..Self::default()
        }
    }

    pub fn is_command(&self, message: &str) -> bool {
        message.trim_start().starts_with(self.prefix)
    }

    pub fn handle<P>(
        &self,
        message: &str,
        room_is_bridged: bool,
        permission_check: P,
    ) -> MatrixCommandOutcome
    where
        P: Fn(MatrixCommandPermission) -> Result<bool, String>,
    {
        let message = message.trim_start();
        let without_prefix = match message.strip_prefix(self.prefix) {
            Some(s) => s.trim_start(),
            None => return MatrixCommandOutcome::Ignored,
        };

        let parts: Vec<&str> = without_prefix.splitn(2, ' ').collect();
        let command = parts.get(0).unwrap_or(&"");
        let args = parts.get(1).unwrap_or(&"");

        match *command {
            "help" => MatrixCommandOutcome::Reply(self.render_help()),
            "bridge" => {
                if let Err(reply) = self.ensure_permission(&permission_check) {
                    return MatrixCommandOutcome::Reply(reply);
                }
                if room_is_bridged {
                    return MatrixCommandOutcome::Reply(
                        "This room is already bridged to a phone number.".to_string(),
                    );
                }

                if args.is_empty() {
                    return MatrixCommandOutcome::Reply(
                        "Usage: !message bridge <phone_number>".to_string(),
                    );
                }

                let phone_number = args.trim().to_string();
                MatrixCommandOutcome::BridgeRequested { phone_number }
            }
            "unbridge" => {
                if let Err(reply) = self.ensure_permission(&permission_check) {
                    return MatrixCommandOutcome::Reply(reply);
                }
                if !room_is_bridged {
                    return MatrixCommandOutcome::Reply("This room is not bridged.".to_string());
                }
                MatrixCommandOutcome::UnbridgeRequested
            }
            "ping" => MatrixCommandOutcome::Reply("Pong!".to_string()),
            "status" => MatrixCommandOutcome::Reply(
                "Matrix-SMS Bridge is running.\nGateway: Configured".to_string(),
            ),
            "" => MatrixCommandOutcome::Reply(self.render_help()),
            _ => MatrixCommandOutcome::Reply(format!(
                "Unknown command. Use {} help for available commands.",
                self.prefix
            )),
        }
    }

    fn ensure_permission<P>(&self, permission_check: &P) -> Result<(), String>
    where
        P: Fn(MatrixCommandPermission) -> Result<bool, String>,
    {
        let permission = MatrixCommandPermission {
            required_level: self.provisioning_power_level,
            category: "provisioning",
            subcategory: "bridge",
            self_service: self.self_service_enabled,
        };

        match permission_check(permission) {
            Ok(true) => Ok(()),
            Ok(false) => Err("You don't have permission to use this command.".to_string()),
            Err(e) => Err(e),
        }
    }

    fn render_help(&self) -> String {
        format!(
            "Matrix-SMS Bridge Commands:\n\
            \n\
            {} help - Show this help message\n\
            {} bridge <phone_number> - Bridge this room to a phone number\n\
            {} unbridge - Remove bridge from this room\n\
            {} ping - Test if the bridge is responding\n\
            {} status - Show bridge status\n\
            \n\
            Examples:\n\
            {} bridge +1234567890\n\
            {} unbridge",
            self.prefix,
            self.prefix,
            self.prefix,
            self.prefix,
            self.prefix,
            self.prefix,
            self.prefix
        )
    }
}
