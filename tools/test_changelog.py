"""Tests for the release notes in four languages and their check tools/changelog.py (#188).

Run: python3 -m unittest discover -s tools -p 'test_changelog.py'
"""

import io
import pathlib
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout

TOOLS = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))
import changelog  # noqa: E402

ROOT = changelog.ROOT

GLOSSARY = """# Glossar

## Übersetzungen

### Kürzel

| DE | EN | FR | IT | Quelle | verbotene Varianten |
|---|---|---|---|---|---|
| AHV | AHV | AVS | AVS | AHVG Titel | fr: AHV; it: AHV |

### Fachbegriffe

| DE | EN | FR | IT | Quelle | verbotene Varianten |
|---|---|---|---|---|---|
| liquides Vermögen | liquid assets | liquidités | liquida | keine amtliche Quelle | it: Liquidità |
"""

HEAD = {"de": "Version", "en": "Version", "fr": "Version", "it": "Versione"}


def entry(lang, version, sections):
    """One version entry; sections is [(index into changelog.SECTIONS, text)]."""
    lines = [f"## {HEAD[lang]} {version}", "", "Intro.", ""]
    for index, text in sections:
        lines += [f"### {changelog.SECTIONS[lang][index]}", "", f"- {text}", ""]
    return "\n".join(lines)


def standard(lang):
    """Two translated versions plus, in German, one older untranslated version."""
    text = "# Changelog\n\n" + entry(lang, "1.0.0-beta.2", [(0, "Tax (#150)"), (1, "Icon (#185), help (#183)"),
                                                           (2, "Tests (#186, #187)")])
    text += "\n" + entry(lang, "1.0.0-beta.1", [(1, "Menu (#112)")])
    if lang == "de":
        text += "\n## Version 0.8.1\n\n- Alt (#93)\n"
    return text


class Tree:
    """A temporary project root with the four changelogs and a glossary."""

    def __init__(self, texts=None):
        self.dir = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.dir.name)
        (self.root / "docs").mkdir()
        (self.root / "docs" / "GLOSSAR.md").write_text(GLOSSARY, encoding="utf-8")
        texts = texts or {}
        for lang in changelog.LANGS:
            if texts.get(lang, "") is not None:
                (self.root / changelog.FILES[lang]).write_text(texts.get(lang) or standard(lang), encoding="utf-8")

    def findings(self):
        return [(f.file, f.where, f.check) for f in changelog.run_checks(self.root)]

    def close(self):
        self.dir.cleanup()


class Base(unittest.TestCase):
    def tree(self, **texts):
        tree = Tree(texts)
        self.addCleanup(tree.close)
        return tree


class ProjectChangelogs(unittest.TestCase):
    def test_issue_188_ak1_each_translation_has_every_version_from_beta1(self):
        # #188/AK1: the three translations exist and carry every German version from 1.0.0-beta.1 on.
        de = [v for v, _, _ in changelog.translated(changelog.parse((ROOT / "CHANGELOG.md").read_text("utf-8")))]
        self.assertEqual(de[-1], "1.0.0-beta.1")
        for lang in changelog.TARGETS:
            text = (ROOT / changelog.FILES[lang]).read_text(encoding="utf-8")
            self.assertEqual([v for v, _, _ in changelog.parse(text)], de, lang)

    def test_issue_188_ak1_older_versions_stay_german_only(self):
        # #188/AK1: 0.8.1 stays in the German file only.
        self.assertIn("## Version 0.8.1", (ROOT / "CHANGELOG.md").read_text(encoding="utf-8"))
        for lang in changelog.TARGETS:
            self.assertNotIn("0.8.1", (ROOT / changelog.FILES[lang]).read_text(encoding="utf-8"))

    def test_issue_188_ak3_ak4_project_changelogs_have_no_finding(self):
        # #188/AK2–AK4: the real files pass the check.
        self.assertEqual([str(f) for f in changelog.run_checks(ROOT)], [])


