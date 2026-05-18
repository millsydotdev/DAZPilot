# DazPilot — Developer Publishing & Store Release Guide

This guide details how to publish the **DazPilot** application on GitHub as an open-source project and submit it to the **Daz 3D Marketplace** store as a Published Artist (PA).

---

## 1. GitHub Open-Source Guidelines

To share DazPilot on GitHub while complying with all software licenses and intellectual property laws:

### A. Keeping the Daz SDK Private (CRITICAL)
The Daz Studio C++ SDK is a proprietary product owned by Daz 3D. **You must NEVER upload the SDK header files or import libraries to a public GitHub repository.**
*   Ensure the following rules exist in your `.gitignore` file:
    ```text
    # Daz Studio SDK
    DAZStudio4.5+ SDK/
    *.lib
    *.exp
    *.dll
    
    # Tauri Build Artifacts
    src-tauri/target/
    src-tauri/binaries/
    ```
*   Instruct contributors to download the Daz SDK separately through the **Daz Install Manager (DIM)** and extract it to a local path (configured via `DAZ_SDK_PATH` environment variable or placed in the project root).

### B. Suggested License
Publish your code under a permissive open-source license, such as:
*   **MIT License**: Very permissive, great for wide developer adoption.
*   **Apache License 2.0**: Permissive, but adds clear clauses about patent grant rights and attribution requirements.

### C. CI/CD Release Automation
Use GitHub Actions to automatically compile and release Tauri desktop bundles (`.msi`, `.exe`) when you push a new tag:
```yaml
name: Release Tauri App
on:
  push:
    tags:
      - 'v*'

jobs:
  build-tauri:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: setup node
        uses: actions/setup-node@v4
        with:
          node-size: 20
      - name: install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: install dependencies
        run: npm install
      - name: build app
        run: npm run tauri build
      - name: upload binaries
        uses: softprops/action-gh-release@v1
        with:
          files: |
            src-tauri/target/release/bundle/msi/*.msi
            src-tauri/target/release/bundle/nsis/*.exe
```

---

## 2. Daz 3D Marketplace Publishing Handbook

To sell or distribute DazPilot directly on the official **Daz 3D Store**, you must apply for and join the **Published Artist (PA)** program.

### A. Preliminary Application Step
Send a professional email to **pa@daz3d.com** to request store registration.
*   **Do not send the actual product files yet.**
*   **Promotional Images (Promos)**: Attach 3 to 6 high-quality, professional render images showcasing the product in action.
    *   Dimensions: **1000 x 1300 pixels** (minimum 500 x 650).
    *   Format: JPEG.
*   **Product Description**: Describe DazPilot, its features, and its target audience.

#### Recommended Preliminary Email Template:
```text
Subject: Published Artist Application - DazPilot (AI Copilot & DazPilot Bridge Plugin)

Dear Daz 3D Published Artist Team,

I would like to apply to become a Published Artist on the Daz 3D Store. 

I have created "DazPilot" — a professional-grade desktop application that acts as an AI Copilot for Daz Studio. It includes:
1. A C++ Daz Studio Bridge Plugin (DazPilotBridge) for viewport click selections, material manipulation, timeline management, and dForce physical simulations.
2. A thread-safe DazScript GUI executor utilizing Qt's event loop to evaluate complex script payloads on the main thread safely.
3. An ultra-fast local asset browser driven by SQLite FTS5 for instant discovery.
4. Mathematical 3D spatial awareness (Left, Right, Behind, In Front, Above, Below) based on world-space bounding boxes to translate user prompts into natural coordinates.

I have attached high-resolution promotional interface previews and operational diagrams (1000x1300 JPG format) showcasing the viewport sync, timeline animation tools, and UI.

You can view the open-source repository skeleton here: [Your GitHub URL]

I look forward to your review and onboarding instructions.

Best regards,
[Your Name]
[Your Contact Info / Website]
```

### B. Standard Onboarding & Licensing
Once accepted, you will sign a standard **Non-Disclosure Agreement (NDA)** and Tax documents.
*   **Royalties**: Published Artists receive a base **50% net revenue royalty** on store sales, with higher tiers and performance bonuses.
*   **The PASS System**: You will get login credentials for the **Published Artist Submission System (PASS)**, where you can upload final builds, manage metadata, and submit products for testing.

---

## 3. Product Packaging Layout (DIM & Daz Central)

When packaging DazPilot for delivery via the **Daz Install Manager (DIM)**, your product zip files must preserve the exact relative folder structures so they unpack seamlessly.

### A. C++ Plugin Packaging (App Folder)
C++ plugins compiled as dynamic link libraries (`.dll`) must reside directly in Daz Studio's application directory.
*   **Zip Directory Structure**:
    ```text
    /plugins/DazPilotBridge.dll
    ```
*   When Daz Install Manager extracts the zip, it places the `.dll` directly into:
    `C:\Program Files\DAZ 3D\DAZStudio4\plugins\`

### B. Companion Scripts & Assets Packaging (Content Folder)
If your product includes companion Daz Scripts (`.dsa`, `.dse`) or standard figure contents:
*   **Zip Directory Structure**:
    ```text
    /Scripts/DazPilot/DazPilotCopilot.dsa
    /Runtime/Support/DazPilot_Metadata.dsx
    ```
*   These are extracted into the user's default Content Library:
    `C:\Users\[User]\Documents\My DAZ 3D Library\`

---

## 4. Operational Release Checklist

1.  **Tag release on GitHub**: Set version in `package.json` and `src-tauri/tauri.conf.json` (e.g. `1.0.0`).
2.  **Generate Release Build**: Run `npm run tauri build` to generate installers.
3.  **Prepare DIM Zips**: Create zip files following the packaging layouts.
4.  **Submit via PASS**: Upload the zips and 1000x1300 promo JPEGs to the PASS system for automated quality assurance testing.
