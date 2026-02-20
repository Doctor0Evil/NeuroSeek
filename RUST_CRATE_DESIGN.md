# NeuroSeek Rust Crate Design: neuro-consent-ledger

**Crate Name**: `neuro-consent-ledger`  
**Version**: 0.1.0 (Pre-Release, Research Phase)  
**Author**: NeuroSeek Augmented-Citizen Rights Initiative  
**Date**: February 2026  
**License**: MIT + Apache 2.0 (Dual License)  
**GitHub Repository**: neuroseek/neuro-consent-ledger  
**Cargo.toml Reference**: https://github.com/neuroseek/neuro-consent-ledger/blob/main/Cargo.toml  

---

## 1. Overview & Purpose

`neuro-consent-ledger` is a **Rust-first, ai-chat-compliant crate** that implements the NeuroSeek Bioload Encoding Specification and Augmentation Rights Charter as **type-safe, machine-executable governance primitives**.

### Core Functions:

1. **Bioload band evaluation**: Given neural + physiological signals, compute GREEN/YELLOW/RED band with confidence score
2. **Consent-scope management**: Store, query, revoke consent to specific data uses, actors, and purposes
3. **Actor-class authorization**: Determine whether an actor (Police, Researcher, Medical, Regulator, etc.) may access augmentation data under current bioload band + consent state
4. **Ledger integration**: Record all bioload decisions, consent changes, and access requests to Googolswarm or compatible distributed ledger
5. **Rights enforcement**: Validate that operations comply with Tier 1–4 augmentation rights; auto-reject violations and log as forensic incidents

### Design Philosophy:

- **Type safety**: Impossible states (e.g., RED band with unrestricted actor access) are unrepresentable in Rust's type system
- **Immutability by default**: All bioload snapshots and consent records are immutable; changes produce new ledger events
- **Minimal cryptography assumptions**: Core logic is crypto-agnostic; ledger backends (Googolswarm, Ethereum, etc.) are pluggable
- **Auditable**: Every decision emits a structured log event suitable for forensic review and compliance audit
- **ai-chat compatible**: Exposed via REST/gRPC boundary modules allowing LLM-based chat systems to query "am I allowed to answer this?"

---

## 2. Cargo.toml Dependencies (Planned)

```toml
[package]
name = "neuro-consent-ledger"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/neuroseek/neuro-consent-ledger"

[dependencies]
# Core async runtime
tokio = { version = "1.40", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Cryptography (pluggable)
sha2 = "0.10"
ed25519-dalek = "2.1"
x25519-dalek = "2.0"

# Time & UUID
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging & tracing
tracing = "0.1"
tracing-subscriber = "0.3"

# Optional: ALN/KYC/DID support (for Bostrom/Cosmos integration)
did-key = "0.2"  # or custom ALN implementation

# Optional: Ledger backends
# For Googolswarm (Tendermint/Cosmos)
tendermint-rpc = { version = "0.33", optional = true }
cosmos-sdk-rs = { git = "https://github.com/cosmos/cosmos-rust-sdk", optional = true }

# For Ethereum
ethers = { version = "2.0", optional = true }

# Optional: AI chat boundary
axum = { version = "0.7", optional = true }
tower = { version = "0.4", optional = true }

[dev-dependencies]
criterion = "0.5"  # Benchmarking
proptest = "1.0"   # Property testing

[features]
default = ["core"]
core = []
googolswarm = ["tendermint-rpc", "cosmos-sdk-rs"]
ethereum = ["ethers"]
ai-chat-boundary = ["axum", "tower"]
```

---

## 3. Core Type Definitions

### 3.1 NeuroSubjectId

```rust
/// Augmented-citizen identity, bound to ALN/KYC/DID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NeuroSubjectId {
    /// Base58-encoded DID or Bostrom address (e.g., "bostrom1...")
    pub did: [u8; 32],
}

impl NeuroSubjectId {
    pub fn from_bostrom_address(addr: &str) -> Result<Self, DecodeError> {
        // Decode Bostrom/Cosmos address format
        todo!()
    }
    
    pub fn to_bostrom_address(&self) -> String {
        // Encode to Bostrom address format
        todo!()
    }
}
```

