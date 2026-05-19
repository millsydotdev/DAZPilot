# Millsy's Make Nude - Asset Tools

## Overview
This folder contains AI-powered tools for detecting and fixing DAZ 3D asset conflicts.

## Tools Included

### 1. ConflictDetector.py
Detects conflicts in DAZ assets including:
- Material zone conflicts (same zone name across different products)
- Morph ID conflicts (duplicate morph definitions)
- UV set name conflicts
- Asset reference issues

**Usage:**
```bash
python ConflictDetector.py "path/to/assets" --output report.json
python ConflictDetector.py "path/to/assets" --fix
```

### 2. ShellMaterialFixer.py
Specialized tool for fixing shell stacking conflicts:
- Identifies shell types (majora, minora, addons, breasts, nipples)
- Detects material zone conflicts between shells
- Adds product-specific prefixes to fix conflicts
- Creates fixed shell files

**Usage:**
```bash
python ShellMaterialFixer.py "path/to/shells" --fix
python ShellMaterialFixer.py "path/to/shells" --output fixed_output/
```

## Make Nude Presets

### Genesis 9
- **Location:** `../../People/Genesis 9/Make Nude/Millsy_Make_Nude_G9.dsf`
- **Features:**
  - Golden Palace Complete (genitalia + 3 geoshells + materials + UVs)
  - Realistic Breasts Complete (breasts + shell)
  - Nipple shapes (14 variations)
  - Areola shapes (10 variations)
  - All breast materials (nipples, areolas, scars, veins, stretch marks)
  - Genitalia shape controls (vagina open, anus open, labia, clitoris)

### Genesis 8 Female  
- **Location:** `../../People/Genesis 8 Female/Make Nude/Millsy_Make_Nude_G8F.dsf`
- **Features:**
  - Golden Palace v2 Complete (genitalia + shell + materials + UVs)
  - Realistic Breasts Complete (breasts + shell)
  - Nipple shapes (15 variations)
  - Areola shapes (10 variations)
  - All breast materials

## How It Works

### One-Click Setup
Load either `Millsy_Make_Nude_G9.dsf` or `Millsy_Make_Nude_G8F.dsf` in DAZ Studio and all assets load automatically with:
- Proper shell stacking order
- Correct material assignments
- UV sets applied
- All morphs available

### AI Conflict Detection
The tools scan your asset library and:
1. Find duplicate material zones across products
2. Identify morph conflicts
3. Detect UV set issues
4. Generate detailed reports
5. Optionally auto-fix issues

## Requirements
- Python 3.7+
- DAZ Studio 4.21+ (for using the presets)

## Notes
- The presets reference original asset files - keep the Fix folder structure intact
- All paths in presets use relative paths from Content folder
- Materials are Iray-optimized

## Author
Millsy - Custom nude asset system for Genesis 8/9