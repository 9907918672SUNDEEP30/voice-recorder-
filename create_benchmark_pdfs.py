import os
from reportlab.lib.pagesizes import letter
from reportlab.lib import colors
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, Table, TableStyle, PageBreak
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle

OUTPUT_DIR = r"D:\ragapp"
os.makedirs(OUTPUT_DIR, exist_ok=True)

styles = getSampleStyleSheet()
title_style = styles['Heading1']
heading2 = styles['Heading2']
normal = styles['Normal']

# 1. INV_2026_VEND_9042.pdf (Vendor Invoice & GST Tax Return)
def generate_invoice():
    pdf_path = os.path.join(OUTPUT_DIR, "INV_2026_VEND_9042.pdf")
    doc = SimpleDocTemplate(pdf_path, pagesize=letter)
    story = []

    # Page 1: Tax Invoice Summary
    story.append(Paragraph("TAX INVOICE - SOVEREIGN MATRIX TECHNOLOGIES PVT LTD", title_style))
    story.append(Spacer(1, 12))
    story.append(Paragraph("<b>Invoice Number:</b> INV-2026-9042", normal))
    story.append(Paragraph("<b>Date of Issue:</b> 14-February-2026", normal))
    story.append(Paragraph("<b>GSTIN:</b> 07AAAAA0000A1Z5 | HSN Code: 998313", normal))
    story.append(Paragraph("<b>Client Name:</b> Shambhavi Enterprise Holdings Ltd", normal))
    story.append(Spacer(1, 12))
    
    data = [
        ["Description", "SAC/HSN", "Qty", "Rate (INR)", "Amount (INR)"],
        ["AI Core High-Performance Compute Cluster License", "998313", "1", "10,50,000.00", "10,50,000.00"],
        ["DirectML Vector Acceleration & Optimization", "998313", "1", "2,10,000.00", "2,10,000.00"],
        ["Zero-RAG In-Memory Matrix Deployment", "998313", "1", "2,27,350.00", "2,27,350.00"],
        ["Subtotal", "", "", "", "14,87,350.00"],
        ["CGST (9%)", "", "", "", "1,33,861.50"],
        ["SGST (9%)", "", "", "", "1,33,861.50"],
        ["Total Payable", "", "", "", "17,55,073.00"]
    ]
    t = Table(data, colWidths=[200, 70, 40, 100, 100])
    t.setStyle(TableStyle([
        ('BACKGROUND', (0,0), (-1,0), colors.navy),
        ('TEXTCOLOR', (0,0), (-1,0), colors.whitesmoke),
        ('ALIGN', (0,0), (-1,-1), 'LEFT'),
        ('GRID', (0,0), (-1,-1), 1, colors.black),
        ('FONTNAME', (0,0), (-1,0), 'Helvetica-Bold'),
    ]))
    story.append(t)
    story.append(Spacer(1, 14))
    story.append(Paragraph("<b>TDS Declaration:</b> TDS deducted under Section 194J @ 10% amounting to Rs. 1,48,735.00.", normal))
    
    # Page 2: Detailed Line Items & Bank Details
    story.append(PageBreak())
    story.append(Paragraph("ANNEXURE - BANK TRANSFER & SETTLEMENT RECEIPT", title_style))
    story.append(Spacer(1, 14))
    story.append(Paragraph("<b>Payment Mode:</b> Electronic Wire / NEFT Transfer", normal))
    story.append(Paragraph("<b>UTR Reference Number:</b> UTIB00029384729184", normal))
    story.append(Paragraph("<b>Settlement Bank:</b> Axis Bank Corporate Banking Branch", normal))
    story.append(Paragraph("<b>Beneficiary Account:</b> 921020084918239 (IFSC: UTIB0000042)", normal))
    story.append(Paragraph("<b>Status:</b> Fully Paid and Reconciled on 16-Feb-2026", normal))
    story.append(Spacer(1, 14))
    story.append(Paragraph("Certified true copy of bank transaction ledger verification for statutory audit compliance.", normal))

    doc.build(story)
    print("Generated Invoice PDF")