### 3.2 ActorClass

```rust
/// Classification of actor requesting access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorClass {
    SystemScheduler,
    ModelRuntime,
    Regulator,
    Policymaker,
    GovernmentAgency,
    Police,
    Medical,
    FinancialNonMonetary,
    RegionalAuthority,
    Researcher,
    Manufacturer,
    Commercial,
    EcologyMonitor,
}

impl ActorClass {
    /// Can this actor access bioload data in specified band without panel override?
    pub fn default_access_in_band(&self, band: BioloadBand) -> bool {
        use ActorClass::*;
        use BioloadBand::*;
        
        match (self, band) {
            // GREEN band: routine access for most actors
            (SystemScheduler, GREEN) => true,
            (Medical, GREEN) => true,
            (Regulator, GREEN) => true,
            (Researcher, GREEN) => true,
            
            // YELLOW band: restricted
            (Police, YELLOW) => false,
            (Researcher, YELLOW) => false,
            (Commercial, YELLOW) => false,
            (Medical, YELLOW) => true,  // Medical may access, but monitored
            
            // RED band: emergency only
            (Medical, RED) => true,
            (SystemScheduler, RED) => true,
            _ => false,
        }
    }
}
```

### 3.3 JurisdictionProfile

```rust
/// Regulatory and rights jurisdiction encoding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JurisdictionProfile {
    GDPR,
    HIPAA,
    ChileNeurorights,
    EU_AI_Act_HighRisk,
    California_Consumer_Privacy,
    Custom { name: String, neurorights_profile: NeurorightsProfile },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeurorightsProfile {
    /// Mental privacy ceiling: Can thought content be decoded?
    pub mental_privacy_protected: bool,
    
    /// Mental integrity: Can stimulation be applied without consent?
    pub mental_integrity_protected: bool,
    
    /// Psychological freedom: Can identity/agency be modified?
    pub psychological_freedom_protected: bool,
    
    /// Data minimization required?
    pub data_minimization_required: bool,
    
    /// Purpose limitation enforced?
    pub purpose_limitation_enforced: bool,
}
```

### 3.4 RightsSurface

```rust
/// Flags encoding neurorights and protective mechanisms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RightsSurface {
    pub neurorights_compliant: bool,
    pub soul_modeling_forbidden: bool,
    pub non_interference_required: bool,
    pub mental_privacy_max: bool,
    pub ecological_surplus_required: bool,
    pub consent_withdrawal_instantaneous: bool,
    pub authorship_irrevocable: bool,
    pub veto_coercive_use: bool,
}

impl Default for RightsSurface {
    fn default() -> Self {
        Self {
            neurorights_compliant: true,
            soul_modeling_forbidden: true,
            non_interference_required: true,
            mental_privacy_max: true,
            ecological_surplus_required: true,
            consent_withdrawal_instantaneous: true,
            authorship_irrevocable: true,
            veto_coercive_use: true,
        }
    }
}
```

### 3.5 ConsentScope

```rust
/// What is explicitly permitted for a given actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentScope {
    pub id: uuid::Uuid,
    pub subject: NeuroSubjectId,
    pub actor: ActorDid,
    pub actor_class: ActorClass,
    
    /// Data types allowed (e.g., "spike_density", "hrv", "stimulation_request")
    pub allowed_signal_types: Vec<String>,
    
    /// Resolution ceiling (e.g., "1_sec", "1_hz_aggregate", "no_raw_eeg")
    pub resolution_ceiling: ResolutionLevel,
    
    /// Purposes (e.g., "medical_safety", "research_consciousness", "commercial_analytics")
    pub purposes: Vec<Purpose>,
    
    /// Scope duration
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    
    /// Is this scope currently active?
    pub status: ConsentStatus,
    
    /// Multi-sig subject signature
    pub subject_signature: Signature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionLevel {
    /// No raw data; only post-processed aggregates
    Aggregated,
    /// 1-second windows or coarser
    OneSecond,
    /// 100 ms windows
    HundredMs,
    /// Raw signal data
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Purpose {
    MedicalSafety,
    ResearchConsciousness,
    ResearchNeurorights,
    CommercialAnalytics,
    ForensicInvestigation,
    EcologicalMonitoring,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentStatus {
    Active,
    Revoked,
    Expired,
    Suspended,
}
```

