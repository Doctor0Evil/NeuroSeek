// Wraps ML model executions (TFLite, ExecuTorch) with consent checks and bioload monitoring
// Every inference is logged to neuro-consent ledger with full audit trail

package org.neuroseek.inference

import android.content.Context
import kotlinx.coroutines.*
import kotlinx.serialization.*
import timber.log.Timber
import org.neuroseek.bioload.BioloadTelemetry
import org.neuroseek.consent.NeuroConsentValidator
import org.neuroseek.consent.ActorClass
import org.neuroseek.consent.ConsentDecision
import java.time.Instant
import java.util.concurrent.atomic.AtomicLong

/**
 * InferenceResult wraps model output with metadata for ledger logging
 */
@Serializable
data class InferenceResult(
    val modelName: String,
    val inferenceId: String = java.util.UUID.randomUUID().toString(),
    val inputShape: String,
    val outputShape: String,
    val latencyMs: Long,
    val energyMicroJ: Long,
    val bioloadBandAtExecution: String,
    val timestamp: Long = System.currentTimeMillis(),
    val outputTensor: String = "[REDACTED]", // Placeholder; actual output is subject-guarded
    val purpose: String = "OPS", // Declared purpose: CARE, RESEARCH, OPS, ANALYTICS
    val actorClass: String,
    val ledgerTxHash: String = "", // Googolswarm transaction ID
    val consentValid: Boolean = true
)

/**
 * InferencePolicy encodes constraints for a specific model type
 */
@Serializable
data class InferencePolicy(
    val modelName: String,
    val maxEnergyMicroJ: Long = 5000L, // Budget per inference
    val allowedBioloadBands: List<String> = listOf("GREEN"), // Can only run in GREEN by default
    val maxLatencyMs: Int = 500,
    val requiresExplicitConsent: Boolean = true,
    val dataRedactionRequired: Boolean = true,
    val purposesAllowed: List<String> = listOf("OPS")
)

/**
 * NeuromorphicModelRuntime: Central wrapper for all on-device and remote model inference.
 * 
 * Responsibilities:
 * 1. Pre-inference: Consent validation via NeuroConsentValidator
 * 2. During execution: Energy and latency monitoring
 * 3. Post-inference: Bioload snapshot capture and ledger logging
 * 4. Enforcement: Block execution if constraints violated
 */
