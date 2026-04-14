# Distribution

## iOS Distribution

### Option 1: TestFlight (Recommended for Initial Release)
- Requires Apple Developer Program membership ($99/year)
- Supports up to 10,000 external testers
- Builds distributed via TestFlight app
- 90-day expiry on builds

### Option 2: Ad Hoc Distribution
- Limited to 100 registered devices per year
- Requires each device's UDID to be registered
- Suitable for very small, closed communities

### Option 3: Enterprise Distribution
- Requires Apple Developer Enterprise Program ($299/year)
- Requires D-U-N-S number and organizational eligibility
- Unlimited internal distribution
- **Not for public distribution**

## Android Distribution

### Direct APK Distribution (Recommended)
- Signed APK hosted on private HTTPS server
- Users enable "Install from unknown sources"
- No Google Play Store involvement (avoids censorship risk)
- In-app update checker for version management

### Alternative: F-Droid
- Open-source app repository
- Reproducible builds requirement
- Good for privacy-focused audience

## Release Signing

- **iOS:** Managed by Apple via provisioning profiles
- **Android:** Release keystore stored in hardware security module
  - Minimum: Encrypted keystore with multi-party access
  - Ideal: Cloud KMS (AWS/GCP) with audit logging
  - **NEVER** commit the keystore to version control

## Build Reproducibility

To verify a release build matches the source code:

```bash
# Clone the exact tagged version
git clone --branch v0.1.0 https://github.com/bitstack852/voryn.git
cd voryn

# Build with Docker for reproducibility
docker build -t voryn-builder .
docker run voryn-builder scripts/build-release-android.sh

# Compare SHA-256 hash with published hash
sha256sum build/voryn-release.apk
```

## In-App Update Mechanism

The app checks for updates on launch by querying a version endpoint:

```
GET https://updates.voryn.app/version.json
{
  "latest": "0.2.0",
  "minimum": "0.1.0",
  "android_url": "https://updates.voryn.app/releases/voryn-0.2.0.apk",
  "ios_url": "itms-beta://testflight.apple.com/...",
  "changelog": "Bug fixes and security improvements"
}
```

Users are prompted but never forced to update (except for critical security patches where `minimum` is bumped).
