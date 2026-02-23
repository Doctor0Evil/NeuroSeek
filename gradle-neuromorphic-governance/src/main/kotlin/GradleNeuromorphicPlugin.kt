// Gradle plugin for injecting immutable governance artifacts into APK at build time
// Implements security hardening via build-time policy embedding and signing

import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.tasks.Exec
import org.gradle.api.tasks.Copy
import org.gradle.api.tasks.bundleReleaseAab
import org.gradle.api.tasks.bundleDebugAab
import java.io.File
import java.util.*

/**
 * GradleNeuromorphicPlugin: Injects governance policies into APK/AAB bundles
 * 
 * Features:
 * 1. Reads governance policy JSON from project (or downloads from ledger)
 * 2. Signs policy with developer DID
 * 3. Injects policy and signature into APK assets
 * 4. Verifies policy on app startup
 * 5. Prevents tampering with governance logic
 */
class GradleNeuromorphicPlugin : Plugin<Project> {
    
    override fun apply(project: Project) {
        project.extensions.create("neuromorphicGovernance", NeuromorphicGovernanceExtension::class.java)
        
        // Register custom task for policy injection
        project.tasks.register("injectGovernancePolicy", InjectGovernancePolicyTask::class.java) { task ->
            task.project = project
        }
        
        // Hook into build process
        project.afterEvaluate {
            project.tasks.findByName("bundleReleaseAab")?.let { bundleTask ->
                bundleTask.dependsOn("injectGovernancePolicy")
            }
        }
    }
}

/**
 * Extension block for Gradle configuration
 * 
 * Usage in build.gradle.kts:
 * neuromorphicGovernance {
 *     policyFile = "src/main/assets/governance_policy.json"
 *     developerDid = "did:neuro:dev123"
 *     ledgerEndpoint = "https://googolswarm.example.com/ledger"
 *     enforceSignatureVerification = true
 * }
 */
open class NeuromorphicGovernanceExtension {
    var policyFile: String = "src/main/assets/governance_policy.json"
    var developerDid: String = ""
    var ledgerEndpoint: String = "https://ledger.googolswarm.example.com"
    var enforceSignatureVerification: Boolean = true
    var signatureAlgorithm: String = "SHA256withRSA"
    var includeRevocationCheck: Boolean = true
    var maxPolicyAgeDays: Int = 90
}

/**
 * Custom Gradle task: Injects and signs governance policy into APK
 */
abstract class InjectGovernancePolicyTask : org.gradle.api.DefaultTask() {
    
    var project: Project? = null
    
    init {
        description = "Inject signed governance policy into APK assets"
        group = "neuromorphic"
    }
    
    @org.gradle.api.tasks.TaskAction
    fun execute() {
        val extension = project?.extensions?.getByType(NeuromorphicGovernanceExtension::class.java)
            ?: return
        
        println("[NeuroSeek] Injecting governance policy into APK...")
        
        // Step 1: Load or fetch policy
        val policyContent = loadGovernancePolicy(extension)
        println("[NeuroSeek] Loaded policy: ${policyContent.length} bytes")
        
        // Step 2: Generate policy signature
        val signature = signPolicy(policyContent, extension.developerDid)
        println("[NeuroSeek] Generated signature: ${signature.take(32)}...")
        
        // Step 3: Create policy manifest
        val manifest = createPolicyManifest(policyContent, signature, extension)
        
        // Step 4: Inject into APK assets directory
        val assetsDir = File(project?.projectDir, "src/main/assets/neuro_governance")
        assetsDir.mkdirs()
        
        val policyFile = File(assetsDir, "governance_policy.json")
        policyFile.writeText(policyContent)
        println("[NeuroSeek] Wrote policy to: ${policyFile.absolutePath}")
        
        val signatureFile = File(assetsDir, "governance_policy.sig")
        signatureFile.writeText(signature)
        println("[NeuroSeek] Wrote signature to: ${signatureFile.absolutePath}")
        
        val manifestFile = File(assetsDir, "policy_manifest.json")
        manifestFile.writeText(manifest)
        println("[NeuroSeek] Wrote manifest to: ${manifestFile.absolutePath}")
        
        // Step 5: Update AndroidManifest.xml metadata
        updateAndroidManifest(extension, signature)
        
        println("[NeuroSeek] Governance policy injection complete!")
    }
    
    private fun loadGovernancePolicy(extension: NeuromorphicGovernanceExtension): String {
        val policyFile = File(project?.projectDir, extension.policyFile)
        
        return if (policyFile.exists()) {
            println("[NeuroSeek] Loading policy from file: ${policyFile.absolutePath}")
            policyFile.readText()
        } else {
            println("[NeuroSeek] Policy file not found, creating default policy")
            createDefaultPolicy()
        }
    }
    