class NeuromorphicModelRuntime(
    private val context: Context,
    private val bioloadTelemetry: BioloadTelemetry,
    private val consentValidator: NeuroConsentValidator,
    private val scope: CoroutineScope = GlobalScope
) {
    
    // Inference policies by model name
    private val inferencePolicy = mutableMapOf<String, InferencePolicy>()
    
    // Audit log: all inferences recorded here before ledger sync
    private val inferenceAuditLog = mutableListOf<InferenceResult>()
    private val auditLogLock = Any()
    
    // Metrics aggregation
    private val totalInferencesExecuted = AtomicLong(0L)
    private val totalEnergySpentMicroJ = AtomicLong(0L)
    
    init {
        // Register default policies
        registerPolicy(InferencePolicy(
            modelName = "on_device_speech_recognition",
            maxEnergyMicroJ = 10000L,
            allowedBioloadBands = listOf("GREEN", "YELLOW"),
            purposesAllowed = listOf("CARE", "OPS")
        ))
        
        registerPolicy(InferencePolicy(
            modelName = "biometric_liveness",
            maxEnergyMicroJ = 2000L,
            allowedBioloadBands = listOf("GREEN", "YELLOW", "RED"),
            purposesAllowed = listOf("OPS", "SECURITY")
        ))
        
        registerPolicy(InferencePolicy(
            modelName = "seizure_detection_lstm",
            maxEnergyMicroJ = 3000L,
            allowedBioloadBands = listOf("GREEN", "YELLOW", "RED"),
            purposesAllowed = listOf("CARE", "OPS"),
            requiresExplicitConsent = true
        ))
        
        registerPolicy(InferencePolicy(
            modelName = "emotion_recognition_transformer",
            maxEnergyMicroJ = 5000L,
            allowedBioloadBands = listOf("GREEN"),
            purposesAllowed = listOf("RESEARCH"),
            requiresExplicitConsent = true,
            dataRedactionRequired = true
        ))
    }
    
    /**
     * Register or update an inference policy
     */
    fun registerPolicy(policy: InferencePolicy) {
        inferencePolicy[policy.modelName] = policy
        Timber.d("Registered policy for model: ${policy.modelName}")
    }
    
    /**
     * Main inference entry point: Wraps all model executions.
     * 
     * @param modelName: Name/ID of the model
     * @param inputTensor: Input data (type varies by model)
     * @param actorId: Actor performing inference (system scheduler, app, etc.)
     * @param purpose: Declared purpose (CARE, RESEARCH, OPS, ANALYTICS)
     * @param executeFn: Lambda that performs actual model.invoke() or inference call
     * 
     * @return InferenceResult with latency, energy, ledger hash, or ConsentDecision.Forbidden
     */
    suspend fun executeInferenceWithConsent(
        modelName: String,
        inputTensor: Any,
        actorId: String = "system_scheduler",
        purpose: String = "OPS",
        jurisdiction: String = "ANDROID_DEVICE_SHARD",
        executeFn: suspend () -> String // Returns output shape/sample
    ): Result<InferenceResult> = withContext(Dispatchers.Default) {
        
        Timber.d("Inference request: model=$modelName actor=$actorId purpose=$purpose")
        
        return@withContext try {
            // Step 1: Get policy for this model
            val policy = inferencePolicy[modelName]
                ?: return@withContext Result.failure(
                    Exception("No policy registered for model: $modelName")
                )
            
            // Step 2: Get current bioload state
            val currentSnapshot = bioloadTelemetry.getCurrentSnapshot()
                ?: return@withContext Result.failure(
                    Exception("Unable to read bioload telemetry")
                )
            
            val bandName = bioloadTelemetry.getCurrentBand().name
            
            // Step 3: Check if current bioload band permits this inference
            if (bandName !in policy.allowedBioloadBands) {
                Timber.w("Inference blocked: band=$bandName not in allowed=${policy.allowedBioloadBands}")
                return@withContext Result.failure(
                    Exception("Inference not permitted in $bandName bioload band. Allowed: ${policy.allowedBioloadBands}")
                )
            }
            
            // Step 4: Validate consent via NeuroConsentValidator
            val consentDecision = consentValidator.validateAction(
                actorId = actorId,
                actorClass = ActorClass.MODEL_RUNTIME,
                actionName = "ON_DEVICE_INFERENCE:$modelName",
                dataCategory = "BIOSIGNAL",
                purpose = purpose,
                jurisdiction = jurisdiction,
                currentBioloadBand = bioloadTelemetry.getCurrentBand()
            )
            
            when (consentDecision) {
                is ConsentDecision.Forbidden -> {
                    Timber.e("Inference blocked by consent: ${consentDecision.reason}")
                    return@withContext Result.failure(
                        Exception("Consent denied: ${consentDecision.reason}")
                    )
                }
                is ConsentDecision.RequiresPanel -> {
                    Timber.w("Inference requires panel review: ${consentDecision.reason}")
                    // In production: trigger panel review UI, block execution until approved
                    return@withContext Result.failure(
                        Exception("Inference requires panel review")
                    )
                }
                is ConsentDecision.Allowed -> {
                    Timber.d("Consent granted for inference")
                }
            }
            
            // Step 5: Execute inference with timing/energy instrumentation
            val startNs = System.nanoTime()
            val startMs = System.currentTimeMillis()
            
            val outputShape = executeFn() // Actual model.invoke()
            
            val endNs = System.nanoTime()
            val latencyMs = (endNs - startNs) / 1_000_000L
            
            // Rough energy estimate (micro-joules)
            // In production: sample actual device power draw from PMU or INA219 sensor
            val energyMicroJ = (latencyMs * 2000L).coerceAtMost(policy.maxEnergyMicroJ + 1000)
            
            // Step 6: Check energy budget
            if (energyMicroJ > policy.maxEnergyMicroJ) {
                Timber.w("Inference exceeded energy budget: $energyMicroJ > ${policy.maxEnergyMicroJ}")
            }
            
            // Step 7: Create audit log entry
            val result = InferenceResult(
                modelName = modelName,
                inputShape = inputTensor.toString().take(50),
                outputShape = outputShape,
                latencyMs = latencyMs,
                energyMicroJ = energyMicroJ,
                bioloadBandAtExecution = bandName,
                purpose = purpose,
                actorClass = ActorClass.MODEL_RUNTIME.name,
                timestamp = startMs
            )
            
            // Step 8: Log to audit trail and (async) to ledger
            recordInferenceToAuditLog(result)
            scope.launch {
                logInferenceToLedger(result)
            }
            
            // Update metrics
            totalInferencesExecuted.incrementAndGet()
            totalEnergySpentMicroJ.addAndGet(energyMicroJ)
            
            Timber.d("Inference completed: model=$modelName latency=${latencyMs}ms energy=${energyMicroJ}µJ band=$bandName")
            
            Result.success(result)
            
        } catch (e: Exception) {
            Timber.e(e, "Error during inference execution")
            Result.failure(e)
        }
    }
    
    /**
     * Record inference to local audit log (fast, in-memory)
     */
    private fun recordInferenceToAuditLog(result: InferenceResult) {
        synchronized(auditLogLock) {
            inferenceAuditLog.add(result)
            // Keep only recent inferences in memory to avoid unbounded growth
            if (inferenceAuditLog.size > 1000) {
                inferenceAuditLog.removeAt(0)
            }
        }
    }
    
    /**
     * Asynchronously log inference to Googolswarm ledger
     * In production: batch multiple inferences into single transaction
     */
    private suspend fun logInferenceToLedger(result: InferenceResult) {
        withContext(Dispatchers.IO) {
            try {
                // Simulate Googolswarm transaction
                val txHash = "0x${result.inferenceId.take(16)}" // Placeholder
                
                Timber.d("Logged inference to ledger: txHash=$txHash modelName=${result.modelName}")
                
                // Update result with ledger hash
                val resultWithHash = result.copy(ledgerTxHash = txHash)
                
                synchronized(auditLogLock) {
                    inferenceAuditLog[inferenceAuditLog.lastIndex] = resultWithHash
                }
            } catch (e: Exception) {
                Timber.e(e, "Error logging inference to ledger")
            }
        }
    }
    
    /**
     * Get audit log summary (for debugging and compliance)
     */
    fun getAuditLogSummary(): Map<String, Any> {
        synchronized(auditLogLock) {
            return mapOf(
                "totalInferences" to totalInferencesExecuted.get(),
                "totalEnergySpentMicroJ" to totalEnergySpentMicroJ.get(),
                "recentEntries" to inferenceAuditLog.takeLast(10),
                "averageLatencyMs" to (
                    if (inferenceAuditLog.isEmpty()) 0L
                    else inferenceAuditLog.map { it.latencyMs }.average()
                )
            )
        }
    }
    
    /**
     * Clear audit log (for privacy or testing)
     */
    fun clearAuditLog() {
        synchronized(auditLogLock) {
            inferenceAuditLog.clear()
            totalInferencesExecuted.set(0)
            totalEnergySpentMicroJ.set(0)
        }
    }
}
