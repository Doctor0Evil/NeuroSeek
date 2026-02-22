plugins {
    id("com.android.application")
    kotlin("android")
    kotlin("kapt")
    id("com.google.dagger.hilt.android")
}

android {
    compileSdk = 35
    
    defaultConfig {
        applicationId = "com.neuroseek.neuroconsentledger"
        minSdk = 28
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0-neuro-alpha"
        
        buildConfigField("String", "LEDGER_VERSION", "\"0.1.0\"")
        buildConfigField("String", "GOVERNANCE_SPEC", "\"spectral-v1\"")
        buildConfigField("boolean", "BIOLOAD_ENFORCEMENT_ENABLED", "true")
    }
    
    buildFeatures {
        compose = true
        dataBinding = false
    }
    
    composeOptions {
        kotlinCompilerExtensionVersion = "1.5.0"
    }
}

dependencies {
    // Jetpack Compose (v1.5+)
    implementation("androidx.compose.ui:ui:1.5.0")
    implementation("androidx.compose.material3:material3:1.1.0")
    implementation("androidx.compose.foundation:foundation:1.5.0")
    implementation("androidx.compose.runtime:runtime:1.5.0")
    
    // Kotlin Coroutines & Flow
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.1")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.1")
    
    // Android Jetpack
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.6.1")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.6.1")
    implementation("androidx.navigation:navigation-compose:2.7.1")
    
    // Hilt DI
    implementation("com.google.dagger:hilt-android:2.47")
    kapt("com.google.dagger:hilt-compiler:2.47")
    
    // Serialization (for Googolswarm ledger integration)
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.5.1")
    
    // Crypto / DID support (ALN integration)
    implementation("com.google.crypto.tink:tink-android:1.10.0")
    
    // ML Inference (TFLite / ExecuTorch integration)
    implementation("org.tensorflow:tensorflow-lite:2.13.0")
    
    // Testing
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.compose.ui:ui-test-junit4:1.5.0")
}