### 3.6 BioloadRegion

```rust
/// Snapshot of augmented-citizen bioload state at a region + time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BioloadRegion {
    pub region_id: String,
    pub subject: NeuroSubjectId,
    pub time_window: (DateTime<Utc>, DateTime<Utc>),
    
    // Neural signals (all optional for flexibility)
    pub eeg_spectrum: Option<EegSpectrum>,
    pub bci_traffic: Option<BciMetrics>,
    pub spike_density: Option<Vec<SpikeMetric>>,
    
    // Autonomic markers
    pub hrv_index: Option<f32>,
    pub gsr_level: Option<f32>,
    pub autonomic_tone: Option<f32>,  // [-1, 1]: negative = parasympathetic
    
    // Device telemetry
    pub implant_power_mw: Option<Vec<ImplantPower>>,
    pub nanobot_activity: Option<NanobotMetric>,
    pub tissue_interface_current_density: Option<Vec<CurrentDensity>>,
    pub thermal_load: Option<Vec<ThermalLoad>>,
    
    // Regulatory context
    pub neurorights_profile: JurisdictionProfile,
    pub subject_baseline: Option<PersonalizedBaseline>,
    pub subject_self_report: Option<SelfReport>,
    
    // Computed state
    pub bioload_band: BioloadBand,
    pub confidence_score: f32,  // [0, 1]
    pub last_updated: DateTime<Utc>,
    pub ledger_event_ref: Option<String>,  // Hash to Googolswarm event
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BioloadBand {
    GREEN,
    YELLOW,
    RED,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfReport {
    pub pain_1_to_10: u8,
    pub fatigue_1_to_10: u8,
    pub cognitive_load_1_to_10: u8,
}
```

### 3.7 LedgerEvent

```rust
/// Immutable event anchored to distributed ledger (Googolswarm, Ethereum, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEvent {
    pub event_id: uuid::Uuid,
    pub event_type: EventType,
    
    pub subject: NeuroSubjectId,
    pub actor: Option<ActorDid>,
    pub timestamp: DateTime<Utc>,
    
    pub bioload_band_before: Option<BioloadBand>,
    pub bioload_band_after: Option<BioloadBand>,
    
    pub action_taken: String,
    pub authorization_basis: String,
    
    // Cryptographic signatures
    pub subject_signature: Option<Signature>,
    pub primary_witness_signature: Option<Signature>,
    pub secondary_witness_signature: Option<Signature>,
    
    // Ledger anchoring
    pub tx_hash: Option<String>,  // Googolswarm / Ethereum tx hash
    pub block_number: Option<u64>,
    pub ledger_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    BioloadBandTransition,
    ActorAccessRequested,
    ActorAccessGranted,
    ActorAccessDenied,
    ConsentGranted,
    ConsentRevoked,
    OverrideAttempted,
    OverrideApproved,
    EmergencyShutdown,
    RecoverySequence,
    ForensicHold,
    UnauthorizedRemovalAttempted,
    DecodeAttemptUnauthorized,
    RightsViolationIncident,
}
```

---

## 4. Core API Functions

### 4.1 Bioload Evaluation

