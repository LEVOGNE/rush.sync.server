#!/usr/bin/env python3
"""
Rush Sync Server JSON Key Analyzer
Speziell angepasst für dein Projekt

Usage:
    python rush_analyzer.py
    python rush_analyzer.py --cleanup  # Erstelle bereinigte JSON-Dateien
"""

import re
import json
import os
from pathlib import Path
from collections import defaultdict


class RushSyncAnalyzer:
    def __init__(self):
        self.project_path = Path(".")
        self.used_keys = set()

        # ✅ RUSH SYNC SPEZIFISCHE PATTERNS
        self.code_patterns = [
            r'get_translation\("([^"]+)"',
            r'get_command_translation\("([^"]+)"',
            r'i18n::get_translation\("([^"]+)"',
            r'crate::i18n::get_translation\("([^"]+)"',
            r'TranslationError::LoadError\("([^"]+)"',
        ]

        # ✅ BEKANNTE RUSH SYNC ÜBERSETZUNGSKEYS (aus dem Code)
        self.known_patterns = [
            "system.startup.*",
            "system.config.*",
            "system.commands.*",
            "system.error.*",
            "system.input.*",
            "system.logging.*",
            "system.translation.*",
            "system.log.*",
        ]

    def scan_project(self):
        """Scannt komplettes Rush Sync Projekt"""
        print("🔍 Scanning Rush Sync Server project...")

        # Relevante Dateien finden
        rust_files = [
            "src/main.rs",
            "src/lib.rs",
            "src/core/error.rs",
            "src/core/config.rs",
            "src/commands/**/*.rs",
            "src/i18n/mod.rs",
            "src/input/input.rs",
            "src/output/logging.rs",
        ]

        all_files = []
        for pattern in rust_files:
            if "**" in pattern:
                all_files.extend(self.project_path.glob(pattern))
            else:
                file_path = self.project_path / pattern
                if file_path.exists():
                    all_files.append(file_path)

        print(f"Found {len(all_files)} relevant Rust files")

        # Scanne alle Dateien
        for file_path in all_files:
            self._scan_file(file_path)

        # ✅ ZUSÄTZLICH: Scan ganzes src/ für missed patterns
        additional_files = list(self.project_path.glob("src/**/*.rs"))
        for file_path in additional_files:
            if file_path not in all_files:
                self._scan_file(file_path)

        print(f"✅ Found {len(self.used_keys)} unique translation keys")
        return self.used_keys

    def _scan_file(self, file_path):
        """Scannt einzelne Datei"""
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                content = f.read()

            for pattern in self.code_patterns:
                matches = re.findall(pattern, content)
                for match in matches:
                    self.used_keys.add(match)
                    print(f"  📝 {file_path.name}: {match}")

        except Exception as e:
            print(f"❌ Error reading {file_path}: {e}")

    def analyze_json_files(self):
        """Analysiert beide JSON-Dateien (de.json + en.json)"""
        json_files = {"de": "src/i18n/langs/de.json", "en": "src/i18n/langs/en.json"}

        results = {}

        for lang, path in json_files.items():
            if not os.path.exists(path):
                print(f"⚠️ {path} not found, skipping")
                continue

            print(f"\n📊 Analyzing {lang.upper()}: {path}")
            results[lang] = self._analyze_single_json(path)

        return results

    def _analyze_single_json(self, json_path):
        """Analysiert einzelne JSON-Datei"""
        try:
            with open(json_path, "r", encoding="utf-8") as f:
                json_data = json.load(f)
        except Exception as e:
            print(f"❌ Error reading JSON: {e}")
            return None

        # Extrahiere Keys
        text_keys = set()
        category_keys = set()
        display_keys = set()
        all_keys = set(json_data.keys())

        for key in json_data.keys():
            if key.endswith(".text"):
                text_keys.add(key[:-5])  # Remove .text
            elif key.endswith(".category"):
                category_keys.add(key[:-9])  # Remove .category
            elif key.endswith(".display_category"):
                display_keys.add(key[:-17])  # Remove .display_category

        return {
            "json_data": json_data,
            "text_keys": text_keys,
            "category_keys": category_keys,
            "display_keys": display_keys,
            "all_keys": all_keys,
            "path": json_path,
        }

    def generate_detailed_report(self, json_results):
        """Generiert detaillierten Rush Sync Report"""
        print("\n" + "=" * 70)
        print("📋 RUSH SYNC SERVER - JSON ANALYSIS REPORT")
        print("=" * 70)

        for lang, data in json_results.items():
            if not data:
                continue

            print(f"\n🌍 {lang.upper()} LANGUAGE FILE:")
            print("-" * 40)

            text_keys = data["text_keys"]
            unused_keys = text_keys - self.used_keys
            missing_keys = self.used_keys - text_keys

            print(f"✅ Used keys in code: {len(self.used_keys)}")
            print(f"📚 Defined in JSON: {len(text_keys)}")
            print(f"❌ Unused keys: {len(unused_keys)}")
            print(f"⚠️ Missing keys: {len(missing_keys)}")

            # Ungenutzte Keys
            if unused_keys:
                print(f"\n🗑️ UNUSED KEYS (safe to remove):")
                for key in sorted(unused_keys):
                    print(f"   - {key}")

            # Fehlende Keys
            if missing_keys:
                print(f"\n⚠️ MISSING KEYS (add to JSON):")
                for key in sorted(missing_keys):
                    print(f"   - {key}")

            # Redundanz-Analyse
            self._analyze_redundancy(data, lang)

    def _analyze_redundancy(self, data, lang):
        """Rush Sync spezifische Redundanz-Analyse"""
        json_data = data["json_data"]
        redundant_count = 0
        redundant_keys = []

        for base_key in data["text_keys"]:
            category_key = f"{base_key}.category"
            display_key = f"{base_key}.display_category"

            if category_key in json_data and display_key in json_data:
                if json_data[category_key] == json_data[display_key]:
                    redundant_count += 1
                    redundant_keys.append(base_key)

        print(f"\n🔍 REDUNDANCY ANALYSIS ({lang.upper()}):")
        print(f"📊 Redundant display_category: {redundant_count}")

        if redundant_keys:
            print("   (identical to category - can be auto-generated)")
            # Zeige Beispiele
            for key in redundant_keys[:5]:
                category_val = json_data[f"{key}.category"]
                print(f"   - {key}: '{category_val}' = '{category_val}'")
            if len(redundant_keys) > 5:
                print(f"   ... and {len(redundant_keys) - 5} more")

        # ✅ RUSH SYNC SPEZIFISCHE MAPPINGS PRÜFEN
        self._check_rush_sync_mappings(data, lang)

    def _check_rush_sync_mappings(self, data, lang):
        """Prüft Rush Sync spezifische Farb-Mappings"""
        json_data = data["json_data"]

        expected_mappings = {
            "error": "fehler" if lang == "de" else "error",
            "warning": "warnung" if lang == "de" else "warning",
            "info": "info",
            "lang": "sprache" if lang == "de" else "language",
            "debug": "debug",
        }

        print(f"\n🎨 COLOR MAPPING CHECK ({lang.upper()}):")
        inconsistent = []

        for base_key in data["text_keys"]:
            category_key = f"{base_key}.category"
            display_key = f"{base_key}.display_category"

            if category_key in json_data and display_key in json_data:
                category = json_data[category_key]
                display = json_data[display_key]

                if category in expected_mappings:
                    expected = expected_mappings[category]
                    if display != expected and display != category:
                        inconsistent.append((base_key, category, display, expected))

        if inconsistent:
            print("⚠️ Inconsistent mappings found:")
            for key, cat, disp, exp in inconsistent[:3]:
                print(f"   - {key}: {cat} → '{disp}' (expected: '{exp}')")
        else:
            print("✅ All color mappings are consistent")

    def create_optimized_files(self, json_results):
        """Erstellt optimierte JSON-Dateien"""
        print(f"\n🛠️ CREATING OPTIMIZED FILES:")
        print("-" * 40)

        for lang, data in json_results.items():
            if not data:
                continue

            original_path = data["path"]
            optimized_path = f"optimized_{lang}.json"

            optimized = self._create_optimized_json(data, lang)

            # Speichern
            with open(optimized_path, "w", encoding="utf-8") as f:
                json.dump(optimized, f, indent=2, ensure_ascii=False)

            original_size = len(data["json_data"])
            optimized_size = len(optimized)
            reduction = ((original_size - optimized_size) / original_size) * 100

            print(
                f"✅ {lang.upper()}: {original_size} → {optimized_size} keys ({reduction:.1f}% reduction)"
            )
            print(f"   Saved to: {optimized_path}")

    def _create_optimized_json(self, data, lang):
        """Erstellt optimierte JSON für eine Sprache"""
        json_data = data["json_data"]
        optimized = {}

        # ✅ NUR VERWENDETE KEYS ÜBERNEHMEN
        for base_key in self.used_keys:
            text_key = f"{base_key}.text"
            category_key = f"{base_key}.category"
            display_key = f"{base_key}.display_category"

            # Text (immer)
            if text_key in json_data:
                optimized[text_key] = json_data[text_key]

            # Category (immer)
            if category_key in json_data:
                optimized[category_key] = json_data[category_key]

            # Display nur wenn unterschiedlich (Rush Sync Optimierung)
            if display_key in json_data and category_key in json_data:
                category_val = json_data[category_key]
                display_val = json_data[display_key]

                # ✅ NUR ÜBERNEHMEN WENN UNTERSCHIEDLICH
                if display_val != category_val:
                    # Verkürze: display_category → display
                    optimized[f"{base_key}.display"] = display_val

        return optimized

    def generate_migration_code(self):
        """Generiert Rust-Code für Migration"""
        print(f"\n🦀 GENERATING RUST MIGRATION CODE:")
        print("-" * 50)

        migration_code = """
// =====================================================
// AUTO-GENERATED: Rush Sync JSON Migration
// =====================================================

impl TranslationConfig {
    fn auto_generate_display(category: &str, lang: &str) -> String {
        match (category.to_lowercase().as_str(), lang) {"""

        # Generiere Mappings
        mappings = {
            ("error", "de"): "fehler",
            ("warning", "de"): "warnung",
            ("warn", "de"): "warnung",
            ("info", "de"): "info",
            ("debug", "de"): "debug",
            ("lang", "de"): "sprache",
            ("version", "de"): "version",
            ("startup", "de"): "bereit",
        }

        for (cat, lang), display in mappings.items():
            migration_code += (
                f'\n            ("{cat}", "{lang}") => "{display}".to_string(),'
            )

        migration_code += """
            // Fallback für andere Sprachen/Kategorien
            _ => category.to_uppercase(),
        }
    }
}"""

        print(migration_code)

        # Speichere als Datei
        with open("migration_code.rs", "w") as f:
            f.write(migration_code)
        print(f"\n💾 Migration code saved to: migration_code.rs")


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Rush Sync Server JSON Analyzer")
    parser.add_argument(
        "--cleanup", action="store_true", help="Create optimized JSON files"
    )
    parser.add_argument(
        "--migration", action="store_true", help="Generate Rust migration code"
    )

    args = parser.parse_args()

    analyzer = RushSyncAnalyzer()

    print("🚀 RUSH SYNC SERVER - JSON KEY ANALYZER")
    print("=" * 50)

    # 1. Scanne Projekt
    analyzer.scan_project()

    # 2. Analysiere JSON-Dateien
    json_results = analyzer.analyze_json_files()

    # 3. Generiere Report
    analyzer.generate_detailed_report(json_results)

    # 4. Optional: Erstelle bereinigte Dateien
    if args.cleanup:
        analyzer.create_optimized_files(json_results)

    # 5. Optional: Generiere Migration-Code
    if args.migration:
        analyzer.generate_migration_code()

    print(f"\n🎯 SUMMARY:")
    print(f"   - Found {len(analyzer.used_keys)} used translation keys")
    print(f"   - Analysis complete for {len(json_results)} language files")

    if not args.cleanup:
        print(f"\n💡 Run with --cleanup to create optimized JSON files")
        print(f"💡 Run with --migration to generate Rust migration code")


if __name__ == "__main__":
    main()
