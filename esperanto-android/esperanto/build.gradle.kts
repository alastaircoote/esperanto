import org.gradle.internal.impldep.org.apache.commons.io.output.ByteArrayOutputStream

object AndroidVersions {
    const val compileSdk = 29
    const val minSdk = 21
    const val targetSdk = 21
    const val versionCode = 1
    const val buildToolsVersion = "29.0.3"
    const val versionName = "1.0.0"
}

object Versions {
    const val kotlin = "1.3.72"
    //    const val appcompat = "1.1.0"
    const val junit = "4.12"
}


val JNI_BUILD_PATH = "${buildDir}/generated/jni-build"

plugins {
    id("com.android.library")
    kotlin("android")
    kotlin("android.extensions")
}

android {
    compileSdkVersion(AndroidVersions.compileSdk)
    buildToolsVersion = AndroidVersions.buildToolsVersion

    defaultConfig {
        minSdkVersion(AndroidVersions.minSdk)
        targetSdkVersion(AndroidVersions.targetSdk)
        versionCode = AndroidVersions.versionCode
        versionName = AndroidVersions.versionName

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFile("consumer-rules.pro")


    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDir(JNI_BUILD_PATH)
        }
    }

    buildTypes {
        getByName("release") {
            isMinifyEnabled = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }


}


val DOCKER_IMAGE_NAME = "esperanto-builder"

tasks.register("buildDockerImage", Exec::class).configure {

    outputs.upToDateWhen { exec {
        isIgnoreExitValue = true
        commandLine("docker", "inspect", "--type=image", DOCKER_IMAGE_NAME)
        standardOutput = ByteArrayOutputStream()
        errorOutput = ByteArrayOutputStream()
    }.exitValue == 0 }

    doLast {
        workingDir("..")
        commandLine("docker", "build", "-t", DOCKER_IMAGE_NAME, ".")
    }
}

enum class ABIs {
    X86 {
        override val rustTarget = "i686-linux-android"
        override val androidABI = "x86"
    },
    X86_64 {
        override val rustTarget = "x86_64-linux-android"
        override val androidABI = "x86_64"
    },
    Arm {
        override val rustTarget = "armv7-linux-androideabi"
        override val androidABI = "arm"
    },
    Arm64 {
        override val rustTarget = "aarch64-linux-android"
        override val androidABI = "arm64"
    };

    abstract val rustTarget: String
    abstract val androidABI: String
}

var activeABIs:List<ABIs> = listOf()
var releaseBuild = false

tasks.register("doNativeDebugBuild").configure {
    this.finalizedBy("buildNativeLibrary")
    doFirst {
        logger.error("Setting debug ABIs")
        activeABIs = listOf(ABIs.X86)
    }
//    doLast {

//    }
}

tasks.register("doNativeReleaseBuild").configure {
    this.finalizedBy("buildNativeLibrary")

    doFirst {
        logger.error("Setting release ABIs")
        activeABIs = listOf(ABIs.X86, ABIs.X86_64, ABIs.Arm, ABIs.Arm64)
        releaseBuild = true
    }
}


tasks.whenTaskAdded {
    if (this.name == "mergeReleaseJniLibFolders") {
        this.dependsOn("doNativeReleaseBuild")
    } else if (this.name == "mergeDebugJniLibFolders") {
        this.dependsOn("doNativeDebugBuild")
    }
}



//open class NativeLibraryBuild : DefaultTask() {
//    lateinit var buildType: String
//}

tasks.register("cleanNativeLibrary").configure {

}


tasks.register<Copy>("copyNativeLibraries").configure {

    // Make sure we clear out any existing files so we don't accidentally bundle
    // architectures we don't want
    delete(JNI_BUILD_PATH)

    from("../rust-build/target")

    // only copy the actual library files
    include("**/*/libesperanto_jni.so")

    into(JNI_BUILD_PATH)

    // something about the path changes we make below means that Gradle thinks
    // it knows when things are up to date but it's wrong. So we say this task
    // always needs to run no matter what.
    outputs.upToDateWhen { false }

    // If we don't include this it creates the entire target directory structure,
    // which we don't want!
    includeEmptyDirs = false

    eachFile {

        // For each library file we want to check the ABI and build profile to make sure
        // it matches the library we're creating.

        // example path: aarch64-linux-android/debug/libesperanto_jni.so

        val pathSplit = this.path.split("/")
        val abi = activeABIs.find { it.rustTarget == pathSplit[0]}
        if (abi == null) {
            // not an active ABI, ignore
            this.exclude()
            return@eachFile
        }
        if (releaseBuild && pathSplit[1] == "debug" || !releaseBuild && pathSplit[1] == "release") {
            // not this build profile, ignore
            this.exclude()
            return@eachFile
        }
        this.relativePath = RelativePath(true,abi.androidABI, "libesperanto.so")
    }

}

tasks.register("buildNativeLibrary").configure {
    this.finalizedBy("copyNativeLibraries")
    val projectDirectory = project.file("../..").absolutePath
    val cacheDirectory = project.file("../rust-build/cargo-registry").absolutePath
    val targetDirectory = project.file("../rust-build/target").absolutePath

    outputs.upToDateWhen { false }

    doLast {
        activeABIs.forEach {
            val abi = it
            exec {

                val args = mutableListOf(
                    "docker", "run",
                    "-t",
                    "-v", "$projectDirectory:/code:ro",
                    "-v", "$targetDirectory:/target",
                    "-v", "$cacheDirectory:/rust/cargo/registry:delegated",
                    "-w", "/code/esperanto-jni",
                    DOCKER_IMAGE_NAME,
                    "--target", abi.rustTarget,
                    "--platform", "21",
                    "build",
                    "--target-dir", "/target"
                )
                if (releaseBuild) {
                    args.add("--release")
                }

                commandLine(*args.toTypedArray())
            }
        }
    }
}


dependencies {
    implementation(fileTree(mapOf("dir" to "libs", "include" to listOf("*.jar", "*.so"))))
    implementation("org.jetbrains.kotlin:kotlin-stdlib:${Versions.kotlin}")
    testImplementation("junit:junit:${Versions.junit}")
    androidTestImplementation("androidx.test.ext:junit:1.1.1")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.2.0")

}