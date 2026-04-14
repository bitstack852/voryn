# Environment Setup

## macOS (Primary Development Platform)

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Add mobile targets
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Add components
rustup component add rustfmt clippy
```

### 2. Install cargo-ndk (Android NDK integration)
```bash
cargo install cargo-ndk
```

### 3. Install Node.js & Yarn
```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
nvm install 20
npm install -g yarn
```

### 4. Install Xcode (iOS)
- Install Xcode 15+ from the Mac App Store
- Install command-line tools: `xcode-select --install`
- Install CocoaPods: `sudo gem install cocoapods`

### 5. Install Android Studio (Android)
- Download from https://developer.android.com/studio
- Install Android SDK, NDK, and build tools via SDK Manager
- Set environment variables:
```bash
export ANDROID_HOME=$HOME/Library/Android/sdk
export ANDROID_SDK_ROOT=$ANDROID_HOME
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/<version>
export PATH=$PATH:$ANDROID_HOME/emulator:$ANDROID_HOME/tools:$ANDROID_HOME/tools/bin:$ANDROID_HOME/platform-tools
```

### 6. Clone and Build
```bash
git clone https://github.com/bitstack852/voryn.git
cd voryn

# Install JS dependencies
yarn install

# Verify Rust compiles
cargo check --workspace

# Run Rust tests
cargo test --workspace

# Start React Native dev server
cd apps/mobile
npx react-native start

# Run on iOS simulator (separate terminal)
npx react-native run-ios

# Run on Android emulator (separate terminal)
npx react-native run-android
```

## Linux

Same as macOS except:
- Skip Xcode (iOS builds require macOS)
- Install Android Studio and NDK directly
- Use `cargo-ndk` for Android cross-compilation

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `ANDROID_HOME` | Android SDK root |
| `ANDROID_SDK_ROOT` | Same as ANDROID_HOME |
| `ANDROID_NDK_HOME` | Android NDK root |

## Troubleshooting

### Rust cross-compilation fails
- Ensure all targets are installed: `rustup target list --installed`
- For Android: verify `ANDROID_NDK_HOME` points to correct NDK version

### React Native build fails
- Clean and rebuild: `cd apps/mobile && yarn clean && yarn install`
- iOS: `cd ios && pod install --repo-update`
- Android: `cd android && ./gradlew clean`

### SQLCipher linking errors
- The `bundled-sqlcipher` feature compiles SQLCipher from source
- Ensure a C compiler is available: `gcc --version` or `clang --version`