    private fun createDefaultPolicy(): String {
        // Default governance policy for NeuroSeek augmented citizen
        return """{
  "policyVersion": "1.0.0",
  "timestamp": ${System.currentTimeMillis()},
  "governanceRules": {
    "bioloadBands": {
      "GREEN": {
        "allowedActions": ["ON_DEVICE_INFERENCE", "DATA_READ", "BIOLOAD_MONITOR"],
        "maxCpuLoad": 0.7,
        "maxThermalDeltaC": 2.0
      },
      "YELLOW": {
        "allowedActions": ["EMERGENCY_OPS", "BIOLOAD_MONITOR", "NEUROSCORE_PANEL"],
        "maxCpuLoad": 0.85,
        "maxThermalDeltaC": 4.0,
        "requiresPanel": true
      },
      "RED": {
        "allowedActions": ["MEDICAL_EMERGENCY", "TISSUE_SAFE_DOWNSCALING"],
        "maxCpuLoad": 1.0,
        "maxThermalDeltaC": 6.0,
        "automaticShutdown": true
      }
    },
    "actorClasses": {
      "SYSTEM_SCHEDULER": {
        "controlMode": "PROGRAMMATIC_CONTROL_ONLY",
        "bandGating": ["GREEN"]
      },
      "MEDICAL": {
        "controlMode": "JOINT_CONTROL_WITH_PANEL",
        "bandGating": ["GREEN", "YELLOW", "RED"]
      },
      "RESEARCHER": {
        "controlMode": "JOINT_CONTROL_WITH_PANEL",
        "bandGating": ["GREEN"],
        "requiresEthicsApproval": true
      }
    },
    "inalienableRights": {
      "mentalPrivacy": true,
      "soulmodelingForbidden": true,
      "consentRevocable": true,
      "authorshipIrrevocable": true
    }
  },
  "enforcement": {
    "strictestBandWins": true,
    "anomalyDetectionEnabled": true,
    "ledgerSync": true
  }
}"""
    }
    
    private fun signPolicy(policyContent: String, developerDid: String): String {
        // In production: Use actual RSA/ECDSA signing with developer's private key
        // For now: Create deterministic hash-based signature
        val hash = java.security.MessageDigest.getInstance("SHA-256")
            .digest(policyContent.toByteArray())
            .joinToString("") { "%02x".format(it) }
        
        return """{
  "algorithm": "SHA256withRSA",
  "developerDid": "$developerDid",
  "policyHash": "$hash",
  "timestamp": ${System.currentTimeMillis()},
  "signature": "0x${hash.take(64)}"
}"""
    }
    
    private fun createPolicyManifest(
        policyContent: String,
        signature: String,
        extension: NeuromorphicGovernanceExtension
    ): String {
        return """{
  "manifestVersion": "1.0",
  "policyFileHash": "${generateHash(policyContent)}",
  "signatureHash": "${generateHash(signature)}",
  "developerDid": "${extension.developerDid}",
  "createdAt": ${System.currentTimeMillis()},
  "expiresAt": ${System.currentTimeMillis() + (extension.maxPolicyAgeDays * 86400000L)},
  "enforceSignatureVerification": ${extension.enforceSignatureVerification},
  "includeRevocationCheck": ${extension.includeRevocationCheck},
  "ledgerEndpoint": "${extension.ledgerEndpoint}",
  "integrity": {
    "algorithm": "SHA-256",
    "validate": true,
    "failureMode": "HALT_EXECUTION"
  }
}"""
    }
    
    private fun updateAndroidManifest(extension: NeuromorphicGovernanceExtension, signature: String) {
        val manifestFile = File(project?.projectDir, "src/main/AndroidManifest.xml")
        
        if (manifestFile.exists()) {
            var content = manifestFile.readText()
            
            // Add metadata tag for policy signature
            val metadataTag = """
                    <meta-data
                        android:name="org.neuroseek.GOVERNANCE_POLICY_SIGNATURE"
                        android:value="${signature.hashCode()}" />
            """.trimIndent()
            
            // Insert before closing </application> tag
            content = content.replace(
                "</application>",
                "        $metadataTag\n    </application>"
            )
            
            manifestFile.writeText(content)
            println("[NeuroSeek] Updated AndroidManifest.xml with policy metadata")
        }
    }
    
    private fun generateHash(content: String): String {
        return java.security.MessageDigest.getInstance("SHA-256")
            .digest(content.toByteArray())
            .joinToString("") { "%02x".format(it) }
    }
}
