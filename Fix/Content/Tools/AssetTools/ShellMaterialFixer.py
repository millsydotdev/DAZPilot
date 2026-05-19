"""
DAZ Shell Material Fixer
Specialized tool for fixing shell stacking material conflicts
Handles: Geoshell material zones, UV sets, collision detection
"""

import os
import json
import gzip
import shutil
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, field
import re


@dataclass
class ShellInfo:
    """Information about a shell asset"""
    name: str
    path: Path
    type: str  # majora, minora, addons, breasts, nipples
    material_zones: List[str] = field(default_factory=list)
    uv_sets: List[str] = field(default_factory=list)
    parent_geometry: Optional[str] = None


class ShellMaterialFixer:
    """Fix shell material conflicts for proper stacking"""
    
    # Material zone mapping for different shell types
    SHELL_MATERIAL_PREFIXES = {
        'majora': 'GM_',     # Geoshell Majora
        'minora': 'GMin_',  # Geoshell Minora  
        'addons': 'GA_',    # Geoshell Addons
        'breasts': 'BR_',   # Breasts shell
        'nipples': 'NP_',   # Nipples shell
        'areola': 'AR_',    # Areola shell
    }
    
    # Standard material zones that conflict across shells
    CONFLICT_ZONES = [
        'Torso', 'Torso_Back', 'Vagina', 'Labia Minora', 'Labia Majora',
        'Urethra', 'Rectum', 'Anus', 'Clitoris', 'Breasts', 'Nipples', 'Areola'
    ]
    
    def __init__(self, root_path: str, create_backup: bool = True):
        self.root_path = Path(root_path)
        self.create_backup = create_backup
        self.shells: List[ShellInfo] = []
        self.fixes: List[Dict] = []
        
    def find_all_shells(self) -> List[ShellInfo]:
        """Find all shell geometry files in the path"""
        shells = []
        
        # Search for shell geometry files
        shell_patterns = [
            '*Shell*.dsf',
            '*shell*.dsf',
            '*Geoshell*.dsf',
            '*_Shell_*.dsf'
        ]
        
        for pattern in shell_patterns:
            for file in self.root_path.rglob(pattern):
                if self._is_gzip_file(file):
                    shell_info = self._analyze_shell(file)
                    if shell_info:
                        shells.append(shell_info)
        
        self.shells = shells
        return shells
    
    def _is_gzip_file(self, path: Path) -> bool:
        """Check if file is gzip compressed"""
        try:
            with open(path, 'rb') as f:
                return f.read(2) == b'\x1f\x8b'
        except:
            return False
    
    def _analyze_shell(self, file_path: Path) -> Optional[ShellInfo]:
        """Analyze a shell file and extract info"""
        try:
            with gzip.open(file_path, 'rt', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                data = json.loads(content)
            
            # Determine shell type from path
            path_str = str(file_path).lower()
            shell_type = self._identify_shell_type(path_str)
            
            # Extract material zones
            material_zones = []
            if 'material_library' in data:
                material_zones = [m.get('id', '') for m in data['material_library'] if m.get('id')]
            
            # Extract UV sets
            uv_sets = []
            if 'uv_library' in data:
                uv_sets = [uv.get('name', '') for uv in data['uv_library'] if uv.get('name')]
            
            return ShellInfo(
                name=file_path.stem,
                path=file_path,
                type=shell_type,
                material_zones=material_zones,
                uv_sets=uv_sets
            )
            
        except Exception as e:
            print(f"Warning: Could not analyze {file_path}: {e}")
            return None
    
    def _identify_shell_type(self, path: str) -> str:
        """Identify shell type from path"""
        if 'majora' in path:
            return 'majora'
        elif 'minora' in path:
            return 'minora'
        elif 'addons' in path or 'addon' in path:
            return 'addons'
        elif 'breast' in path and 'nipple' not in path:
            return 'breasts'
        elif 'nipple' in path:
            return 'nipples'
        elif 'areola' in path or 'areola' in path:
            return 'areola'
        else:
            return 'unknown'
    
    def detect_conflicts(self) -> Dict:
        """Detect material zone conflicts between shells"""
        conflicts = {
            'shell_pairs': [],
            'conflict_zones': [],
            'uv_conflicts': []
        }
        
        # Check for material zone conflicts
        zone_shells: Dict[str, List[str]] = {}
        
        for shell in self.shells:
            for zone in shell.material_zones:
                if zone not in zone_shells:
                    zone_shells[zone] = []
                zone_shells[zone].append(shell.name)
        
        for zone, shells in zone_shells.items():
            if len(shells) > 1 and zone in self.CONFLICT_ZONES:
                conflicts['conflict_zones'].append({
                    'zone': zone,
                    'shells': shells,
                    'severity': 'high' if len(shells) > 2 else 'medium'
                })
        
        # Check UV set conflicts
        uv_shells: Dict[str, List[str]] = {}
        for shell in self.shells:
            for uv in shell.uv_sets:
                if uv not in uv_shells:
                    uv_shells[uv] = []
                uv_shells[uv].append(shell.name)
        
        for uv, shells in uv_shells.items():
            if len(shells) > 1:
                conflicts['uv_conflicts'].append({
                    'uv_name': uv,
                    'shells': shells
                })
        
        return conflicts
    
    def fix_shell_materials(self, output_dir: Optional[str] = None) -> Dict:
        """Fix shell material conflicts by adding prefixes"""
        
        if not self.shells:
            self.find_all_shells()
        
        conflicts = self.detect_conflicts()
        
        result = {
            'shells_processed': len(self.shells),
            'conflicts_found': len(conflicts['conflict_zones']),
            'files_modified': [],
            'errors': []
        }
        
        # Create output directory
        if output_dir:
            out_path = Path(output_dir)
        else:
            out_path = self.root_path / 'Fixed_Shells'
        
        out_path.mkdir(parents=True, exist_ok=True)
        
        # Backup original files
        if self.create_backup:
            backup_path = out_path / 'Original_Backup'
            backup_path.mkdir(parents=True, exist_ok=True)
            for shell in self.shells:
                dest = backup_path / shell.path.name
                shutil.copy2(shell.path, dest)
        
        # Process each shell
        for shell in self.shells:
            try:
                fixed_path = self._fix_shell_file(shell, out_path)
                result['files_modified'].append(str(fixed_path))
                self.fixes.append({
                    'shell': shell.name,
                    'type': shell.type,
                    'zones_fixed': len(shell.material_zones)
                })
            except Exception as e:
                result['errors'].append({
                    'shell': shell.name,
                    'error': str(e)
                })
        
        return result
    
    def _fix_shell_file(self, shell: ShellInfo, output_dir: Path) -> Path:
        """Fix a single shell file by renaming material zones"""
        
        # Read and decompress
        with gzip.open(shell.path, 'rt', encoding='utf-8', errors='ignore') as f:
            content = f.read()
        
        data = json.loads(content)
        
        # Get prefix for this shell type
        prefix = self.SHELL_MATERIAL_PREFIXES.get(shell.type, 'S_')
        
        # Rename material zones with prefix
        if 'material_library' in data:
            for mat in data['material_library']:
                original_id = mat.get('id', '')
                # Only rename if it's a conflict zone
                if any(zone in original_id for zone in self.CONFLICT_ZONES):
                    mat['id'] = prefix + original_id
        
        # Add shell type comment
        if 'asset_info' not in data:
            data['asset_info'] = {}
        data['asset_info']['shell_type'] = shell.type
        data['asset_info']['original_zones_renamed'] = True
        
        # Write fixed file
        output_path = output_dir / f"Fixed_{shell.path.name}"
        
        with gzip.open(output_path, 'wt', encoding='utf-8') as f:
            json.dump(data, f, indent=2)
        
        return output_path
    
    def create_stacking_script(self, output_path: str, shell_order: List[str]) -> Path:
        """Create a script for proper shell stacking order"""
        
        script = {
            'file_version': '0.6.1.0',
            'asset_info': {
                'id': '/Millsy/ShellStacking/MakeNudeShells.dsf',
                'type': 'script',
                'name': 'Millsy Shell Stacking Script'
            },
            'script': {
                'description': 'Proper shell stacking order for Millsy Make Nude',
                'shell_order': shell_order,
                'instructions': [
                    '1. Load base character first',
                    '2. Load shells in specified order',
                    '3. Apply materials after all shells loaded',
                    '4. Apply UV sets last'
                ]
            }
        }
        
        output_file = Path(output_path) / 'ShellStacking_Script.dsf'
        
        with gzip.open(output_file, 'wt', encoding='utf-8') as f:
            json.dump(script, f, indent=2)
        
        return output_file


def scan_shells(root_path: str, verbose: bool = True) -> Dict:
    """Main function to scan and analyze shells"""
    
    fixer = ShellMaterialFixer(root_path)
    shells = fixer.find_all_shells()
    
    if verbose:
        print(f"\n=== SHELL ANALYSIS ===")
        print(f"Total shells found: {len(shells)}")
        
        for shell in shells:
            print(f"\nShell: {shell.name}")
            print(f"  Type: {shell.type}")
            print(f"  Path: {shell.path}")
            print(f"  Material Zones: {len(shell.material_zones)}")
            print(f"  UV Sets: {len(shell.uv_sets)}")
    
    conflicts = fixer.detect_conflicts()
    
    if verbose:
        print(f"\n=== CONFLICTS ===")
        print(f"Material zone conflicts: {len(conflicts['conflict_zones'])}")
        for c in conflicts['conflict_zones']:
            print(f"  - {c['zone']}: {c['shells']}")
        
        print(f"UV conflicts: {len(conflicts['uv_conflicts'])}")
        for c in conflicts['uv_conflicts']:
            print(f"  - {c['uv_name']}: {c['shells']}")
    
    return {
        'shells': [s.name for s in shells],
        'conflicts': conflicts
    }


def fix_shells(root_path: str, output_dir: Optional[str] = None, verbose: bool = True) -> Dict:
    """Main function to fix shell conflicts"""
    
    fixer = ShellMaterialFixer(root_path)
    result = fixer.fix_shell_materials(output_dir)
    
    if verbose:
        print(f"\n=== SHELL FIX RESULTS ===")
        print(f"Shells processed: {result['shells_processed']}")
        print(f"Conflicts found: {result['conflicts_found']}")
        print(f"Files modified: {len(result['files_modified'])}")
        if result['errors']:
            print(f"Errors: {len(result['errors'])}")
    
    return result


if __name__ == '__main__':
    import argparse
    
    parser = argparse.ArgumentParser(description='DAZ Shell Material Fixer')
    parser.add_argument('path', help='Root path containing shell assets')
    parser.add_argument('--output', '-o', help='Output directory for fixed files')
    parser.add_argument('--fix', '-f', action='store_true', help='Apply fixes')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    if args.fix:
        result = fix_shells(args.path, args.output, args.verbose)
    else:
        result = scan_shells(args.path, args.verbose)
    
    print("\nDone!")