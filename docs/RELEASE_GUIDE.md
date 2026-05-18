# DazPilot Release Guide

DazPilot uses a highly optimized GitHub Actions workflow to automate the building, packaging, and publishing of Windows desktop installers (`.msi` and `.exe`) directly to GitHub Releases.

This guide explains how the release pipeline works, how to trigger it, and how to configure GitHub settings to support code signing and publishing.

---

## 🚀 How to Trigger a Release

The release pipeline is completely automated and triggers based on **Git Version Tags**.

### Step 1: Update App Version
Before tagging, ensure you increment the version number in both:
1. [package.json](file:///e:/DazAI/package.json) (line 4): `"version": "0.1.0"`
2. [src-tauri/tauri.conf.json](file:///e:/DazAI/src-tauri/tauri.conf.json) (line 5): `"version": "0.1.0"`

### Step 2: Push a Version Tag
To trigger the build, tag your main commit with the version number (prefixed with `v`) and push it to GitHub:

```bash
# 1. Commit all your changes (including package version updates)
git add .
git commit -m "chore: bump version to v0.1.0"

# 2. Add the Git tag
git tag v0.1.0

# 3. Push the commit and the tag to GitHub
git push origin main
git push origin v0.1.0
```

Once pushed, GitHub Actions will automatically start the **Publish Release** job.

---

## ⚙️ Required GitHub Repository Configuration

To enable the release pipeline to run successfully, you need to configure two things in your GitHub repository:

### 1. Enable Workflow Write Permissions
By default, GitHub Action tokens have read-only permissions, which will prevent the action from creating a release and uploading assets.

1. Go to your repository on GitHub.
2. Click on **Settings** (top tab) -> **Actions** -> **General** (left sidebar).
3. Scroll down to the **Workflow permissions** section.
4. Select **Read and write permissions**.
5. Click **Save**.

### 2. Configure Action Secrets (For Installer Signing)
Windows displays a "SmartScreen" warning if installers are unsigned. To sign your Tauri application for production releases, you must add your code-signing certificate credentials to GitHub.

1. In your repository on GitHub, navigate to **Settings** -> **Secrets and variables** -> **Actions**.
2. Create a new **Repository Secret** named `TAURI_SIGNING_PRIVATE_KEY` containing your Tauri private key.
3. Create a second secret named `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` containing the key's password.
4. Ensure the environment variables are uncommented in [.github/workflows/release.yml](file:///e:/DazAI/.github/workflows/release.yml).

---

## 🛠️ Managing Pre-Built C++ Bridge DLLs

Because the **DAZ Studio 4.5+ SDK** is proprietary and has licensing restrictions, **it cannot be committed to Git or compiled directly in the GitHub runner**. 

To build Tauri installers, we rely on pre-compiled DLLs placed inside [src-tauri/resources/](file:///e:/DazAI/src-tauri/resources/).

### How to update the pre-built DLLs:
1. Make your C++ changes in the [plugins/daz3d-bridge/](file:///e:/DazAI/plugins/daz3d-bridge/) folder.
2. Build the plugin locally using the CMake release profile:
   ```bash
   npm run plugin:rebuild
   ```
3. The build script automatically copies the newly compiled DLLs into the Tauri resources folder:
   - `src-tauri/resources/DazPilotBridge.dll`
   - `src-tauri/resources/VibeBridgePlugin.dll`
4. Stage, commit, and push these pre-built DLL files. The `.gitignore` has been custom-tailored to allow tracking of DLLs specifically under the `src-tauri/resources` directory!

---

## 📦 What the Pipeline Does
Every time a version tag is pushed, the runner:
1. Verifies that all required DLLs and the `llama-server.exe` are present.
2. Caches and installs npm & Rust packages for lightning-fast subsequent builds.
3. Compiles the TypeScript/Vite frontend.
4. Bundles all resources into the compiled Rust binary using Tauri CLI.
5. Generates the final Windows MSI installer.
6. Uploads the MSI to a new **draft release** on GitHub, ready for you to publish to the public!