class Structure(Base):
    def test_issue_188_ak2_valid_tree_has_no_finding(self):
        self.assertEqual(self.tree().findings(), [])

    def test_issue_188_ak2_unknown_section_is_a_finding(self):
        # #188/AK2: a heading outside the fixed list of the language.
        text = standard("en").replace("### Tests and development", "### Internals")
        self.assertIn(("CHANGELOG.en.md", "1.0.0-beta.2", "sections"), self.tree(en=text).findings())

    def test_issue_188_ak2_wrong_order_is_a_finding(self):
        # #188/AK2: «May change results» after «New, changed, fixed».
        text = "# Changelog\n\n" + entry("en", "1.0.0-beta.2", [(1, "Icon (#185), help (#183)"), (0, "Tax (#150)"),
                                                               (2, "Tests (#186, #187)")])
        text += "\n" + entry("en", "1.0.0-beta.1", [(1, "Menu (#112)")])
        self.assertIn(("CHANGELOG.en.md", "1.0.0-beta.2", "sections"), self.tree(en=text).findings())

    def test_issue_188_ak2_entry_without_section_is_a_finding(self):
        # #188/AK2: also in German.
        text = standard("de").replace("### Neu, geändert, behoben\n", "")
        self.assertIn(("CHANGELOG.md", "1.0.0-beta.1", "sections"), self.tree(de=text).findings())

    def test_issue_188_ak2_old_german_versions_are_not_checked(self):
        # #188/AK1/AK2: 0.8.1 has no sections and no translation, without a finding.
        self.assertEqual(self.tree().findings(), [])


class Comparison(Base):
    def test_issue_188_ak3_missing_issue_in_french_is_a_finding(self):
        # #188/AK3: the example of the issue: #185 missing in CHANGELOG.fr.md for 1.0.0-beta.2.
        text = standard("fr").replace("Icon (#185), help (#183)", "Icon, help (#183)")
        self.assertEqual(self.tree(fr=text).findings(), [("CHANGELOG.fr.md", "1.0.0-beta.2", "issues")])

    def test_issue_188_ak3_extra_issue_is_a_finding(self):
        text = standard("it").replace("Menu (#112)", "Menu (#112, #999)")
        self.assertEqual(self.tree(it=text).findings(), [("CHANGELOG.it.md", "1.0.0-beta.1", "issues")])

    def test_issue_188_ak3_issue_in_other_section_is_a_finding(self):
        # #188/AK3: issue numbers are compared per section, not per version.
        text = standard("en").replace("Tax (#150)", "Tax").replace("Tests (#186, #187)", "Tests (#150, #186, #187)")
        self.assertEqual(len(self.tree(en=text).findings()), 2)

    def test_issue_188_ak3_missing_version_is_a_finding(self):
        text = standard("en").split("## Version 1.0.0-beta.1")[0]
        self.assertEqual(self.tree(en=text).findings(), [("CHANGELOG.en.md", "file", "versions")])

    def test_issue_188_ak3_swapped_versions_are_a_finding(self):
        b2, b1 = standard("fr").split("\n## Version 1.0.0-beta.1")
        text = "# Changelog\n\n## Version 1.0.0-beta.1" + b1 + "\n" + b2.replace("# Changelog\n\n", "")
        self.assertIn(("CHANGELOG.fr.md", "file", "versions"), self.tree(fr=text).findings())

    def test_issue_188_ak3_missing_section_is_a_finding(self):
        text = standard("it").replace("### Può modificare i risultati\n\n- Tax (#150)\n", "")
        self.assertIn(("CHANGELOG.it.md", "1.0.0-beta.2", "sections"), self.tree(it=text).findings())

    def test_issue_188_ak3_missing_file_is_an_input_error(self):
        tree = self.tree(it=None)
        with redirect_stdout(io.StringIO()) as out:
            self.assertEqual(changelog.main(["--root", str(tree.root), "check"]), 2)
        self.assertIn("CHANGELOG.it.md missing", out.getvalue())


class Terms(Base):
    def test_issue_188_ak4_forbidden_variant_is_a_finding(self):
        # #188/AK4: the example of the issue: «Liquidità» in CHANGELOG.it.md.
        text = standard("it").replace("Menu (#112)", "Menu e Liquidità (#112)")
        self.assertEqual(self.tree(it=text).findings(), [("CHANGELOG.it.md", "1.0.0-beta.1", "terms")])

    def test_issue_188_ak4_forbidden_abbreviation_only_as_whole_word(self):
        text = standard("fr").replace("Menu (#112)", "Menu AHV (#112)")
        self.assertEqual(self.tree(fr=text).findings(), [("CHANGELOG.fr.md", "1.0.0-beta.1", "terms")])
        text = standard("fr").replace("Menu (#112)", "Menu AHVG (#112)")
        self.assertEqual(self.tree(fr=text).findings(), [])

    def test_issue_188_ak4_variant_of_other_language_is_allowed(self):
        text = standard("en").replace("Menu (#112)", "Menu Liquidità (#112)")
        self.assertEqual(self.tree(en=text).findings(), [])

    def test_issue_188_ak3_cli_exit_code_with_findings(self):
        text = standard("it").replace("Menu (#112)", "Menu Liquidità (#112)")
        tree = self.tree(it=text)
        with redirect_stdout(io.StringIO()) as out:
            self.assertEqual(changelog.main(["--root", str(tree.root), "check"]), 1)
        self.assertIn("CHANGELOG.it.md 1.0.0-beta.1 [terms]", out.getvalue())


