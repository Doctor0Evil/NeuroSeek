// Real-time bioload monitoring from device sensors and optional BCI inputs
// Implements NEUROSEEKBIOLOADSPEC GREEN/YELLOW/RED band computation

package org.neuroseek.bioload

import android.content.Context
import android.os.BatteryManager
import android.content.IntentFilter
import android.os.Build
import androidx.lifecycle.LiveData
import androidx.lifecycle.MutableLiveData
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.*
import timber.log.Timber
import java.time.Instant
import kotlin.math.sqrt

/**
 * BioloadBand represents the current cognitive and physiological load state.
 * GREEN: Safe operation, full autonomic allowance
 * YELLOW: Biostretched-zone, throttling recommended, neurorights panel convenes
 * RED: High risk, automatic protective response, tissue-safe mode only
 */
enum class BioloadBand {
    GREEN, YELLOW, RED
}

/**
 * Represents a snapshot of all bioload signals at a given moment.
 * Aligned with NEUROSEEKBIOLOADSPEC regional encoding.
 */
data class BioloadRegion(
    val timestamp: Instant,
    val subject: String, // NeuroSubjectId
    val region: String, // Jurisdiction shard
    
    // Neurophys signals (EEG-derived, if available via BCI interface)
    val eegBands: Map<String, Double>? = null, // alpha, beta, theta, delta, gamma power in uV
    val eegCoherence: Double? = null,
    val bciTraffic: BciMetrics? = null,
    
    // Autonomic markers
    val hrvIndex: Double? = null, // Heart rate variability std dev (ms)
    val gsrLevel: Double? = null, // Galvanic skin response (microsiemens)
    val autonomicTone: Double? = null, // -1 (sympathetic) to +1 (parasympathetic)
    
    // Device/augmentation telemetry
    val implantPowerMw: Double? = null, // Current draw at interface
    val thermalLoadMwPerMm2: Double? = null, // Power density at tissue interface
    val tissueInterfaceCurrentMicroA: Double? = null, // Current density safety limit: 10 µA/mm²
    val nanoswarmDensityPerMm3: Double? = null, // Active nanocybernetic particle density
    
    // Device-level metrics (always available)
    val deviceBatteryPercent: Int = 100,
    val deviceCpuLoad: Double = 0.0, // 0-1 scale
    val deviceTempC: Double = 25.0,
    val deviceThermoDeltaC: Double = 0.0, // Rise since baseline
    
    // Ecological context
    val regionalCarbonIntensity: Double = 400.0, // gCO₂/kWh
    val bioloadBand: BioloadBand = BioloadBand.GREEN,
    val confidence: Double = 0.8,
    val recommendedActions: List<String> = emptyList()
) {
    fun toMetrics(): String {
        return """
            BioloadRegion(
              timestamp=$timestamp,
              subject=$subject,
              band=$bioloadBand,
              battery=$deviceBatteryPercent%,
              cpuLoad=$deviceCpuLoad,
              deviceTemp=${deviceTempC}°C,
              thermalDelta=${deviceThermoDeltaC}°C,
              hrvIndex=$hrvIndex,
              gsr=$gsrLevel,
              autonomicTone=$autonomicTone,
              confidence=$confidence,
              actions=$recommendedActions
            )
        """.trimIndent()
    }
}

/**
 * BCI-specific metrics (if augmented citizen has active brain-computer interface)
 */
data class BciMetrics(
    val packetsPerSec: Int = 0,
    val errorRate: Double = 0.0, // 0-1 scale
    val spikeDensityPerNeuronPerSec: Double = 0.0,
    val decodingLatencyMs: Int = 0,
    val signalQuality: Double = 1.0 // 0-1, 1 is perfect
)

/**
 * BioloadTelemetry: Core bioload monitoring engine.
 * Reads from device sensors, computes GREEN/YELLOW/RED bands per jurisdiction,
 * emits events to NeuroConsent ledger and neuroscore panels.
 */
