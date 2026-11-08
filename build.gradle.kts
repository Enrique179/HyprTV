plugins {
    id("com.android.application") version "9.4.1" apply false
}

// Las salidas de compilación van fuera de OneDrive: si no, se sincronizan miles de archivos que cambian en cada build.
val outRoot = File(System.getenv("LOCALAPPDATA") ?: System.getProperty("java.io.tmpdir"), "hyprtv-build")
allprojects {
    layout.buildDirectory.set(File(outRoot, project.name))
}
