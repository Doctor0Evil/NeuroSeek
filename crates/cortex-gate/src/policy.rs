#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub ip: String,
    pub asn: Option<String>,
    pub country: Option<String>,
    pub device_fingerprint: String,
    pub is_interactive: bool,
}

impl PolicyEngine {
    pub fn evaluate_session(&self, sess: &SessionContext, action: &SovereignAction) -> Decision {
        let mut reasons = Vec::new();

        // Example: block writes from untrusted geo
        if let Some(country) = &sess.country {
            if !self.trusted_countries.contains(country) && action.kind != SovereignActionKind::ReadNeuralShard {
                reasons.push(format!("untrusted_geo:{}", country));
                return Decision::Deny {
                    reason: "Untrusted geo for non-read action".into(),
                };
            }
        }

        // Example: require interactive sessions for high-risk actions
        if !sess.is_interactive && matches!(action.kind, SovereignActionKind::ProposeEvolve | SovereignActionKind::ApplyOta | SovereignActionKind::SignTransaction) {
            reasons.push("non_interactive_high_risk".into());
            return Decision::Deny {
                reason: "High-risk action requires interactive session".into(),
            };
        }

        Decision::Allow
    }
}
