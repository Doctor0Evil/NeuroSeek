package com.neuroseek.governance

import android.util.Log
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOf

/**
 * NeuroConsentEvaluator: Core decision engine for bioload bands and authorization
 */
class NeuroConsentEvaluator {
    
    private val TAG = "NeuroConsentEval"
    
    /**
     * Evaluate bioload band based on signal metrics
     */
    fun evaluateBioloadBand(
        bioloadRegion: BioloadRegion,
        subjectBaseline: SubjectBaseline
    ): BioloadBand {
        val eegCoherence = calculateEegCoherence(bioloadRegion.eeg_bands, subjectBaseline.baseline_eeg)
        val autonomicTone = bioloadRegion.autonomic_tone
        val implantPowerRatio = bioloadRegion.implant_power_mw / subjectBaseline.implant_power_cap_mw
        
        // GREEN BAND CRITERIA
        if (eegCoherence > 0.85 &&
            bioloadRegion.bci_traffic.error_rate < 0.05 &&
            autonomicTone > subjectBaseline.hrv_40th_percentile &&
            implantPowerRatio < 0.80 &&
            bioloadRegion.tissue_interface_current_density < 10.0 &&
            bioloadRegion.thermal_load_mw_per_mm2 < 1.0
        ) {
            Log.d(TAG, "Band evaluated: GREEN")
            return BioloadBand.GREEN
        }
        
        // YELLOW BAND CRITERIA
        if (eegCoherence > 0.70 ||
            bioloadRegion.bci_traffic.error_rate > 0.05 && bioloadRegion.bci_traffic.error_rate < 0.10 ||
            autonomicTone < subjectBaseline.hrv_40th_percentile ||
            implantPowerRatio > 0.85 && implantPowerRatio < 0.95
        ) {
            Log.d(TAG, "Band evaluated: YELLOW")
            return BioloadBand.YELLOW
        }
        
        // RED BAND CRITERIA
        if (eegCoherence < 0.60 ||
            bioloadRegion.bci_traffic.error_rate > 0.15 ||
            autonomicTone < subjectBaseline.hrv_10th_percentile ||
            implantPowerRatio > 0.95 ||
            bioloadRegion.tissue_interface_current_density > 25.0 ||
            bioloadRegion.thermal_load_mw_per_mm2 > 2.5
        ) {
            Log.d(TAG, "Band evaluated: RED - EMERGENCY RESPONSE TRIGGERED")
            return BioloadBand.RED
        }
        
        return BioloadBand.GREEN
    }
    
    /**
     * Evaluate authorization decision: can actor perform action under given bioload band?
     */
    fun evaluateCanAccess(
        actor: ActorWithDid,
        bioloadBand: BioloadBand,
        jurisdiction: JurisdictionProfile,
        rights: RightsSurface,
        subject: NeuroSubjectId
    ): AuthorizationDecision {
        
        // TIER 1: INALIENABLE RIGHTS (NO OVERRIDE)
        if (rights.soul_modeling_forbidden) {
            return AuthorizationDecision(
                allowed = false,
                reason = "TIER_1_VIOLATION: Soul modeling forbidden by neurorights profile",
                veto_by_roles = listOf("SUBJECT_ONLY")
            )
        }
        
        // TIER 2: PROTECTED RIGHTS (YELLOW-BAND GATING)
        if (bioloadBand == BioloadBand.YELLOW && actor.actor_class != ActorClass.MEDICAL) {
            return AuthorizationDecision(
                allowed = false,
                requires_panel = true,
                reason = "TIER_2_PROTECTION: Yellow band active, panel required for non-medical actors",
                veto_by_roles = listOf("SUBJECT", "MEDICAL", "ETHICS_PANEL")
            )
        }
        
        // TIER 3: MANAGED RIGHTS (GREEN-BAND ROUTINE)
        if (bioloadBand == BioloadBand.GREEN) {
            val tierThreeAllowed = when (actor.actor_class) {
                ActorClass.REGULATOR -> true  // Can audit in green band
                ActorClass.MEDICAL -> true
                ActorClass.RESEARCHER -> false  // Researcher cannot access without explicit consent
                else -> false
            }
            
            if (!tierThreeAllowed && actor.actor_class == ActorClass.RESEARCHER) {
                return AuthorizationDecision(
                    allowed = false,
                    requires_panel = false,
                    reason = "TIER_3_MANAGED: Researchers cannot access without explicit subject consent",
                    veto_by_roles = listOf("SUBJECT", "ETHICS_COMMITTEE")
                )
            }
        }
        
        // RED BAND: ONLY MEDICAL + EMERGENCY
        if (bioloadBand == BioloadBand.RED) {
            val redBandAllowed = actor.actor_class == ActorClass.MEDICAL || 
                                 actor.actor_class == ActorClass.SYSTEM_SCHEDULER
            
            if (!redBandAllowed) {
                return AuthorizationDecision(
                    allowed = false,
                    requires_panel = false,
                    reason = "RED_BAND_LOCKDOWN: Only Medical and System Scheduler allowed",
                    veto_by_roles = listOf("EMERGENCY_ONLY")
                )
            }
        }
        
        // Default: ALLOWED with logging
        return AuthorizationDecision(
            allowed = true,
            requires_panel = false,
            reason = "AUTHORIZATION_GRANTED",
            recommended_actions = listOf("LOG_TO_LEDGER", "AUDIT_TRACE")
        )
    }
    
    private fun calculateEegCoherence(
        eegBands: Map<String, Double>,
        baselineEeg: Map<String, Double>
    ): Double {
        if (eegBands.isEmpty() || baselineEeg.isEmpty()) return 1.0
        
        var totalDeviation = 0.0
        eegBands.forEach { (band, value) ->
            val baseline = baselineEeg[band] ?: 0.0
            totalDeviation += kotlin.math.abs(value - baseline) / (baseline + 1e-6)
        }
        
        return kotlin.math.max(0.0, 1.0 - (totalDeviation / eegBands.size))
    }
    
    data class SubjectBaseline(
        val baseline_eeg: Map<String, Double>,
        val hrv_40th_percentile: Double,
        val hrv_10th_percentile: Double,
        val implant_power_cap_mw: Double,
    )
}
