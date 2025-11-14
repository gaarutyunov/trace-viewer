# Playwright Trace Viewer - Rust Edition

A browser-based Playwright trace viewer built with Rust and Yew (WebAssembly).

## Features

- 📁 **File Drop Support**: Drag and drop Playwright trace ZIP files directly into the browser
- 🚀 **Pure Client-Side**: All processing happens in your browser using WebAssembly - no server needed
- 🔍 **Action Timeline**: View all test actions with timing information
- 📊 **Detailed View**: Inspect individual actions with parameters, errors, and logs
- 🎨 **Modern UI**: Clean, dark-themed interface optimized for trace analysis
- 🔒 **Privacy First**: Your trace data never leaves your browser

## What is this?

This is a Rust/WebAssembly reimplementation of the official [Playwright trace viewer](https://github.com/microsoft/playwright/tree/main/packages/trace-viewer), designed to run entirely in the browser without requiring Node.js or a development server.

## Building

### Prerequisites

1. Install Rust: https://rustup.rs/
2. Install trunk: `cargo install trunk`
3. Add WASM target: `rustup target add wasm32-unknown-unknown`

### Development

```bash
# Build and serve with hot reload
trunk serve

# Open browser to http://127.0.0.1:8080
```

### Production Build

```bash
# Build optimized WASM bundle
trunk build --release

# Output will be in ./dist/
```

## Usage

1. Open the application in your browser
2. Either:
   - Drag and drop a Playwright trace ZIP file onto the drop zone
   - Click "Select File" to browse for a trace file
3. View the parsed trace data:
   - Browse actions in the left panel
   - Click an action to view details in the right panel
   - See timing, parameters, errors, and logs

## Project Structure

```
src/
├── lib.rs                    # Main application entry point
├── models.rs                 # Trace data structures
├── trace_loader.rs           # ZIP parsing and trace loading
└── components/
    ├── mod.rs               # Component exports
    ├── file_drop_zone.rs    # File upload interface
    ├── trace_viewer.rs      # Main viewer component
    ├── action_list.rs       # Action timeline list
    └── action_details.rs    # Action detail panel
```

## Credits

Based on the official [Playwright Trace Viewer](https://github.com/microsoft/playwright) by Microsoft Corporation.

Built with [Yew](https://yew.rs/), [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen), and [gloo](https://github.com/rustwasm/gloo)
