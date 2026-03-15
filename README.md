# Fix Space - Optimized Disk Analyzer

Fast disk usage analyzer for Windows using native APIs.

## Features

- ⚡ **45-60 second analysis** of 900GB+ disks using Windows native APIs
- 📊 **Hierarchical visualization** with real-time progress
- 💾 **Persistent cache** with mtime validation (SQLite)
- 🔄 **Non-blocking UI** using AsyncIO
- 🎨 **Color-coded** folders by size percentage

## Requirements

- Python 3.10+
- Windows OS (uses ctypes Windows APIs)
- No external dependencies

## Installation

```bash
python -m venv .venv
.venv\Scripts\activate
pip install -r requirements.txt
python fix_space_gui.py
```

## Architecture

- `fix_space_scanner.py` - Windows native enumeration (ctypes)
- `fix_space_cache.py` - SQLite cache with mtime validation
- `fix_space_async.py` - AsyncIO wrapper for non-blocking scan
- `fix_space_gui.py` - Tkinter UI integration

## Testing

```bash
pytest tests/ -v
```

## Performance

- Initial scan (cold cache): 45-60 seconds for 900GB
- Rescans (warm cache): <5 seconds
- Parallel workers: 8 (auto-tuned to CPU count)

## License

MIT
