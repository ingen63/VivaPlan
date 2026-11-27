// #285: text of a PDF with PDFKit (part of macOS, no further dependency), for sources whose tables the reader of
// `steuerfuesse.py` cannot take apart. Usage: swift tools/steuerdaten/pdf2txt.swift <file.pdf> > <file.txt>
import PDFKit

guard CommandLine.arguments.count == 2,
      let doc = PDFDocument(url: URL(fileURLWithPath: CommandLine.arguments[1])) else {
    FileHandle.standardError.write("usage: swift pdf2txt.swift <file.pdf>\n".data(using: .utf8)!)
    exit(1)
}
for i in 0..<doc.pageCount {
    print(doc.page(at: i)?.string ?? "")
}