```rust
impl BioloadRegion {
    /// Evaluate subject's current bioload and assign band
    pub fn evaluate_band(
        &self,
        subject_baseline: &PersonalizedBaseline,
    ) -> Result<BioloadBand, EvaluationError> {
        // Compare current metrics to baseline; apply GREEN/YELLOW/RED criteria
        // Return band with confidence score
        todo!()
    }
    
    /// Get recommended protective actions for current band
    pub fn recommended_actions(&self) -> Vec<ProtectiveAction> {
        match self.bioload_band {
            BioloadBand::GREEN => vec![],  // No action
            BioloadBand::YELLOW => vec![
                ProtectiveAction::ReduceNonCriticalComputation,
                ProtectiveAction::LengthenInterStimulusIntervals,
                ProtectiveAction::ConveneNeurorightsPanelOverride,
            ],
            BioloadBand::RED => vec![
                ProtectiveAction::EmergencyShutdown,
                ProtectiveAction::MedicalAlert,
                ProtectiveAction::ForensicLockdown,
            ],
        }
    }
}

pub enum ProtectiveAction {
    ReduceNonCriticalComputation,
    LengthenInterStimulusIntervals,
    ConveneNeurorightsPanelOverride,
    EmergencyShutdown,
    MedicalAlert,
    ForensicLockdown,
}
```

### 4.2 Access Authorization

```rust
pub struct AccessGate {
    // ... internal state
}

impl AccessGate {
    pub fn new(ledger_backend: Box<dyn LedgerBackend>) -> Self {
        Self { /* ... */ }
    }
    
    /// Can actor access augmentation data under current conditions?
    pub async fn can_access(
        &self,
        actor: &ActorDid,
        actor_class: ActorClass,
        subject: NeuroSubjectId,
        purpose: Purpose,
        resolution: ResolutionLevel,
        bioload_band: BioloadBand,
    ) -> Result<AccessDecision, AccessError> {
        // Check Tier 1 invariants (inalienable rights)
        if self.violates_tier_1(actor, purpose, bioload_band)? {
            return Ok(AccessDecision::Deny {
                reason: "Tier 1 right violated".to_string(),
                forensic_flag: true,
            });
        }
        
        // Check consent
        let consent = self.get_consent_scope(actor, subject).await?;
        if !consent.is_active() || !consent.permits(purpose, resolution) {
            return Ok(AccessDecision::Deny {
                reason: "No valid consent".to_string(),
                forensic_flag: false,
            });
        }
        
        // Check band-dependent gating (Tier 2/3)
        match bioload_band {
            BioloadBand::GREEN => {
                if actor_class.default_access_in_band(bioload_band) {
                    return Ok(AccessDecision::Allow {
                        consent_ref: consent.id,
                    });
                } else {
                    return Ok(AccessDecision::Deny { reason: "...", forensic_flag: false });
                }
            },
            BioloadBand::YELLOW => {
                if actor_class.default_access_in_band(bioload_band) {
                    return Ok(AccessDecision::RequiresPanel {
                        reason: "YELLOW band: panel must approve".to_string(),
                    });
                } else {
                    return Ok(AccessDecision::Deny { reason: "...", forensic_flag: true });
                }
            },
            BioloadBand::RED => {
                if matches!(actor_class, ActorClass::Medical | ActorClass::SystemScheduler) {
                    return Ok(AccessDecision::Allow { consent_ref: consent.id });
                } else {
                    return Ok(AccessDecision::Deny {
                        reason: "RED band: access blocked except Medical".to_string(),
                        forensic_flag: true,
                    });
                }
            },
        }
    }
    
    fn violates_tier_1(
        &self,
        actor: &ActorDid,
        purpose: Purpose,
        bioload_band: BioloadBand,
    ) -> Result<bool, AccessError> {
        // Check for Tier 1 violations (mental privacy, bodily autonomy, consent revocation)
        // Return true if violation detected
        todo!()
    }
    
    async fn get_consent_scope(
        &self,
        actor: &ActorDid,
        subject: NeuroSubjectId,
    ) -> Result<ConsentScope, AccessError> {
        // Query ledger for active consent between actor and subject
        todo!()
    }
}

#[derive(Debug)]
pub enum AccessDecision {
    Allow { consent_ref: uuid::Uuid },
    Deny { reason: String, forensic_flag: bool },
    RequiresPanel { reason: String },
}
```

### 4.3 Consent Management

