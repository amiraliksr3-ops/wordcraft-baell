# Releasing WordCraft (Baell) for Android

## The release process (GitHub-hosted builds)

Everything is built by GitHub Actions — nothing needs a local toolchain.

1. **Tag and push**:

   ```bash
   git tag v0.18.1-android.1
   git push origin v0.18.1-android.1
   ```

   The tag pattern is `v<major>.<minor>.<patch>-android[.<build>]`, for example
   `v0.18.1-android.1`. The workflow derives versionName and versionCode from it (see
   [docs/BUILD.md](BUILD.md#release-version)); bump the workspace version when tracking a new
   upstream WordCraft release. A second release of the same upstream version needs a higher
   build number, such as `v0.18.1-android.2`, so Android accepts it as an update.

2. **The workflow** (`.github/workflows/ci-android.yml`, `release` job) then:

   - refuses to continue unless the four `ANDROID_*` signing secrets are set (see
     [docs/BUILD.md](BUILD.md#release-signing-required-for-releases));
   - builds `libwordcraft_android.so` for `arm64-v8a`, `armeabi-v7a` and `x86_64` with cargo-ndk;
   - assembles signed release APKs (one per ABI plus a universal one) and an AAB;
   - verifies each APK's signature with `apksigner`;
   - computes SHA-256 checksums;
   - creates a GitHub release named after the tag with all artifacts attached.

3. **Verify** the release on the repository's Releases page: the APKs and `SHA256SUMS.txt`
   should be attached, and the checksums should match (`sha256sum -c SHA256SUMS.txt` after
   downloading).

## Signing

The release job requires all four `ANDROID_*` secrets and stops before building without them.
Create them once with `pwsh packaging/android/generate-signing-key.ps1` and add the values under
`Settings ▸ Secrets and variables ▸ Actions`; see
[docs/BUILD.md](BUILD.md#release-signing-required-for-releases). Every release is then signed
with the same key, so Android installs it over the previous version. Keep the keystore backed up:
losing it means every future update needs a fresh install.

## Regular (non-release) builds

Every push to `main` runs the `build` job and uploads unsigned **debug** APKs as workflow
artifacts — use those to test a commit before tagging it.
