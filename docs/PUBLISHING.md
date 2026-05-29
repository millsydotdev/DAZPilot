# Publishing Guide

This guide covers two publishing tracks:

- Open-source publication on GitHub.
- Product submission to the Daz 3D Marketplace as a Published Artist.

## GitHub Publication

### Keep The Daz SDK Private

The Daz Studio C++ SDK is proprietary. Do not commit SDK headers, import libraries, generated plugin build output, or local SDK folders.

Confirm ignore rules cover the local SDK and build artifacts:

```text
# Daz Studio SDK
thirdparty/DAZStudio4.5+ SDK/
*.lib
*.exp

# Tauri build artifacts
src-tauri/target/
src-tauri/binaries/
```

### Contributor Setup

Contributors should install the Daz SDK through Daz Install Manager, then either:

- Place `DAZStudio4.5+ SDK` in the `thirdparty` directory.
- Set `DAZ_SDK_PATH` to the SDK include path.

### License Choice

Use a permissive license unless there is a specific commercial reason not to.

| License | Good fit when... |
| --- | --- |
| MIT | You want maximum reuse with minimal terms |
| Apache 2.0 | You want permissive reuse plus explicit patent language |

### Release Automation

Use GitHub Actions to build and attach Windows installers when a `v*` tag is pushed. See [Release Guide](RELEASE_GUIDE.md) for the current release flow.

## Daz 3D Marketplace

To sell or distribute through the official Daz 3D store, apply for the Published Artist program.

### Preliminary Application

Email `pa@daz3d.com` to request store registration.

Include:

- A short product description.
- 3 to 6 professional promotional images.
- JPEG promo images at `1000 x 1300` pixels when possible, with `500 x 650` as the minimum.
- A link to the public repository or product page if available.

Do not send final product packages until Daz asks for them.

### Application Email Template

```text
Subject: Published Artist Application - DazPilot

Dear Daz 3D Published Artist Team,

I would like to apply to become a Published Artist on the Daz 3D Store.

I have created DazPilot, a desktop application with a custom Daz Studio bridge plugin for AI-assisted scene control. It includes a local desktop UI, a bridge plugin (`plugins/daz3d-bridge/`), local asset and SDK indexing, and bridge commands for scene inspection, asset loading, pose application, viewport capture, and model import.

I have attached promotional interface previews and workflow images in the requested format.

Repository or product page:
[Your GitHub URL]

Best regards,
[Your Name]
[Your Contact Info / Website]
```

### Onboarding

If accepted, expect Daz to provide the current legal, tax, and submission requirements. The marketplace process and royalty terms can change, so confirm details directly with Daz before planning launch dates or revenue assumptions.

## Product Packaging

### Plugin Package

Build your own custom bridge plugin and package it so it lands in Daz Studio's application plugin directory.

Common destination:

```text
C:\Program Files\DAZ 3D\DAZStudio4\plugins\
```

### Content Package

If DazPilot ships Daz scripts, support metadata, or content-library assets, keep the package rooted like a normal Daz content archive:

```text
/Scripts/DazPilot/DazPilotCopilot.dsa
/Runtime/Support/DazPilot_Metadata.dsx
```

Common destination:

```text
C:\Users\[User]\Documents\My DAZ 3D Library\
```

## Release Checklist

1. Update versions in `package.json` and `src-tauri/tauri.conf.json`.
2. Run `npm run check`.
3. Build installers with `npm run tauri build`.
5. Prepare marketplace ZIP packages with the correct folder layouts.
6. Prepare promotional images and product copy.
7. Submit through the current Daz Published Artist process.
