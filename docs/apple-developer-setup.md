# Apple Developer Setup Guide

This guide covers setting up Apple Developer code signing and notarization for GoNhanh.

## Prerequisites

- Apple Developer Program membership ($99/year)
- macOS with Xcode installed
- Access to [developer.apple.com](https://developer.apple.com)

## Quick Reference

| Secret Name | Description | Where to Get |
|-------------|-------------|--------------|
| `APPLE_TEAM_ID` | 10-character Team ID | [Membership Details](https://developer.apple.com/account#MembershipDetailsCard) |
| `APPLE_SIGNING_IDENTITY` | Certificate name | `security find-identity -v -p codesigning` |
| `APPLE_CERTIFICATE_BASE64` | Base64-encoded .p12 file | Export from Keychain Access |
| `APPLE_CERTIFICATE_PASSWORD` | Password for .p12 file | Set when exporting |
| `APPLE_ID` | Your Apple ID email | Your Apple account email |
| `APPLE_APP_PASSWORD` | App-specific password | [appleid.apple.com](https://appleid.apple.com) |

## Step 1: Create Developer ID Certificate

1. Go to [Certificates, IDs & Profiles](https://developer.apple.com/account/resources/certificates/list)

2. Click **+** to create a new certificate

3. Select **Developer ID Application** (for distribution outside App Store)

4. Follow the instructions to create a Certificate Signing Request (CSR):
   - Open **Keychain Access** on your Mac
   - Menu: **Keychain Access > Certificate Assistant > Request a Certificate From a Certificate Authority**
   - Enter your email and select **Save to disk**

5. Upload the CSR and download the certificate

6. Double-click the downloaded `.cer` file to install it in Keychain

## Step 2: Export Certificate as .p12

1. Open **Keychain Access**

2. Find your certificate under **My Certificates**
   - Look for: `Developer ID Application: Your Name (TEAM_ID)`

3. Right-click > **Export "Developer ID Application..."**

4. Save as `.p12` format with a strong password

5. Convert to Base64:
   ```bash
   base64 -i certificate.p12 -o certificate-base64.txt
   ```

## Step 3: Get Your Team ID

1. Go to [Membership Details](https://developer.apple.com/account#MembershipDetailsCard)

2. Copy your **Team ID** (10-character alphanumeric code)

## Step 4: Create App-Specific Password

1. Go to [appleid.apple.com](https://appleid.apple.com)

2. Sign in and go to **App-Specific Passwords**

3. Click **+** to generate a new password

4. Name it `GoNhanh CI/CD` and save the password

## Step 5: Configure GitHub Secrets

Go to your repository: **Settings > Secrets and variables > Actions**

Add these secrets:

| Secret | Value |
|--------|-------|
| `APPLE_TEAM_ID` | Your 10-character Team ID |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Your Name (TEAM_ID)` |
| `APPLE_CERTIFICATE_BASE64` | Contents of `certificate-base64.txt` |
| `APPLE_CERTIFICATE_PASSWORD` | Password you set when exporting .p12 |
| `APPLE_ID` | Your Apple ID email |
| `APPLE_APP_PASSWORD` | App-specific password from Step 4 |

## Step 6: Verify Setup

### Local Build with Signing

```bash
# Check available certificates
security find-identity -v -p codesigning

# Build with Developer ID signing
./scripts/build-macos.sh --sign

# Build with signing + notarization
export APPLE_ID="your@email.com"
export APPLE_APP_PASSWORD="xxxx-xxxx-xxxx-xxxx"
export APPLE_TEAM_ID="XXXXXXXXXX"
./scripts/build-macos.sh --notarize
```

### GitHub Actions

After adding secrets, the release workflow will automatically:

1. Import your certificate
2. Sign the app with Developer ID
3. Submit to Apple for notarization
4. Staple the notarization ticket
5. Create DMG and publish release

## Troubleshooting

### Certificate Not Found

```
Error: No Developer ID Application certificate found in Keychain.
```

**Solution**: Make sure the certificate is installed in Keychain Access under "My Certificates"

```bash
security find-identity -v -p codesigning
```

### Notarization Failed

```
Error: Notarization requires APPLE_ID, APPLE_APP_PASSWORD, and APPLE_TEAM_ID
```

**Solution**: Set all required environment variables or GitHub secrets

### Invalid Signature

```
a sealed resource is missing or invalid
```

**Solution**: Use `--force --deep` flags when signing:

```bash
codesign --force --deep --sign "Developer ID Application: ..." app.app
```

### App-Specific Password Issues

If notarization fails with authentication errors:

1. Revoke and regenerate the app-specific password
2. Make sure 2FA is enabled on your Apple ID
3. Use the new password in `APPLE_APP_PASSWORD`

## File Changes Summary

The following files have been updated for Apple Developer support:

| File | Changes |
|------|---------|
| `platforms/macos/GoNhanh.entitlements` | Development entitlements (with debug flags) |
| `platforms/macos/GoNhanh.entitlements.production` | Production entitlements (hardened runtime) |
| `scripts/build-macos.sh` | Added `--sign` and `--notarize` options |
| `.github/workflows/release.yml` | Code signing + notarization pipeline |
| `.github/workflows/pre-release.yml` | Optional signing for PR builds |

## Fallback Behavior

If Apple Developer secrets are not configured:

- **release.yml**: Falls back to ad-hoc signing (current behavior)
- **pre-release.yml**: Uses ad-hoc signing by default, optional Developer ID with `sign: true`
- **build-macos.sh**: Uses ad-hoc signing without `--sign` flag

This ensures the build process works for contributors without Apple Developer access.
