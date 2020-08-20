// Top-level build file where you can add configuration options common to all sub-projects/modules.
buildscript {
    repositories {
        google()
        jcenter()
    }
    dependencies {
        val kotlinVersion = "1.3.72"
        classpath("com.android.tools.build:gradle:4.0.1")
        classpath(kotlin("gradle-plugin", kotlinVersion))
    }
}

allprojects {
    repositories {
        google()
        jcenter()
    }
}

tasks.register("clean").configure {
    delete(buildDir)
}