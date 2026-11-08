plugins {
    id("com.android.application")
}

android {
    namespace = "com.hyprtv"
    compileSdk = 37
    ndkVersion = "27.2.12479018"

    defaultConfig {
        applicationId = "com.hyprtv"
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
        // La TV es de 32 bits; arm64 por si se usa en otra.
        ndk { abiFilters += listOf("armeabi-v7a", "arm64-v8a") }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            // Build local: firmado con la clave de depuración para poder instalarlo por ADB.
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    // libhyprcore.so la deja scripts\build-rust.ps1 fuera de OneDrive.
    sourceSets["main"].jniLibs.srcDir(File(System.getenv("LOCALAPPDATA") ?: ".", "hyprtv-jni"))
}

kotlin {
    compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) }
}

// Sin dependencias a propósito: nada de AndroidX ni Compose. Es una TV con 1,5 GB de RAM.
