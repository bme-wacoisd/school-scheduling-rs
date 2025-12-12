#!/usr/bin/env python3
"""
Generate PDF from FEA schedule markdown files.
Output goes to output/fea/ which is gitignored (contains student PII).
"""

from fpdf import FPDF
from pathlib import Path
import re
import sys

class MarkdownPDF(FPDF):
    def __init__(self):
        super().__init__()
        self.add_page()
        self.set_auto_page_break(auto=True, margin=15)

    def header(self):
        self.set_font('Helvetica', 'I', 8)
        self.set_text_color(128, 128, 128)
        self.cell(0, 10, 'FEA Semester 2 Scheduling Report - CONFIDENTIAL', align='C')
        self.ln(10)

    def footer(self):
        self.set_y(-15)
        self.set_font('Helvetica', 'I', 8)
        self.set_text_color(128, 128, 128)
        self.cell(0, 10, f'Page {self.page_no()}', align='C')

    def chapter_title(self, title):
        self.set_font('Helvetica', 'B', 16)
        self.set_text_color(26, 54, 93)  # Dark blue
        self.cell(0, 10, title, ln=True)
        self.set_draw_color(26, 54, 93)
        self.line(10, self.get_y(), 200, self.get_y())
        self.ln(5)

    def section_title(self, title):
        self.set_font('Helvetica', 'B', 13)
        self.set_text_color(44, 82, 130)
        self.ln(3)
        self.cell(0, 8, title, ln=True)
        self.ln(2)

    def subsection_title(self, title):
        self.set_font('Helvetica', 'B', 11)
        self.set_text_color(43, 108, 176)
        self.ln(2)
        self.cell(0, 7, title, ln=True)
        self.ln(1)

    def body_text(self, text):
        self.set_font('Helvetica', '', 10)
        self.set_text_color(0, 0, 0)
        self.set_x(10)  # Reset x position
        # Handle bold text
        text = self.process_inline_formatting(text)
        self.multi_cell(0, 5, text)
        self.ln(2)

    def process_inline_formatting(self, text):
        # Remove markdown bold/italic markers for plain text
        text = re.sub(r'\*\*(.+?)\*\*', r'\1', text)
        text = re.sub(r'\*(.+?)\*', r'\1', text)
        text = re.sub(r'`(.+?)`', r'\1', text)
        return text

    def bullet_item(self, text, indent=0):
        self.set_font('Helvetica', '', 10)
        self.set_text_color(0, 0, 0)
        self.set_x(10)  # Reset first
        text = self.process_inline_formatting(text)
        self.multi_cell(0, 5, f"  * {text}")

    def numbered_item(self, num, text, indent=0):
        self.set_font('Helvetica', '', 10)
        self.set_text_color(0, 0, 0)
        self.set_x(10)  # Reset first
        text = self.process_inline_formatting(text)
        self.multi_cell(0, 5, f"  {num}. {text}")

    def add_table(self, headers, rows):
        self.set_font('Helvetica', 'B', 8)
        self.set_fill_color(237, 242, 247)
        self.set_text_color(0, 0, 0)

        # Calculate column widths based on content
        num_cols = len(headers)
        if num_cols == 0:
            return

        # Use available width, minimum 15 per column
        available_width = 190
        col_width = max(15, available_width / num_cols)

        # If too many columns, skip table and print as text
        if num_cols > 6:
            self.set_font('Helvetica', '', 9)
            self.multi_cell(0, 5, " | ".join(headers))
            for row in rows:
                self.multi_cell(0, 5, " | ".join(str(c) for c in row))
            self.ln(3)
            return

        # Headers
        for header in headers:
            text = str(header)[:20] if len(str(header)) > 20 else str(header)
            self.cell(col_width, 6, text, border=1, fill=True, align='C')
        self.ln()

        # Rows
        self.set_font('Helvetica', '', 8)
        fill = False
        for row in rows:
            if fill:
                self.set_fill_color(247, 250, 252)
            else:
                self.set_fill_color(255, 255, 255)
            for i, cell in enumerate(row):
                if i >= num_cols:
                    break
                cell_text = str(cell)[:20] if cell else ''
                self.cell(col_width, 5, cell_text, border=1, fill=True)
            # Fill remaining cells if row is short
            for _ in range(num_cols - len(row)):
                self.cell(col_width, 5, '', border=1, fill=True)
            self.ln()
            fill = not fill
        self.ln(3)
        self.set_x(10)  # Reset x position

    def horizontal_rule(self):
        self.ln(5)
        self.set_draw_color(200, 200, 200)
        self.line(10, self.get_y(), 200, self.get_y())
        self.ln(5)

    def add_markdown(self, content):
        """Parse and add markdown content to PDF."""
        lines = content.split('\n')
        i = 0
        in_table = False
        table_headers = []
        table_rows = []

        while i < len(lines):
            line = lines[i].strip()

            # Skip empty lines
            if not line:
                if in_table and table_headers:
                    self.add_table(table_headers, table_rows)
                    in_table = False
                    table_headers = []
                    table_rows = []
                i += 1
                continue

            # Horizontal rule
            if line == '---' or line == '***':
                if in_table and table_headers:
                    self.add_table(table_headers, table_rows)
                    in_table = False
                    table_headers = []
                    table_rows = []
                self.add_page()
                i += 1
                continue

            # Headers
            if line.startswith('# '):
                self.chapter_title(line[2:])
                i += 1
                continue
            if line.startswith('## '):
                self.section_title(line[3:])
                i += 1
                continue
            if line.startswith('### '):
                self.subsection_title(line[4:])
                i += 1
                continue

            # Tables
            if '|' in line and not line.startswith('|--'):
                cells = [c.strip() for c in line.split('|') if c.strip()]
                if not in_table:
                    table_headers = cells
                    in_table = True
                    # Skip separator line
                    if i + 1 < len(lines) and '|--' in lines[i + 1]:
                        i += 1
                else:
                    table_rows.append(cells)
                i += 1
                continue

            # Skip table separator lines
            if '|--' in line or '|:' in line:
                i += 1
                continue

            # Bullet points
            if line.startswith('- ') or line.startswith('* '):
                self.bullet_item(line[2:])
                i += 1
                continue

            # Numbered lists
            match = re.match(r'^(\d+)\.\s+(.+)', line)
            if match:
                self.numbered_item(match.group(1), match.group(2))
                i += 1
                continue

            # Checkbox items
            if line.startswith('- [ ]') or line.startswith('- [x]'):
                self.bullet_item(line[6:])
                i += 1
                continue

            # Regular paragraph
            self.body_text(line)
            i += 1

        # Handle remaining table
        if in_table and table_headers:
            self.add_table(table_headers, table_rows)


