"""#285 (way b3): Tests of reading the tax multipliers of the cantons (invented municipalities and values).

Run: `python3 -I -m unittest discover -s tools/steuerdaten`
"""

import io
import json
import os
import sys
import tempfile
import unittest
import zlib
from contextlib import redirect_stdout

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import steuerfuesse  # noqa: E402

ZH_HEAD = ("LAST_REFRESH,YEAR,COMMUNITY,BFSNR,FLAG,STF_O_KIRCHE1,STF_O_KIRCHE2,STF_O_KIRCHE3,DIFF_O_KIRCHE,"
           "STF_REF1,STF_REF2,STF_REF3,DIFF_REF,STF_KATH1,STF_KATH2,STF_KATH3,DIFF_KATH,"
           "STF_CKRKATH1,STF_CKRKATH2,STF_CKRKATH3,DIFF_CHRKATH,JUR_PERS\n")
ZH_ROWS = ('"01.03.2026",2026,"Seedorf",9001,"definitiv",110,,,0,121,,,0,122,,,0,124,,,0,121.5\n'
           '"01.03.2026",2026,"Bachtal",9002,"provisorisch",100,98,,0,110,108,,0,112,,,0,115,,,0,111\n')

REFERENCE = [
    [9001, "Seedorf", "ZH", [0.95, 1.1, 0.11, 0.12, 0.14], [0.95, 1.1, 0.11, 0.12, 0.14]],
    [9002, "Bachtal", "ZH", [0.95, 1.0, 0.1, 0.13, 0.15], [0.95, 1.0, 0.1, 0.13, 0.15]],
    [9003, "Hügelwil", "ZH", [0.95, 1.2, 0.1, 0.1, 0.1], [0.95, 1.2, 0.1, 0.1, 0.1]],
    [9101, "Muri bei Langsee", "BE", [2.5, 1.5, 0.2, 0.19, 0.28], [2.5, 1.5, 0.2, 0.19, 0.28]],
    [9102, "Rüti bei Waldhof", "BE", [2.5, 1.7, 0.18, 0.19, 0.28], [2.5, 1.7, 0.18, 0.19, 0.28]],
    [9103, "Rüti bei Langsee", "BE", [2.5, 1.6, 0.18, 0.19, 0.28], [2.5, 1.6, 0.18, 0.19, 0.28]],
    [9104, "Forst-Ebenau", "BE", [2.5, 1.8, 0.2, 0.2, 0.2], [2.5, 1.8, 0.2, 0.2, 0.2]],
    [9105, "Bergen (BE)", "BE", [2.5, 1.9, 0.2, 0.2, 0.2], [2.5, 1.9, 0.2, 0.2, 0.2]],
]

BE_LINES = [
    "Kantonssteueranlagen",
    "2025         2.400 / 2.200",
    "2026    2.500 / 2.300",
    "Tabelle 2.1  Alphabetische Reihenfolge",
    "Bergen  | 1.90 | 1.90 | 1.00 | 1\x1e204 | 80 | 0.2000 | 0.2000 | 0.2000 | 0.2000",
    "Forst | - | Ebenau | 1.80 | 1.80 | 1.20 | 209 | 17 | 0.2000 | 0.2000 | 0.2000 | 0.2000",
    "Ebenau-Dorf | 246 | 13 | 0.2460 | 0.1950 | 0.2300 | 0.2428",
    "Muri b. L.**  1.50  1.50  1.50  9\x1e056  622  0.2000  0.1900  0.2800  0.1871 ",
    "Rüti b. W.  | 1.70 | 1.70 | 1.00 | 300 | 20 | 0.1800 | 0.1900 | 0.2800 | 0.1800",
    "Rüti b. L.  | 1.60 | 1.60 | 1.00 | 300 | 20 | 0.1800 | 0.1900 | 0.2800 | 0.1800",
    "Rüttiwil  | 1.60 | 1.60 | 1.00 | 300 | 20 | 0.1800 | 0.1900 | 0.2800 | 0.1800",
    "Tabelle 2.2  Reihenfolge nach Verwaltungskreisen",
    "Muri b. L.  | 9.99 | 9.99 | 1.50 | 9056 | 622 | 0.9000 | 0.9000 | 0.9000 | 0.9000",
]


def write(text, suffix):
    f = tempfile.NamedTemporaryFile("w", suffix=suffix, delete=False, encoding="utf-8")
    f.write(text)
    f.close()
    return f.name


class ReadZurich(unittest.TestCase):
    # #285/AK1 (b3): the open data of ZH give municipal and church multipliers; the cantonal one comes as an option.
    def test_issue_285_ak1_zh_open_data(self):
        path = write(ZH_HEAD + ZH_ROWS, ".csv")
        rows = steuerfuesse.read_zh(path, 0.95)
        os.unlink(path)
        name, values, notes = rows[9001]
        self.assertEqual(name, "Seedorf")
        self.assertEqual(values, [0.95, 1.1, 0.11, 0.12, 0.14])
        self.assertEqual(notes, [])
        self.assertEqual(rows[9002][1], [0.95, 1.0, 0.1, 0.12, 0.15])
        self.assertEqual(rows[9002][2], ["several parts with different multipliers, part 1 taken",
                                         "status provisorisch"])


