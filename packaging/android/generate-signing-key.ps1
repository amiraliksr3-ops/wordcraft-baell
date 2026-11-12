<#
.SYNOPSIS
  Creates the Android release signing key WordCraft (Baell) uses, and the four GitHub secrets
  that go with it.

.DESCRIPTION
  The Android release workflow signs every APK and the AAB with one stable key, so a new version
  installs over the old one. This script creates that key once, outside the repository, and
  writes the values you paste into the repository's Actions secrets:

    Settings ▸ Secrets and variables ▸ Actions ▸ New repository secret

  Outputs, next to the keystore:
    baell-release.keystore         the key itself (never commit it)
    baell-release.secrets.txt      the four secret values (never commit or share it)

  The keystore has no recovery mechanism. Back it up somewhere safe; if it is lost, every future
  update needs a fresh install.

.EXAMPLE
  pwsh packaging/android/generate-signing-key.ps1

.EXAMPLE
  pwsh packaging/android/generate-signing-key.ps1 -Path D:\keys\baell.keystore
#>
param(
  [string] $Path = (Join-Path $HOME '.wordcraft/android/baell-release.keystore'),
  [string] $Alias = 'baell-release',
  [string] $Dname = 'CN=WordCraft (Baell), O=Baell',
  [switch] $Force
)

$ErrorActionPreference = 'Stop'

if ((Test-Path -LiteralPath $Path) -and -not $Force) {
  throw "Refusing to overwrite the existing keystore $Path (pass -Force only if you are sure). Back it up instead."
}

$keytool = Get-Command keytool -ErrorAction SilentlyContinue
if (-not $keytool) {
  throw 'keytool not found: install a JDK 17 (for example Eclipse Temurin) and put it on PATH.'
}

# 32 random characters from an unambiguous alphabet.
$alphabet = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789'
$rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
$bytes = [byte[]]::new(32)
$rng.GetBytes($bytes)
$password = -join ($bytes | ForEach-Object { $alphabet[$_ % $alphabet.Length] })

$dir = Split-Path -Parent $Path
New-Item -ItemType Directory -Force -Path $dir | Out-Null

# keytool writes its progress to stderr; capture both streams so a successful run stays quiet
# and a failure still shows what keytool said.
$previous = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$keytoolOutput = & $keytool.Source -genkeypair -noprompt `
  -keystore $Path `
  -storepass $password -keypass $password `
  -alias $Alias -keyalg RSA -keysize 4096 -validity 10000 `
  -dname $Dname 2>&1
$keytoolExit = $LASTEXITCODE
$ErrorActionPreference = $previous
if ($keytoolExit -ne 0) {
  $keytoolOutput | ForEach-Object { Write-Output $_ }
  throw "keytool failed with exit code $keytoolExit"
}

$b64 = [Convert]::ToBase64String([System.IO.File]::ReadAllBytes($Path))
$secretsPath = [System.IO.Path]::ChangeExtension($Path, '.secrets.txt')
@(
  'WordCraft (Baell) Android release secrets',
  '',
  'Add each line below under Settings > Secrets and variables > Actions > New repository secret.',
  'Keep this file and the keystore private. Never commit either one.',
  '',
  "ANDROID_KEYSTORE_B64=$b64",
  "ANDROID_KEYSTORE_PASSWORD=$password",
  "ANDROID_KEY_ALIAS=$Alias",
  "ANDROID_KEY_PASSWORD=$password"
) | Set-Content -LiteralPath $secretsPath -Encoding ascii

Write-Output "Created $Path"
Write-Output "Wrote the four secret values to $secretsPath"
Write-Output "Back up both files; the keystore cannot be recovered."
