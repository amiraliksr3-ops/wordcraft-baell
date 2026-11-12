# Releasing WordCraft (Baell) for Android

## The release process (GitHub-hosted builds)

Everything is built by GitHub Actions — nothing needs a local toolchain.

1. **Tag and push**:

   ```bash
   git tag v0.18.1-android.1
   git push origin v0.18.1-android.1
   ```

   The tag name pattern is `v*-android*`. Bump `versionName` in
   `android/app/build.gradle` (and the workspace version, when tracking a new upstream
   WordCraft release) in the same commit if the version changes.

2. **The workflow** (`.github/workflows/ci-android.yml`, `release` job) then:

   - builds `libwordcraft_android.so` for `arm64-v8a`, `armeabi-v7a` and `x86_64` with cargo-ndk;
   - assembles signed release APKs (one per ABI plus a universal one) and an AAB;
   - computes SHA-256 checksums;
   - creates a GitHub release named after the tag with all artifacts attached.

3. **Verify** the release on the repository's Releases page: the APKs and `SHA256SUMS.txt`
   should be attached, and the checksums should match (`sha256sum -c SHA256SUMS.txt` after
   downloading).

## Signing

Without signing secrets the workflow signs with a temporary key generated on the runner:
installs fine, but each release has a different key, so updating requires uninstalling
first. For a stable update identity, set the repository secrets described in
[docs/BUILD.md](BUILD.md#release-signing-optional) once; every later release is then signed
with the same key.

## Regular (non-release) builds

Every push to `main` runs the `build` job and uploads unsigned **debug** APKs as workflow
artifacts — use those to test a commit before tagging it.