def main():
    output_dir = Path("output/fea")

    # Files in order
    md_files = [
        "0-EXECUTIVE-SUMMARY.md",
        "1-technical-explanation.md",
        "2-assumptions.md",
        "3-schedule-minimal.md",
        "4-schedule-optimal.md",
        "5-rosters.md",
    ]

    # Check all files exist
    for f in md_files:
        if not (output_dir / f).exists():
            print(f"ERROR: {f} not found in {output_dir}")
            return 1

    # Create PDF
    pdf = MarkdownPDF()

    for f in md_files:
        print(f"Processing: {f}")
        content = (output_dir / f).read_text(encoding="utf-8")
        pdf.add_markdown(content)

    # Add confidentiality notice at end
    pdf.add_page()
    pdf.set_font('Helvetica', 'B', 14)
    pdf.set_text_color(180, 0, 0)
    pdf.cell(0, 10, 'CONFIDENTIAL - STUDENT PII', ln=True, align='C')
    pdf.ln(5)
    pdf.set_font('Helvetica', '', 11)
    pdf.set_text_color(0, 0, 0)
    pdf.multi_cell(0, 6,
        "This document contains personally identifiable information (PII) about students. "
        "Handle according to FERPA guidelines. Do not share outside authorized personnel. "
        "Do not upload to public repositories or cloud storage without encryption.\n\n"
        "Generated: December 12, 2025\n"
        "FEA Semester 2 Scheduling Report"
    )

    # Save PDF
    output_pdf = output_dir / "FEA-Schedule-Report.pdf"
    pdf.output(str(output_pdf))

    print(f"\nSUCCESS: PDF created at {output_pdf}")
    print(f"File size: {output_pdf.stat().st_size / 1024:.1f} KB")

    return 0

if __name__ == "__main__":
    sys.exit(main())
