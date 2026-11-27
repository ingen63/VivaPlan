// #285: text of a PDF with PDFKit (part of macOS, no further dependency), for sources whose tables the reader of
// `steuerfuesse.py` cannot take apart. Usage: swift tools/steuerdaten/pdf2txt.swift [--rows] <file.pdf> > <file.txt>
//
// Without --rows: the text as PDFKit orders it (good for simple tables). With --rows: PDFKit's runs of text are placed by
// their position on the page; runs on one baseline form one line, left to right. That keeps rows of tables together
// even where PDFKit reads column by column.
import PDFKit

var args = Array(CommandLine.arguments.dropFirst())
let rows = args.first == "--rows"
if rows { args.removeFirst() }
guard args.count == 1, let doc = PDFDocument(url: URL(fileURLWithPath: args[0])) else {
    FileHandle.standardError.write("usage: swift pdf2txt.swift [--rows] <file.pdf>\n".data(using: .utf8)!)
    exit(1)
}

struct Piece { let x: CGFloat; let y: CGFloat; let h: CGFloat; let text: String }

for i in 0..<doc.pageCount {
    guard let page = doc.page(at: i), let text = page.string else { continue }
    if !rows {
        print(text)
        continue
    }
    // PDFKit's runs of text with their place on the page, put together again by baseline
    guard let all = page.selection(for: page.bounds(for: .mediaBox)) else { continue }
    var pieces: [Piece] = []
    for run in all.selectionsByLine() {
        let b = run.bounds(for: page)
        let s = (run.string ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
        if !s.isEmpty && b.height > 0 {
            pieces.append(Piece(x: b.minX, y: b.midY, h: b.height, text: s))
        }
    }
    var lines: [[Piece]] = []
    for p in pieces.sorted(by: { $0.y > $1.y }) {
        if let first = lines.last?.first, abs(first.y - p.y) < min(first.h, p.h) / 2 {
            lines[lines.count - 1].append(p)
        } else {
            lines.append([p])
        }
    }
    for line in lines {
        print(line.sorted(by: { $0.x < $1.x }).map { $0.text }.joined(separator: " "))
    }
    print("")
}