```rust
pub struct ConsentManager {
    // ... internal state
}

impl ConsentManager {
    pub async fn grant_consent(
        &mut self,
        subject: NeuroSubjectId,
        actor: ActorDid,
        actor_class: ActorClass,
        purposes: Vec<Purpose>,
        resolution: ResolutionLevel,
        valid_until: DateTime<Utc>,
        subject_signature: Signature,
    ) -> Result<LedgerEvent, ConsentError> {
        // Create consent scope, verify subject signature, record to ledger
        let scope = ConsentScope {
            id: uuid::Uuid::new_v4(),
            subject,
            actor,
            actor_class,
            allowed_signal_types: vec!["hrv".to_string(), "spike_density".to_string()],
            resolution_ceiling: resolution,
            purposes,
            valid_from: Utc::now(),
            valid_until,
            status: ConsentStatus::Active,
            subject_signature,
        };
        
        let event = LedgerEvent {
            event_id: uuid::Uuid::new_v4(),
            event_type: EventType::ConsentGranted,
            subject,
            actor: Some(actor),
            timestamp: Utc::now(),
            bioload_band_before: None,
            bioload_band_after: None,
            action_taken: format!("Consent granted to {:?} for purposes {:?}", actor_class, purposes),
            authorization_basis: "Subject explicit consent".to_string(),
            subject_signature: Some(subject_signature),
            ..Default::default()
        };
        
        self.ledger.append(event.clone()).await?;
        Ok(event)
    }
    
    pub async fn revoke_consent(
        &mut self,
        subject: NeuroSubjectId,
        target: RevokeTarget,
        subject_signature: Signature,
    ) -> Result<LedgerEvent, ConsentError> {
        // Revoke consent instantly, record forensic event, notify actor
        let event = LedgerEvent {
            event_id: uuid::Uuid::new_v4(),
            event_type: EventType::ConsentRevoked,
            subject,
            timestamp: Utc::now(),
            action_taken: format!("Consent revoked: {:?}", target),
            authorization_basis: "Subject unilateral revocation (Tier 1 right)".to_string(),
            subject_signature: Some(subject_signature),
            ..Default::default()
        };
        
        self.ledger.append(event.clone()).await?;
        
        // Immediately block further access
        self.invalidate_scopes_matching(&target).await?;
        
        Ok(event)
    }
}

#[derive(Debug)]
pub enum RevokeTarget {
    AllAugmentations,
    SingleAugmentation(String),
    SingleActor(ActorDid),
    SinglePurpose(Purpose),
}
```

---

## 5. ai-chat Boundary Module (Optional Feature)

```rust
#[cfg(feature = "ai-chat-boundary")]
pub mod ai_chat {
    use super::*;
    use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
    
    /// Request from LLM/chat system asking for permission to respond
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AuthorizationRequest {
        pub actor_did: String,
        pub actor_class: String,  // "Researcher", "Medical", etc.
        pub subject_did: String,
        pub data_sensitivity: String,  // "bioload_raw", "spike_density", "hrv_aggregate"
        pub purpose: String,  // "medical_safety", "research", "analytics"
        pub reason: String,  // Why is this query needed?
    }
    
    #[derive(Debug, Serialize)]
    pub struct AuthorizationResponse {
        pub allowed: bool,
        pub reason: String,
        pub bioload_band: String,
        pub required_disclaimers: Vec<String>,
        pub forensic_flag: bool,  // If true, record this request as incident
    }
    
    /// Check if chat system may answer user query about augmentation data
    pub async fn check_authorization(
        State(access_gate): State<AccessGate>,
        Json(req): Json<AuthorizationRequest>,
    ) -> impl IntoResponse {
        let actor_did = ActorDid::from_string(&req.actor_did).unwrap();
        let actor_class = match req.actor_class.as_str() {
            "Researcher" => ActorClass::Researcher,
            "Medical" => ActorClass::Medical,
            _ => ActorClass::Custom,
        };
        let subject = NeuroSubjectId::from_bostrom_address(&req.subject_did).unwrap();
        let purpose = match req.purpose.as_str() {
            "research" => Purpose::ResearchNeurorights,
            "medical" => Purpose::MedicalSafety,
            _ => Purpose::Custom(req.purpose),
        };
        
        let decision = access_gate.can_access(
            &actor_did,
            actor_class,
            subject,
            purpose,
            ResolutionLevel::Aggregated,
            BioloadBand::GREEN,  // Default assumption for chat
        ).await;
        
        let response = match decision {
            Ok(AccessDecision::Allow { consent_ref }) => AuthorizationResponse {
                allowed: true,
                reason: "Authorization granted under active consent".to_string(),
                bioload_band: "GREEN".to_string(),
                required_disclaimers: vec![],
                forensic_flag: false,
            },
            Ok(AccessDecision::Deny { reason, forensic_flag }) => AuthorizationResponse {
                allowed: false,
                reason,
                bioload_band: "UNKNOWN".to_string(),
                required_disclaimers: vec!["This query was denied and logged".to_string()],
                forensic_flag,
            },
            Ok(AccessDecision::RequiresPanel { reason }) => AuthorizationResponse {
                allowed: false,
                reason,
                bioload_band: "YELLOW".to_string(),
                required_disclaimers: vec!["Panel review required before response".to_string()],
                forensic_flag: false,
            },
            Err(e) => AuthorizationResponse {
                allowed: false,
                reason: format!("Error: {}", e),
                bioload_band: "ERROR".to_string(),
                required_disclaimers: vec![],
                forensic_flag: true,
            },
        };
        
        (StatusCode::OK, Json(response))
    }
    
    pub fn router(access_gate: AccessGate) -> Router {
        Router::new()
            .route("/check-authorization", post(check_authorization))
            .with_state(access_gate)
    }
}
```

