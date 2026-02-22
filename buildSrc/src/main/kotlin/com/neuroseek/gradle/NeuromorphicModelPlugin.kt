package com.neuroseek.gradle

import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.tasks.Copy
import org.gradle.api.tasks.Exec
import org.gradle.kotlin.dsl.register

/**
 * NeuromorphicModelPlugin: Gradle plugin for converting and optimizing neuromorphic models
 * for Android deployment with bioload-aware optimization
 */
class NeuromorphicModelPlugin : Plugin<Project> {
    
    override fun apply(project: Project) {
        project.tasks.register<Exec>("convertNeuromorphicModel") {
            group = "neuromorphic"
            description = "Convert snnTorch/SpikingJelly model to TFLite or ExecuTorch format"
            
            commandLine = listOf(
                "python3",
                "tools/convert_neuromorphic_model.py",
                "--input", "models/neuromorphic_model.pt",
                "--output", "build/models/model.tflite",
                "--format", "tflite",
                "--quantize", "true",
                "--bioload-ceiling", "50mW"  // Energy ceiling for bioload compliance
            )
        }
        
        project.tasks.register<Copy>("copyOptimizedModels") {
            group = "neuromorphic"
            description = "Copy optimized models to assets"
            
            from("build/models/")
            into("src/main/assets/models/")
            
            dependsOn(project.tasks.named("convertNeuromorphicModel"))
        }
        
        project.tasks.register<Exec>("benchmarkBioload") {
            group = "neuromorphic"
            description = "Benchmark model performance and bioload metrics"
            
            commandLine = listOf(
                "python3",
                "tools/benchmark_bioload.py",
                "--model", "build/models/model.tflite",
                "--output", "build/reports/bioload-metrics.json",
                "--include-energy", "true",
                "--include-latency", "true",
                "--governance-profile", "eu_gdpr_v3.1"
            )
        }
    }
}
