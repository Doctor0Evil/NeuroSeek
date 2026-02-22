package com.neuroseek.governance

import kotlinx.serialization.Serializable
import java.util.*

/**
 * NeuroSubjectId: ALN/KYC/DID-compatible identifier for augmented-citizen
 */
@Serializable
data class NeuroSubjectId(
    val alnAddress: String,  // e.g., "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7"
    val didUri: String,      // e.g., "did:bostrom:18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7"
    val kycVerified: Boolean,
    val biostretched_zone_profile: String  // e.g., "EU_GDPR_v3.1"
)

/**
 * ActorClass: Enum for programmatic and institutional actors
 */
@Serializable
enum class ActorClass {
    SYSTEM_SCHEDULER,
    MODEL_RUNTIME,
    REGULATOR,
    POLICYMAKER,
    GOVERNMENT_AGENCY,
    POLICE,
    MEDICAL,
    FINANCIAL_NON_MONETARY,
    REGIONAL_AUTHORITY,
    RESEARCHER,
    MANUFACTURER,
}

/**
 * BioloadBand: Physiological safety state
 */
@Serializable
enum class BioloadBand {
    GREEN,   // Safe routine operation
    YELLOW,  // Caution - biostretched-zone engaged
    RED,     // High risk - emergency protective response
}

/**
 * JurisdictionProfile: Regulatory context binding
 */
@Serializable
data class JurisdictionProfile(
    val name: String,  // "GDPR", "HIPAA", "ChileNeurorights", etc.
    val region: String,
    val neurorights_mandatory: Boolean,
    val data_residency_required: Boolean,
    val ecological_surplus_threshold: Double,  // % of energy budget for ecology
)

/**
 * RightsSurface: Bundle of flags from BiostretchedZone + neurorights profile
 */
@Serializable
data class RightsSurface(
    val neurorights_compliant: Boolean,
    val soul_modeling_forbidden: Boolean,
    val non_interference_required: Boolean,
    val mental_privacy_max: Boolean,
    val ecological_surplus_required: Boolean,
    val authorship_irrevocable: Boolean,
    val consent_withdrawal_instantaneous: Boolean,
)

/**
 * ConsentScope: Granular "what is allowed" surface
 */
@Serializable
data class ConsentScope(
    val signal_types: List<String>,  // e.g., ["eeg_alpha", "eeg_beta", "heart_rate"]
    val resolution: String,  // e.g., "downsampled_1Hz", "raw_10kHz"
    val purposes: List<String>,  // e.g., ["care", "research", "forensic"]
    val duration_seconds: Long,
    val revocable: Boolean = true,
    val sharing_allowed: Boolean = false,
)

/**
 * ControlMode: Which actor side has dominance
 */
@Serializable
enum class ControlMode {
    PROGRAMMATIC_CONTROL_ONLY,
    NEURO_CONSENT_REQUIRED,
    JOINT_CONTROL_WITH_PANEL,
    EMERGENCY_OVERRIDE,
    FORENSIC_FREEZE,
}

/**
 * BioloadRegion: Atomic unit for bioload decisions per region
 */
@Serializable
data class BioloadRegion(
    val region_id: String,
    val subject: NeuroSubjectId,
    val time_window: Pair<Long, Long>,  // (start_ms, end_ms)
    
    // Neural signals
    val eeg_bands: Map<String, Double>,  // e.g., {"alpha": 12.5, "beta": 8.2}
    val bci_traffic: TrafficMetric,
    val spike_density: Double,  // spikes per neuron per second
    
    // Autonomic markers
    val hrv_index: Double,  // std dev of RR intervals
    val gsr_level: Double,  // microsiemens
    val autonomic_tone: Double,  // [-1.0, 1.0]
    
    // Device/augmentation telemetry
    val implant_power_mw: Double,
    val nanobot_activity: SwarmMetric,
    val tissue_interface_current_density: Double,  // μA/mm²
    val thermal_load_mw_per_mm2: Double,
    
    // Environmental context
    val regional_carbon_intensity: Double,
    val neurorights_profile: RightsSurface,
    
    // State
    val bioload_band: BioloadBand,
    val confidence: Double,  // 0.0..1.0
    val timestamp: Long = System.currentTimeMillis(),
    val googolswarm_tx_ref: String? = null,  // Multi-sig ledger hash
)

@Serializable
data class TrafficMetric(
    val packets_per_sec: Int,
    val error_rate: Double,
)

@Serializable
data class SwarmMetric(
    val density_per_mm3: Double,
    val motion_rate_mm_per_sec: Double,
)

/**
 * LedgerEvent: Immutable record of consent/access changes
 */
@Serializable
data class LedgerEvent(
    val event_id: String = UUID.randomUUID().toString(),
    val region_session_key: String,
    val subject: NeuroSubjectId,
    val actor: ActorWithDid,
    val event_kind: LedgerEventKind,
    val jurisdiction_profile_before: JurisdictionProfile,
    val jurisdiction_profile_after: JurisdictionProfile,
    val consent_scope_before: ConsentScope,
    val consent_scope_after: ConsentScope,
    val control_mode_before: ControlMode,
    val control_mode_after: ControlMode,
    val bioload_band: BioloadBand,
    val neuroscore_band: String,  // e.g., "green", "yellow", "red"
    val googolswarm_tx_ref: String,  // Multi-sig attested hash
    val timestamp: Long = System.currentTimeMillis(),
    val revocable: Boolean = true,
)

@Serializable
data class ActorWithDid(
    val actor_class: ActorClass,
    val did_uri: String,
    val organization: String? = null,
    val role: String? = null,
)

@Serializable
enum class LedgerEventKind {
    SUBJECT_CONSENT_GRANTED,
    SUBJECT_CONSENT_REVOKED,
    SUBJECT_CONSENT_NARROWED,
    SUBJECT_CONSENT_EXPANDED,
    JURISDICTION_PROFILE_ATTACHED,
    JURISDICTION_PROFILE_SWITCHED,
    REGULATOR_OVERRIDE_REQUESTED,
    REGULATOR_OVERRIDE_APPROVED,
    REGULATOR_OVERRIDE_DENIED,
    EMERGENCY_MODE_ENTERED,
    EMERGENCY_MODE_EXITED,
    FORENSIC_HOLD_PLACED,
    FORENSIC_HOLD_RELEASED,
    SCHEDULER_AUTO_THROTTLE,
    SCHEDULER_AUTO_SHUTDOWN,
}

/**
 * Authorization Decision: Result of evaluating can_access()
 */
@Serializable
data class AuthorizationDecision(
    val allowed: Boolean,
    val requires_panel: Boolean,
    val reason: String,
    val veto_by_roles: List<String> = emptyList(),
    val recommended_actions: List<String> = emptyList(),
    val ledger_event: LedgerEvent? = null,
)