class ReadPdf(unittest.TestCase):
    # #285/AK1 (b3): text of a PDF page without libraries: pieces on one baseline form one line, left to right.
    def test_issue_285_ak1_pdf_text(self):
        content = (b"BT /F0 1 Tf 8.5 0 0 8.5 40 700 Tm [(Kon)-5(ol)]TJ 10 0 Td [(\\037ngen)]TJ ET\n"
                   b"BT 8.5 0 0 8.5 200 700 Tm (1.50)Tj ET\n"
                   b"BT 8.5 0 0 8.5 40 690 Tm [(Muri)-300(b.)]TJ ET\n")
        self.assertEqual(steuerfuesse.text_lines(content), ["Konol | flngen | 1.50", "Muri b."])
        pdf = tempfile.NamedTemporaryFile(suffix=".pdf", delete=False)
        pdf.write(b"%PDF-1.4\n1 0 obj<<>>stream\n" + zlib.compress(content) + b"\nendstream\nendobj\n"
                  b"2 0 obj<<>>stream\n" + zlib.compress(b"0 0 m 1 1 l S") + b"\nendstream\nendobj\n"
                  b"3 0 obj<<>>stream\nnot compressed\nendstream\nendobj\n")
        pdf.close()
        self.assertEqual(steuerfuesse.pdf_lines(pdf.name), ["Konol | flngen | 1.50", "Muri b."])
        os.unlink(pdf.name)
        self.assertEqual(steuerfuesse.unescape(b"\\(a\\)\\n\\374\\035x"), "(a)\nüfix")


class ReadBern(unittest.TestCase):
    # #285/AK1 (b3): table 2.1 of the Bern PDF: quota and church rates, cantonal quota of the year, sections.
    def test_issue_285_ak1_be_table(self):
        rows, quota, sections = steuerfuesse.read_be(BE_LINES, 2026)
        self.assertEqual(quota, 2.5)
        self.assertEqual(rows["Muri b. L."], [2.5, 1.5, 0.2, 0.19, 0.28])  # table 2.2 is not read
        self.assertEqual(rows["Forst-Ebenau"], [2.5, 1.8, 0.2, 0.2, 0.2])
        self.assertEqual(sections, {"Forst-Ebenau": ["Ebenau-Dorf"]})
        with self.assertRaises(ValueError):
            steuerfuesse.read_be(BE_LINES[3:], 2026)

    # #285/AK5: names without BFS numbers are matched with the reference, also abbreviated; ambiguous or unknown
    # names stay unmatched.
    def test_issue_285_ak5_name_matching(self):
        rows, _, _ = steuerfuesse.read_be(BE_LINES, 2026)
        matched, unmatched = steuerfuesse.match_names(rows, [m for m in REFERENCE if m[2] == "BE"])
        self.assertEqual({k: v[0] for k, v in matched.items()},
                         {9101: "Muri b. L.", 9102: "Rüti b. W.", 9103: "Rüti b. L.", 9104: "Forst-Ebenau",
                          9105: "Bergen"})
        self.assertEqual(unmatched, ["Rüttiwil"])
        self.assertEqual(steuerfuesse.keys("Rüti bei Langsee (BE)"), ["ruti bei langsee", "ruti bl", "ruti"])
        self.assertEqual(steuerfuesse.keys("--"), [])


GE_HTML = ("<html><p>arrête :</p><p>Communes Taux</p><table><tr><td>Ville de Seeville</td><td>45,49</td></tr>"
           "<tr><td>Le Grand-Pont</td><td>50</td></tr><tr><td>Bergd&ocirc;t</td><td>31 (1)</td></tr></table>"
           "<p>RSG Intitulé Date</p><p>Lacville 99</p></html>")
VD_CSV = ('2026,,,"En % imp. cant. base",,,\n'
          '"DISTRICT DU LAC",,,,,,\n'
          '"Seeville",2021,2026," 66.0 "," - "," 66.0 "\n'
          '"St-Martin (Lac)","2024","2029"," 70.0 "," 2.5 "," 72.5 "\n'
          '"Le Hameau (fraction)",2024,2026," 60.0 "," - "," 60.0 "\n'
          '"Bergdorf",2025,2026," - "," - "," - "\n')
FR_CSV = ("annee;commune;commune_id;coefficients_de;valeur\n"
          "2024;Seeville;9201;Einkommens- und Vermoegenssteuer, in % der einfachen Kantonssteuer;70.0\n"
          "2025;Seeville;9201;Einkommens- und Vermoegenssteuer, in % der einfachen Kantonssteuer;75.0\n"
          "2025;Seeville;9201;Gewinn- und Kapitalsteuer, in % der einfachen Kantonssteuer;80.0\n"
          "2027;Seeville;9201;Einkommens- und Vermoegenssteuer, in % der einfachen Kantonssteuer;90.0\n")