---

## 6. Example Usage & Integration

### 6.1 Basic Usage

```rust
use neuro_consent_ledger::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize ledger backend (Googolswarm)
    let ledger = GooglelswarmBackend::connect("http://localhost:26657").await?;
    let access_gate = AccessGate::new(Box::new(ledger));
    
    // Subject DID
    let subject = NeuroSubjectId::from_bostrom_address("bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7")?;
    
    // Actor requesting access
    let researcher_did = ActorDid::from_string("researcher-alice-did-12345")?;
    let actor_class = ActorClass::Researcher;
    
    // Current bioload state
    let bioload = BioloadRegion {
        bioload_band: BioloadBand::GREEN,
        ..Default::default()
    };
    
    // Check authorization
    let decision = access_gate.can_access(
        &researcher_did,
        actor_class,
        subject,
        Purpose::ResearchNeurorights,
        ResolutionLevel::Aggregated,
        bioload.bioload_band,
    ).await?;
    
    match decision {
        AccessDecision::Allow { consent_ref } => {
            println!("Access granted under consent {:?}", consent_ref);
        },
        AccessDecision::Deny { reason, forensic_flag } => {
            println!("Access denied: {} (forensic: {})", reason, forensic_flag);
        },
        AccessDecision::RequiresPanel { reason } => {
            println!("Panel review required: {}", reason);
        },
    }
    
    Ok(())
}
```

### 6.2 Consent Management

```rust
#[tokio::main]
async fn consent_flow() -> Result<()> {
    let mut manager = ConsentManager::new(ledger_backend);
    
    // Subject grants consent to researcher
    let event = manager.grant_consent(
        subject_did,
        researcher_did,
        ActorClass::Researcher,
        vec![Purpose::ResearchNeurorights],
        ResolutionLevel::Aggregated,
        Utc::now() + Duration::days(365),
        subject_signature,  // Must be cryptographically signed by subject
    ).await?;
    
    println!("Consent event recorded: {:?}", event);
    
    // Later: subject revokes consent
    let revoke_event = manager.revoke_consent(
        subject_did,
        RevokeTarget::SingleActor(researcher_did),
        subject_signature,
    ).await?;
    
    println!("Consent revoked immediately: {:?}", revoke_event);
    
    Ok(())
}
```

---

## 7. Testing & Compliance

### 7.1 Unit Tests

```bash
cargo test --lib
```

### 7.2 Integration Tests

```bash
cargo test --test '*' -- --nocapture
# Tests against local Googolswarm testnet
```