# 2. COMMERCIAL_LEASE_AGREEMENT_2025.pdf
def generate_lease():
    pdf_path = os.path.join(OUTPUT_DIR, "COMMERCIAL_LEASE_AGREEMENT_2025.pdf")
    doc = SimpleDocTemplate(pdf_path, pagesize=letter)
    story = []

    # Page 1: Parties
    story.append(Paragraph("COMMERCIAL LEASE DEED & PREMISES AGREEMENT", title_style))
    story.append(Spacer(1, 12))
    story.append(Paragraph("This Commercial Lease Deed is entered into on 01-November-2025 by and between:", normal))
    story.append(Paragraph("<b>Lessor:</b> DLF Prime Commercial Towers Pvt Ltd, Cyber City Gurgaon.", normal))
    story.append(Paragraph("<b>Lessee:</b> Sovereign AI Core Systems LLP.", normal))
    story.append(Paragraph("<b>Leased Premises:</b> Unit 1204, 12th Floor, Tower B, Plot 42, Cyber Hub Sector 24.", normal))
    story.append(Spacer(1, 14))

    # Page 2: Financial Terms & Lock-in
    story.append(PageBreak())
    story.append(Paragraph("FINANCIAL TERMS & LOCK-IN COVENANTS", heading2))
    story.append(Spacer(1, 12))
    story.append(Paragraph("<b>Clause 4.1 (Monthly Rent):</b> The agreed monthly lease rental is Rs. 4,50,000/- (Rupees Four Lakhs Fifty Thousand only) payable on or before the 5th day of each calendar month.", normal))
    story.append(Spacer(1, 8))
    story.append(Paragraph("<b>Clause 5.1 (Interest-Free Refundable Security Deposit):</b> The Lessee has deposited Rs. 27,00,000/- (Rupees Twenty Seven Lakhs only) equivalent to 6 months rent.", normal))
    story.append(Spacer(1, 8))
    story.append(Paragraph("<b>Clause 8.2 (Mandatory Lock-in Period):</b> The lease shall have an absolute mandatory lock-in period of 36 (thirty-six) months commencing from 01-Nov-2025. Any early exit by the Lessee shall incur an early exit penalty of 6 months gross rent (Rs. 27,00,000/-).", normal))
    story.append(Spacer(1, 14))

    # Page 3: Dispute Resolution
    story.append(PageBreak())
    story.append(Paragraph("GOVERNING LAW & ARBITRATION", heading2))
    story.append(Spacer(1, 12))
    story.append(Paragraph("<b>Clause 14.1 (Arbitration):</b> Any dispute arising out of this agreement shall be referred to a Sole Arbitrator appointed mutually in New Delhi under the Arbitration and Conciliation Act, 1996.", normal))
    story.append(Paragraph("<b>Clause 14.2 (Jurisdiction):</b> Courts of New Delhi shall have exclusive jurisdiction over all interim measures.", normal))

    doc.build(story)
    print("Generated Lease PDF")

# 3. QUARTERLY_FINANCIAL_EBITDA_Q3_2025.pdf
def generate_ebitda():
    pdf_path = os.path.join(OUTPUT_DIR, "QUARTERLY_FINANCIAL_EBITDA_Q3_2025.pdf")
    doc = SimpleDocTemplate(pdf_path, pagesize=letter)
    story = []

    # Page 1: Executive Summary
    story.append(Paragraph("Q3 2025 EXECUTIVE FINANCIAL PERFORMANCE REPORT", title_style))
    story.append(Spacer(1, 12))
    story.append(Paragraph("<b>Consolidated Operating Revenue:</b> INR 42.8 Crore (28.4% YoY Expansion)", normal))
    story.append(Paragraph("<b>Reporting Period:</b> October 1, 2025 to December 31, 2025", normal))
    story.append(Spacer(1, 12))
    story.append(Paragraph("The third quarter demonstrated extraordinary resilience propelled by enterprise software licensing, local LLM vector deployment contracts, and offline zero-RAG audit matrix adoption.", normal))

    # Page 2: EBITDA Margins & R&D
    story.append(PageBreak())
    story.append(Paragraph("EBITDA MARGINS & R&D CAPITAL ALLOCATION", heading2))
    story.append(Spacer(1, 12))
    story.append(Paragraph("<b>EBITDA Margin:</b> 31.4% with Operating Profit reaching INR 13.44 Crore.", normal))
    story.append(Spacer(1, 8))
    story.append(Paragraph("<b>R&D Capital Expenditure:</b> INR 6.2 Crore allocated strictly towards Local Vision Models (Qwen2-VL INT4 quantization) and Rayon Parallel In-Memory Traversal engines.", normal))
    story.append(Paragraph("<b>Debt-to-Equity Ratio:</b> 0.04 (Virtually Zero Debt Sovereign Balance Sheet).", normal))

    doc.build(story)
    print("Generated EBITDA PDF")

# 4. SCAN_DOC_9921_MISC_TECH.pdf
def generate_cryptic():
    pdf_path = os.path.join(OUTPUT_DIR, "SCAN_DOC_9921_MISC_TECH.pdf")
    doc = SimpleDocTemplate(pdf_path, pagesize=letter)
    story = []

    # Page 1: Kernel Ring-0 Security
    story.append(Paragraph("TECHNICAL SPECIFICATION: HARDWARE-BOUND DRM ENCLAVE", title_style))
    story.append(Spacer(1, 12))
    story.append(Paragraph("<b>Internal Document Tag:</b> SPEC-RING0-SEC-9921", normal))
    story.append(Paragraph("<b>Motherboard Serial & CPU ID Binding:</b> Derives a non-exportable 256-bit AES encryption key in memory via SHA-256 double-salt hashing.", normal))
    story.append(Paragraph("<b>Anti-Tamper Latency:</b> Zeroization trigger terminates process within 4.2 milliseconds upon debugger attachment or binary hash mismatch.", normal))

    # Page 2: Key Rotation Interval
    story.append(PageBreak())
    story.append(Paragraph("AUTHENTICATED CIPHER ROTATION & AUDIT DISK FORMAT", heading2))
    story.append(Spacer(1, 12))
    story.append(Paragraph("<b>Key Rotation Interval:</b> Fixed at exactly 86,400 seconds (24 hours).", normal))
    story.append(Paragraph("<b>Audit Trail Chaining:</b> Chained SHA-256 blocks written to audit_log.bin. Any modification to a historical record invalidates the entire block ledger.", normal))

    doc.build(story)
    print("Generated Cryptic Tech PDF")

if __name__ == "__main__":
    generate_invoice()
    generate_lease()
    generate_ebitda()
    generate_cryptic()
    print("All 4 PDFs generated successfully in:", OUTPUT_DIR)