class ReadFrenchCantons(unittest.TestCase):
    # #285/AK1 (b3): GE decree: «centimes additionnels» in hundredths, «Ville de» dropped, footnotes ignored.
    def test_issue_285_ak1_ge_decree(self):
        path = write(GE_HTML, ".htm")
        rows = steuerfuesse.read_ge(path, 1.5)
        os.unlink(path)
        self.assertEqual(rows, {"Seeville": [1.5, 0.4549, None, None, None],
                                "Le Grand-Pont": [1.5, 0.5, None, None, None],
                                "Bergdôt": [1.5, 0.31, None, None, None]})
        path = write("<p>nothing</p>", ".htm")
        with self.assertRaises(ValueError):
            steuerfuesse.read_ge(path, 1.5)
        os.unlink(path)

    # #285/AK1 (b3): VD table: «Pour-cent total» of the municipalities; districts, fractions and empty rates skipped.
    def test_issue_285_ak1_vd_table(self):
        path = write(VD_CSV, ".csv")
        rows = steuerfuesse.read_vd(path, 1.55)
        os.unlink(path)
        self.assertEqual(rows, {"Seeville": [1.55, 0.66, None, None, None],
                                "St-Martin (Lac)": [1.55, 0.725, None, None, None]})

    # #285/AK1 (b3): FR open data: latest year up to the reference year, with a note if it is older.
    def test_issue_285_ak1_fr_open_data(self):
        path = write(FR_CSV, ".csv")
        rows, year = steuerfuesse.read_fr(path, 0.96, 2026)
        self.assertEqual(year, 2025)
        self.assertEqual(rows, {9201: ("Seeville", [0.96, 0.75, None, None, None], ["source year 2025, not 2026"])})
        self.assertEqual(steuerfuesse.read_fr(path, 0.96, 2024)[0][9201][2], [])
        with self.assertRaises(ValueError):
            steuerfuesse.read_fr(path, 0.96, 2020)
        os.unlink(path)

    # #285/AK5: articles, «St-» and accents do not prevent the match.
    def test_issue_285_ak5_french_names(self):
        self.assertEqual(steuerfuesse.words("Le Grand-Pont"), ["grand", "pont"])
        self.assertEqual(steuerfuesse.words("St-Martin (Lac)"), ["saint", "martin"])
        self.assertEqual(steuerfuesse.words("Vandœuvres"), ["vandoeuvres"])
        self.assertEqual(steuerfuesse.words("Les"), ["les"])


class Compare(unittest.TestCase):
    # #285/AK2: the comparison names every differing value, missing municipalities and notes.
    def test_issue_285_ak2_compare(self):
        source = {9001: ("Seedorf", [0.95, 1.1, 0.11, 0.12, 0.14], []),
                  9002: ("Bachtal", [0.95, 1.0, 0.1, 0.12, 0.15], ["note A"]),
                  9009: ("Neudorf", [0.95, 1.0, 0.1, 0.1, 0.1], [])}
        lines, n = steuerfuesse.compare("ZH", source, REFERENCE)
        self.assertEqual(lines, ["ZH 9002 Bachtal: catholic 0.12 (reference 0.13)",
                                 "ZH 9002 Bachtal: note: note A",
                                 "ZH 9003 Hügelwil: missing in the source",
                                 "ZH 9009 Neudorf: not in the reference"])
        self.assertEqual(n, {"reference": 3, "source": 3, "equal": 1})
        # values the source does not have are not compared
        lines, n = steuerfuesse.compare("ZH", {9002: ("Bachtal", [0.95, 1.0, None, None, None], [])}, REFERENCE[:2])
        self.assertEqual(lines, ["ZH 9001 Seedorf: missing in the source"])
        self.assertEqual(n["equal"], 1)

    def test_issue_285_ak2_main(self):
        ref = write(json.dumps({"year": 2026, "municipalities": REFERENCE}), ".json")
        zh = write(ZH_HEAD + ZH_ROWS, ".csv")
        out = io.StringIO()
        with redirect_stdout(out):
            code = steuerfuesse.main([ref, "--zh", zh])
        self.assertEqual(code, 1)
        self.assertIn("ZH: 1 of 3 municipalities equal, 2 differences or gaps", out.getvalue())
        with redirect_stdout(io.StringIO()):
            self.assertEqual(steuerfuesse.main([ref]), 0)
        ge, vd, fr = write(GE_HTML, ".htm"), write(VD_CSV, ".csv"), write(FR_CSV, ".csv")
        out = io.StringIO()
        with redirect_stdout(out):
            steuerfuesse.main([ref, "--ge", ge, "--vd", vd, "--fr", fr])
        self.assertIn("GE: 0 of 0 municipalities equal", out.getvalue())
        self.assertIn("FR 9201 Seeville: not in the reference", out.getvalue())
        for path in (ge, vd, fr):
            os.unlink(path)
        os.unlink(ref)
        os.unlink(zh)


if __name__ == "__main__":
    unittest.main()