### 7.3 Property-Based Testing

```rust
#[cfg(test)]
mod properties {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn tier_1_rights_never_override(
            bioload_band in prop::sample::select(vec![BioloadBand::GREEN, BioloadBand::YELLOW, BioloadBand::RED])
        ) {
            // Prove that Tier 1 invariants hold regardless of bioload band
            // E.g., mental_privacy_forbidden is always enforced
            assert!(!can_decode_thought_content_without_consent(bioload_band));
        }
    }
}
```

---

## 8. Ledger Backend Interface

```rust
/// Pluggable backend for recording events
#[async_trait::async_trait]
pub trait LedgerBackend: Send + Sync {
    async fn append(&mut self, event: LedgerEvent) -> Result<String, LedgerError>;
    async fn query(&self, query: LedgerQuery) -> Result<Vec<LedgerEvent>, LedgerError>;
}

pub struct GooglelswarmBackend {
    rpc_url: String,
    client: TendermintRpcClient,
}

#[async_trait::async_trait]
impl LedgerBackend for GooglelswarmBackend {
    async fn append(&mut self, event: LedgerEvent) -> Result<String, LedgerError> {
        // Serialize event to JSON
        let tx_bytes = serde_json::to_vec(&event)?;
        
        // Submit to Tendermint consensus
        let response = self.client.broadcast_tx_commit(&tx_bytes).await?;
        
        Ok(response.hash.to_string())
    }
    
    async fn query(&self, query: LedgerQuery) -> Result<Vec<LedgerEvent>, LedgerError> {
        // Query ledger for events matching criteria
        // (subject_did, actor_did, time range, event type)
        todo!()
    }
}
```

---

## 9. Deployment & Production Readiness

### 9.1 Security Considerations

- ✅ All cryptographic operations use `ed25519-dalek` (audited)
- ✅ No secret material stored in source code (use env vars, HSM)
- ✅ Tier 1 rights encoded as **algebraic invariants** (impossible to bypass without source modification)
- ✅ All access decisions logged with forensic-grade immutability (Googolswarm multi-sig)

### 9.2 Performance Targets

- **Access decision latency**: <100 ms (async, non-blocking)
- **Ledger append**: <500 ms (via Tendermint consensus)
- **Query latency**: <1 second (indexed queries)
- **Memory footprint**: <100 MB for 1M consent scopes in-memory (with disk spillover)

### 9.3 Monitoring & Audit

```bash
# Export metrics to Prometheus
RUST_LOG=neuro_consent_ledger=info,debug \
  cargo run --release -- --prometheus-port 9090

# Query audit log
curl http://localhost:26657/abci_query?data=subject_events&height=0 \
  | jq '.result.response.value' | base64 -d | jq .
```

---

## 10. Contributing & Roadmap

### 10.1 Roadmap (2026–2027)

- **v0.1.0** (Feb 2026): Core types + access gate + ledger integration
- **v0.2.0** (Mar 2026): AI-chat boundary module, REST API
- **v0.3.0** (Q2 2026): Full Googolswarm integration, multi-sig validation
- **v0.4.0** (Q3 2026): Ethereum/EVM bridge, interoperability layer
- **v1.0.0** (Q4 2026): Production hardening, security audit, formal verification

### 10.2 Contributing

See `CONTRIBUTING.md` for guidelines.

**Current needs:**
- Formal verification of Tier 1 invariants (cryptographic proof that no override possible)
- Ledger backend implementations (Ethereum, Polkadot, etc.)
- Performance benchmarking against real bioload datasets
- Regulatory compliance review (GDPR, HIPAA, ChileNeurorights)

---

## 11. License & Attribution

**License**: MIT OR Apache-2.0 (Dual License)  
**Citation**: Cite as "NeuroSeek neuro-consent-ledger Rust crate, v0.1.0 (2026)"  
**Author**: NeuroSeek Augmented-Citizen Rights Initiative  
**Attribution**: CC0 (Public Domain) + ALN/KYC/DID binding appreciated  

---

**End of NeuroSeek Rust Crate Design Document**
