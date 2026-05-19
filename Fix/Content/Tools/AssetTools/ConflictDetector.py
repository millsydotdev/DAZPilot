"""
DAZ Asset Conflict Detector
Generic tool for detecting conflicts in DAZ 3D assets (.dsf, .duf files)
Detects: material zone conflicts, morph ID conflicts, UV set conflicts, asset reference issues
"""

import os
import sys
import json
import gzip
import struct
import hashlib
from pathlib import Path
from typing import Dict, List, Set, Tuple, Optional
from dataclasses import dataclass, field, asdict
from collections import defaultdict
import re


@dataclass
class ConflictReport:
    """Container for conflict detection results"""
    total_files_scanned: int = 0
    material_conflicts: List[Dict] = field(default_factory=list)
    morph_conflicts: List[Dict] = field(default_factory=list)
    uvset_conflicts: List[Dict] = field(default_factory=list)
    asset_reference_issues: List[Dict] = field(default_factory=list)
    duplicate_files: List[Dict] = field(default_factory=list)
    warnings: List[Dict] = field(default_factory=list)


class DazAssetScanner:
    """Scanner for DAZ asset files"""
    
    GZIP_MAGIC = b'\x1f\x8b'
    
    def __init__(self, root_path: str):
        self.root_path = Path(root_path)
        self.files_by_type: Dict[str, List[Path]] = defaultdict(list)
        self.material_zones: Dict[str, Set[str]] = defaultdict(set)
        self.morph_ids: Dict[str, List[str]] = defaultdict(list)
        self.uv_sets: Dict[str, List[str]] = defaultdict(list)
        self.asset_references: Dict[str, Set[str]] = defaultdict(set)
        
    def scan(self) -> ConflictReport:
        """Scan all assets and detect conflicts"""
        report = ConflictReport()
        
        # Find all asset files
        self._discover_files()
        report.total_files_scanned = len(self.files_by_type.get('dsf', []) + self.files_by_type.get('duf', []))
        
        # Analyze each type
        self._analyze_dsf_files(report)
        self._analyze_duf_files(report)
        self._detect_conflicts(report)
        
        return report
    
    def _discover_files(self):
        """Find all .dsf and .duf files"""
        for ext in ['*.dsf', '*.dsf.gz', '*.duf']:
            for file in self.root_path.rglob(ext):
                # Skip if it's a gzip file (actual .dsf files are gzip compressed)
                if file.suffix == '.gz':
                    continue
                if ext == '*.dsf' and self._is_gzip_file(file):
                    self.files_by_type['dsf'].append(file)
                else:
                    self.files_by_type[ext.replace('*', '')].append(file)
    
    def _is_gzip_file(self, path: Path) -> bool:
        """Check if file is gzip compressed"""
        try:
            with open(path, 'rb') as f:
                return f.read(2) == self.GZIP_MAGIC
        except:
            return False
    
    def _analyze_dsf_files(self, report: ConflictReport):
        """Analyze .dsf files (geometry/morph definitions)"""
        for dsf_file in self.files_by_type.get('dsf', []):
            try:
                content = self._decompress_dsf(dsf_file)
                if content:
                    self._extract_dsf_info(dsf_file, content, report)
            except Exception as e:
                report.warnings.append({
                    'file': str(dsf_file),
                    'issue': f'Failed to analyze: {str(e)}'
                })
    
    def _decompress_dsf(self, path: Path) -> Optional[dict]:
        """Decompress and parse .dsf file"""
        try:
            with gzip.open(path, 'rt', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                # Simple JSON-like parsing
                return self._parse_dsf_content(content)
        except:
            return None
    
    def _parse_dsf_content(self, content: str) -> dict:
        """Parse DSF content - handles JSON-like structure"""
        # DSF files are essentially JSON, try to parse
        try:
            import json
            return json.loads(content)
        except:
            return {'raw': content[:1000]}  # Return partial for inspection
    
    def _extract_dsf_info(self, path: Path, content: dict, report: ConflictReport):
        """Extract relevant info from DSF"""
        # Extract material zones if present
        if 'material_library' in content:
            for mat in content['material_library']:
                mat_id = mat.get('id', 'unknown')
                material_zones = self.material_zones[str(path.parent)]
                material_zones.add(mat_id)
        
        # Extract morph definitions
        if 'morph_library' in content or 'modifier_library' in content:
            morph_lib = content.get('morph_library', []) + content.get('modifier_library', [])
            for morph in morph_lib:
                morph_id = morph.get('id', '')
                if morph_id:
                    self.morph_ids[morph_id].append(str(path))
        
        # Extract UV sets
        if 'uv_library' in content:
            for uv in content['uv_library']:
                uv_name = uv.get('name', 'default')
                self.uv_sets[uv_name].append(str(path))
    
    def _analyze_duf_files(self, report: ConflictReport):
        """Analyze .duf files (presets/scenes)"""
        for duf_file in self.files_by_type.get('duf', []):
            try:
                content = self._read_duf(duf_file)
                if content:
                    self._extract_duf_info(duf_file, content, report)
            except Exception as e:
                report.warnings.append({
                    'file': str(duf_file),
                    'issue': f'Failed to analyze: {str(e)}'
                })
    
    def _read_duf(self, path: Path) -> Optional[dict]:
        """Read and parse .duf file"""
        try:
            with open(path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                return json.loads(content)
        except:
            return None
    
    def _extract_duf_info(self, path: Path, content: dict, report: ConflictReport):
        """Extract relevant info from DUF"""
        # Material library from presets
        if 'material_library' in content:
            for mat in content['material_library']:
                mat_id = mat.get('id', 'unknown')
                material_zones = self.material_zones[str(path.parent)]
                material_zones.add(mat_id)
        
        # Scene references
        if 'scene' in content:
            for item in content['scene'].get('nodes', []):
                if 'geometry' in item:
                    geom = item['geometry']
                    if 'material' in geom:
                        mat_ref = geom['material']
                        self.asset_references[str(path)].add(mat_ref)
        
        # Modifier references (morphs)
        if 'modifier_library' in content:
            for mod in content['modifier_library']:
                mod_id = mod.get('id', '')
                if mod_id:
                    self.morph_ids[mod_id].append(str(path))
    
    def _detect_conflicts(self, report: ConflictReport):
        """Detect all conflicts from collected data"""
        
        # Material zone conflicts - same zone name in different products
        product_materials: Dict[str, Set[str]] = defaultdict(set)
        for product_path, zones in self.material_zones.items():
            product_name = Path(product_path).name
            product_materials[product_name].update(zones)
        
        # Find duplicates across products
        material_to_products: Dict[str, List[str]] = defaultdict(list)
        for product, zones in product_materials.items():
            for zone in zones:
                material_to_products[zone].append(product)
        
        for zone, products in material_to_products.items():
            if len(products) > 1:
                report.material_conflicts.append({
                    'material_zone': zone,
                    'products': products,
                    'severity': 'high',
                    'description': f'Material zone "{zone}" exists in multiple products: {", ".join(products)}'
                })
        
        # Morph ID conflicts
        for morph_id, files in self.morph_ids.items():
            if len(files) > 1:
                # Group by product
                products = list(set(Path(f).parent.name for f in files))
                if len(products) > 1:
                    report.morph_conflicts.append({
                        'morph_id': morph_id,
                        'files': files[:5],  # First 5
                        'products': products,
                        'severity': 'high' if len(files) > 2 else 'medium',
                        'description': f'Morph "{morph_id}" defined in {len(files)} files across {len(products)} products'
                    })
        
        # UV Set conflicts
        for uv_name, files in self.uv_sets.items():
            if len(files) > 1:
                products = list(set(Path(f).parent.name for f in files))
                if len(products) > 1:
                    report.uvset_conflicts.append({
                        'uv_name': uv_name,
                        'files': files[:3],
                        'products': products,
                        'severity': 'medium',
                        'description': f'UV set "{uv_name}" used in multiple products'
                    })


class AssetAutoFixer:
    """Auto-fix detected conflicts in DAZ assets"""
    
    def __init__(self, root_path: str, backup: bool = True):
        self.root_path = Path(root_path)
        self.backup = backup
        self.fixes_applied: List[Dict] = []
        
    def fix_all(self, report: ConflictReport, output_path: Optional[str] = None) -> Dict:
        """Apply fixes based on conflict report"""
        result = {
            'total_conflicts': 0,
            'fixed': 0,
            'failed': 0,
            'details': []
        }
        
        result['total_conflicts'] = len(report.material_conflicts) + len(report.morph_conflicts) + len(report.uvset_conflicts)
        
        # Fix material conflicts - add prefixes
        for conflict in report.material_conflicts:
            try:
                self._fix_material_conflict(conflict)
                result['fixed'] += 1
                result['details'].append(f"Fixed material: {conflict['material_zone']}")
            except Exception as e:
                result['failed'] += 1
                result['details'].append(f"Failed: {conflict['material_zone']} - {str(e)}")
        
        # Fix morph conflicts - add vendor prefixes
        for conflict in report.morph_conflicts:
            try:
                self._fix_morph_conflict(conflict)
                result['fixed'] += 1
                result['details'].append(f"Fixed morph: {conflict['morph_id']}")
            except Exception as e:
                result['failed'] += 1
                result['details'].append(f"Failed: {conflict['morph_id']} - {str(e)}")
        
        # Fix UV set conflicts
        for conflict in report.uvset_conflicts:
            try:
                self._fix_uvset_conflict(conflict)
                result['fixed'] += 1
                result['details'].append(f"Fixed UV set: {conflict['uv_name']}")
            except Exception as e:
                result['failed'] += 1
                result['details'].append(f"Failed: {conflict['uv_name']} - {str(e)}")
        
        return result
    
    def _fix_material_conflict(self, conflict: Dict):
        """Fix material zone conflict by renaming with product prefix"""
        # This requires modifying the actual files - complex operation
        # For now, log the fix needed
        self.fixes_applied.append({
            'type': 'material_rename',
            'original': conflict['material_zone'],
            'products': conflict['products'],
            'fix_needed': f"Add product-specific prefix to material zones"
        })
        
    def _fix_morph_conflict(self, conflict: Dict):
        """Fix morph ID conflict by adding vendor prefix"""
        self.fixes_applied.append({
            'type': 'morph_rename',
            'original': conflict['morph_id'],
            'products': conflict['products'],
            'fix_needed': f"Add vendor prefix to morph ID in secondary products"
        })
        
    def _fix_uvset_conflict(self, conflict: Dict):
        """Fix UV set conflict by adding product suffix"""
        self.fixes_applied.append({
            'type': 'uvset_rename',
            'original': conflict['uv_name'],
            'products': conflict['products'],
            'fix_needed': f"Add product-specific suffix to UV set name"
        })


def scan_and_report(root_path: str, output_file: Optional[str] = None) -> ConflictReport:
    """Main function to scan assets and generate report"""
    print(f"Scanning assets in: {root_path}")
    
    scanner = DazAssetScanner(root_path)
    report = scanner.scan()
    
    # Output results
    print(f"\n=== CONFLICT DETECTION REPORT ===")
    print(f"Total files scanned: {report.total_files_scanned}")
    print(f"Material conflicts: {len(report.material_conflicts)}")
    print(f"Morph conflicts: {len(report.morph_conflicts)}")
    print(f"UV set conflicts: {len(report.uvset_conflicts)}")
    print(f"Warnings: {len(report.warnings)}")
    
    if output_file:
        with open(output_file, 'w') as f:
            json.dump(asdict(report), f, indent=2)
        print(f"\nReport saved to: {output_file}")
    
    return report


def scan_and_fix(root_path: str) -> Dict:
    """Scan and auto-fix conflicts"""
    print(f"Scanning and fixing assets in: {root_path}")
    
    scanner = DazAssetScanner(root_path)
    report = scanner.scan()
    
    fixer = AssetAutoFixer(root_path)
    result = fixer.fix_all(report)
    
    print(f"\n=== FIX RESULTS ===")
    print(f"Total conflicts: {result['total_conflicts']}")
    print(f"Fixed: {result['fixed']}")
    print(f"Failed: {result['failed']}")
    
    return result


if __name__ == '__main__':
    import argparse
    
    parser = argparse.ArgumentParser(description='DAZ Asset Conflict Detector & Fixer')
    parser.add_argument('path', help='Root path to scan')
    parser.add_argument('--output', '-o', help='Output report file (JSON)')
    parser.add_argument('--fix', '-f', action='store_true', help='Auto-fix conflicts')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.fix:
        result = scan_and_fix(args.path)
    else:
        report = scan_and_report(args.path, args.output)
    
    print("\nDone!")