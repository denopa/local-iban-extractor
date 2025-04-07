# Local IBAN Extractor

A web-based tool for extracting IBAN (International Bank Account Number) information from PDF documents. This tool runs locally on your machine, ensuring your documents never leave your computer.

## Features

- **Local Processing**: All processing happens on your machine - no data is sent to external servers
- **Multiple Detection Methods**: Uses various techniques to find IBANs with different confidence levels:
  - Label-based detection (High confidence)
  - Pattern-based detection (Medium confidence)
  - Simple pattern detection (Low confidence)
  - Fallback detection (Basic pattern matching)
- **Text Preview**: Shows the first 500 characters of extracted text from the PDF
- **Duplicate Handling**: Automatically removes duplicate IBANs, keeping the highest confidence match
- **Modern UI**: Clean, responsive interface with drag-and-drop support
- **Copy Functionality**: Easy one-click copying of found IBANs

## Prerequisites

- Rust (latest stable version) https://www.rust-lang.org/tools/install
- Cargo (comes with Rust)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/denopa/local-iban-extractor.git
cd local-iban-extractor
```

2. Build the project:
```bash
cargo build --release
```

## Usage

1. Start the server:
```
release/local-iban-extractor
```

2. Open your web browser and navigate to:
```
http://localhost:8080
```
or
```
http://127:0:0:1:8080
```

3. Either drag and drop a PDF file onto the interface or click to select one

4. View the extracted IBANs and text preview

## Supported IBAN Formats

The tool supports IBANs from various countries, including:
- France (FR)
- Germany (DE)
- Netherlands (NL)
- Belgium (BE)
- Luxembourg (LU)
- And many more...

## Security

- All processing is done locally on your machine
- No data is sent to external servers
- PDF files are processed in memory and not stored permanently

## Development

The project is built with:
- Rust for the backend
- Actix-web for the web server
- PDF-extract for PDF processing
- Vanilla JavaScript for the frontend

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 