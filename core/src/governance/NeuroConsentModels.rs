// Kotlin equivalent types are derived from these canonical definitions

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
}

pub enum BioloadBand {
    GREEN,    // Full autonomic allowance
    YELLOW,   // Biostretched-zone engaged, conservation active
    RED,      // Emergency protective response
}

pub enum JurisdictionProfile {
    GDPR,
    HIPAA,
    ChileNeurorights,
    EU_AI_ACT_HIGH_RISK,
    SB_1223,
    CUSTOM(String),
}

pub enum ControlMode {
    ProgrammaticControlOnly,
    NeuroConsentRequired,
    JointControlWithPanel,
    EmergencyOverride,
    ForensicFreeze,
}
