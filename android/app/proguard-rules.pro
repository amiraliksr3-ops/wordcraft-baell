# ProGuard / R8 rules for the WordCraft (Baell) Android host.
#
# The activity is a NativeActivity with no Java or Kotlin code, and release builds have
# minification off, so there is little to keep. These rules only make sure nothing that does
# exist gets renamed while the native library is in the APK.

# There is no code to mangle; keep the standard attributes for readable stack traces.
-keepattributes SourceFile,LineNumberTable