class BioloadTelemetry(
    private val context: Context,
    private val scope: CoroutineScope = GlobalScope
) {
    
    // Reactive streams for subscribers
    private val _bioloadState = MutableStateFlow<BioloadRegion?>(null)
    val bioloadState: StateFlow<BioloadRegion?> = _bioloadState.asStateFlow()
    
    private val _bandTransitions = MutableSharedFlow<Pair<BioloadBand, BioloadBand>>(replay = 1)
    val bandTransitions: SharedFlow<Pair<BioloadBand, BioloadBand>> = _bandTransitions.asSharedFlow()
    
    // Thresholds for GREEN/YELLOW/RED (customizable per jurisdiction)
    data class BandThresholds(
        // GREEN band limits
        val greenMaxCpuLoad: Double = 0.7,
        val greenMaxThermalDeltaC: Double = 2.0,
        val greenMinBattery: Int = 20,
        val greenMaxHrvVariation: Double = 1.5, // SD from baseline
        val greenMaxGsr: Double = 10.0,
        val greenMinAutonomicTone: Double = -0.3, // Parasympathetic bias
        
        // YELLOW band thresholds (biostretched-zone)
        val yellowMaxCpuLoad: Double = 0.85,
        val yellowMaxThermalDeltaC: Double = 4.0,
        val yellowMinBattery: Int = 10,
        val yellowMaxHrvVariation: Double = 2.5,
        val yellowMaxGsr: Double = 20.0,
        
        // RED band (emergency)
        val redMaxCpuLoad: Double = 1.0,
        val redMaxThermalDeltaC: Double = 6.0,
        val redMaxGsr: Double = 40.0
    )
    
    private val bandThresholds = BandThresholds()
    private var baselineHrv: Double = 50.0 // ms, personalized from enrollment
    private var baselineGsr: Double = 2.0 // microsiemens
    private var currentBand: BioloadBand = BioloadBand.GREEN
    
    init {
        // Start background bioload monitoring
        scope.launch {
            monitorBioloadContinuous()
        }
    }
    
    /**
     * Main monitoring loop: reads device state every 1 second
     */
    private suspend fun monitorBioloadContinuous() {
        while (isActive) {
            try {
                val snapshot = captureBioloadSnapshot()
                val band = evaluateBioloadBand(snapshot)
                
                if (band != currentBand) {
                    _bandTransitions.emit(Pair(currentBand, band))
                    currentBand = band
                    Timber.i("Bioload band transition: ${currentBand} → $band")
                }
                
                _bioloadState.emit(snapshot.copy(bioloadBand = band))
            } catch (e: Exception) {
                Timber.e(e, "Error capturing bioload snapshot")
            }
            delay(1000L) // 1 Hz polling
        }
    }
    
    /**
     * Capture current device state: battery, CPU, temperature, etc.
     */
    private fun captureBioloadSnapshot(): BioloadRegion {
        val battery = getBatteryStatus()
        val cpuLoad = getSystemCpuLoad()
        val deviceTemp = getDeviceTemperature()
        
        // Simulate autonomic metrics (would integrate with Android Health Connect in production)
        val hrvIndex = baselineHrv + (cpuLoad * 10.0) // Crude sim: HRV degrades with load
        val gsrLevel = baselineGsr + (cpuLoad * 5.0)
        val autonomicTone = 0.5 - (cpuLoad * 0.8) // More load = less parasympathetic
        
        return BioloadRegion(
            timestamp = Instant.now(),
            subject = android.os.Build.SERIAL, // Placeholder; use actual NeuroSubjectId
            region = "android_device_shard", // Placeholder jurisdiction
            hrvIndex = hrvIndex,
            gsrLevel = gsrLevel,
            autonomicTone = autonomicTone,
            deviceBatteryPercent = battery.level,
            deviceCpuLoad = cpuLoad,
            deviceTempC = deviceTemp,
            deviceThermoDeltaC = deviceTemp - 25.0, // Assume 25°C baseline
            confidence = 0.85
        )
    }
    
    /**
     * Evaluate which band a bioload snapshot belongs to.
     * GREEN < YELLOW < RED; strictest band wins.
     */
    private fun evaluateBioloadBand(snapshot: BioloadRegion): BioloadBand {
        // Check RED band first (highest priority)
        if (snapshot.deviceCpuLoad > bandThresholds.redMaxCpuLoad ||
            snapshot.deviceThermoDeltaC > bandThresholds.redMaxThermalDeltaC ||
            (snapshot.gsrLevel ?: 0.0) > bandThresholds.redMaxGsr
        ) {
            return BioloadBand.RED
        }
        
        // Check YELLOW band (biostretched-zone)
        if (snapshot.deviceCpuLoad > bandThresholds.yellowMaxCpuLoad ||
            snapshot.deviceThermoDeltaC > bandThresholds.yellowMaxThermalDeltaC ||
            snapshot.deviceBatteryPercent < bandThresholds.yellowMinBattery ||
            (snapshot.gsrLevel ?: 0.0) > bandThresholds.yellowMaxGsr
        ) {
            return BioloadBand.YELLOW
        }
        
        // Otherwise GREEN (safe operation)
        return BioloadBand.GREEN
    }
    
    /**
     * Get battery status from Android BatteryManager
     */
    private fun getBatteryStatus(): BatteryStatus {
        val intent = context.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
            ?: return BatteryStatus(level = 50, temperature = 300)
        
        val level = intent.getIntExtra(BatteryManager.EXTRA_LEVEL, -1)
        val temperature = intent.getIntExtra(BatteryManager.EXTRA_TEMPERATURE, 300)
        
        return BatteryStatus(level = level, temperature = temperature)
    }
    
    /**
     * Estimate CPU load from available Android APIs
     */
    private fun getSystemCpuLoad(): Double {
        return try {
            val runtime = Runtime.getRuntime()
            val totalMemory = runtime.totalMemory()
            val freeMemory = runtime.freeMemory()
            val usedMemory = totalMemory - freeMemory
            (usedMemory.toDouble() / totalMemory.toDouble()).coerceIn(0.0, 1.0)
        } catch (e: Exception) {
            0.5 // Fallback
        }
    }
    
    /**
     * Get device temperature (API 29+)
     */
    private fun getDeviceTemperature(): Double {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            try {
                val thermalService = context.getSystemService(Context.THERMAL_SERVICE)
                // Would use thermalService.currentThermalStatus in production
                25.0 // Placeholder
            } catch (e: Exception) {
                25.0
            }
        } else {
            25.0
        }
    }
    
    /**
     * Get current bioload state synchronously (for immediate checks)
     */
    fun getCurrentBand(): BioloadBand {
        return currentBand
    }
    
    /**
     * Get current bioload snapshot synchronously
     */
    fun getCurrentSnapshot(): BioloadRegion? {
        return _bioloadState.value
    }
    
    data class BatteryStatus(
        val level: Int,
        val temperature: Int
    )
}

// Extension: Flow-based band monitoring
fun BioloadTelemetry.bandChanges(): Flow<BioloadBand> {
    return bandTransitions.map { (_, newBand) -> newBand }
}