class Notes(Base):
    def test_issue_188_ak6_english_open_first_others_folded(self):
        # #188/AK6: English first and open, then German, French and Italian in <details> blocks.
        text = changelog.notes(self.tree().root, "v1.0.0-beta.2")
        self.assertTrue(text.startswith("Intro."))
        self.assertIn("### May change results", text.split("<details>")[0])
        summaries = [part.split("</summary>")[0] for part in text.split("<summary>")[1:]]
        self.assertEqual(summaries, ["Deutsch", "Français", "Italiano"])
        self.assertEqual(text.count("<details>"), 3)
        self.assertEqual(text.count("</details>"), 3)
        self.assertIn("### Kann Ergebnisse ändern", text.split("<summary>Deutsch</summary>")[1])
        self.assertNotIn("## Version", text)
        self.assertNotIn("Menu (#112)", text)

    def test_issue_188_ak6_version_without_v_prefix(self):
        root = self.tree().root
        self.assertEqual(changelog.notes(root, "1.0.0-beta.1"), changelog.notes(root, "v1.0.0-beta.1"))

    def test_issue_188_ak6_missing_version_is_an_error(self):
        # #188/AK6/AK7: a version missing in one language fails, with exit code 1 on the command line.
        text = standard("fr").split("## Version 1.0.0-beta.1")[0]
        tree = self.tree(fr=text)
        with self.assertRaisesRegex(ValueError, "CHANGELOG.fr.md: version 1.0.0-beta.1 missing"):
            changelog.notes(tree.root, "v1.0.0-beta.1")
        with redirect_stderr(io.StringIO()) as err:
            self.assertEqual(changelog.main(["--root", str(tree.root), "notes", "v1.0.0-beta.1"]), 1)
        self.assertIn("missing", err.getvalue())

    def test_issue_188_ak6_old_german_only_version_is_an_error(self):
        with self.assertRaisesRegex(ValueError, "CHANGELOG.en.md: version 0.8.1 missing"):
            changelog.notes(self.tree().root, "v0.8.1")

    def test_issue_188_ak6_output_file(self):
        tree = self.tree()
        out = tree.root / "notes.md"
        self.assertEqual(changelog.main(["--root", str(tree.root), "notes", "v1.0.0-beta.2", "-o", str(out)]), 0)
        self.assertEqual(out.read_text(encoding="utf-8"), changelog.notes(tree.root, "v1.0.0-beta.2"))

    def test_issue_188_ak6_project_notes_of_current_version(self):
        text = changelog.notes(ROOT, "v1.0.0-beta.2")
        self.assertIn("(#188)", text.split("<details>")[0])
        self.assertEqual(text.count("<details>"), 3)


class Wiring(unittest.TestCase):
    def test_issue_188_ak5_check_runs_in_test_quiet_and_on_github(self):
        # #188/AK5: make test-quiet and the GitHub run (job english) run the check and its tests.
        makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
        line = next(l for l in makefile.splitlines() if l.startswith("test-quiet:"))
        self.assertIn("tools/changelog.py check", line)
        workflow = (ROOT / ".github" / "workflows" / "test-quality.yml").read_text(encoding="utf-8")
        english = workflow.split("\n  english:")[1].split("\n  # ")[0]
        self.assertIn("test_changelog.py", english)
        self.assertIn("tools/changelog.py check", english)

    def test_issue_188_ak7_package_build_uses_and_requires_the_notes(self):
        # #188/AK7: the test job (before the packages) fails without notes for the tag; the release uses them.
        workflow = (ROOT / ".github" / "workflows" / "pakete.yml").read_text(encoding="utf-8")
        test_job = workflow.split("\n  test:")[1].split("\n  pakete:")[0]
        self.assertIn("tools/changelog.py notes", test_job)
        release_job = workflow.split("\n  release:")[1]
        self.assertIn("tools/changelog.py notes", release_job)
        self.assertIn("body_path:", release_job)

    def test_issue_188_ak8_release_procedure_documented(self):
        # #188/AK8: docs/ENTWICKLUNG.md describes the steps per release.
        text = (ROOT / "docs" / "ENTWICKLUNG.md").read_text(encoding="utf-8")
        section = text.split("### Release Notes")[1].split("\n## ")[0]
        for word in ("CHANGELOG.md", "CHANGELOG.en.md", "tools/changelog.py check", "Tag"):
            self.assertIn(word, section)


if __name__ == "__main__":
    unittest.main()
