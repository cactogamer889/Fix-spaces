import os
import tempfile
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).parent.parent))
from fix_space_scanner import WindowsScanner

def test_scanner_enumerate_directory():
    """Test that scanner can enumerate directory contents"""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create test structure
        Path(tmpdir, "file1.txt").write_text("a" * 100)
        Path(tmpdir, "file2.txt").write_text("b" * 200)
        os.makedirs(os.path.join(tmpdir, "subdir"))
        Path(tmpdir, "subdir", "file3.txt").write_text("c" * 300)

        scanner = WindowsScanner()
        entries = scanner.list_directory(tmpdir)

        # Should have 2 files + 1 directory
        assert len(entries) == 3, f"Expected 3 entries, got {len(entries)}"

        # Find each entry
        files = {e["name"]: e for e in entries}
        assert "file1.txt" in files
        assert "file2.txt" in files
        assert "subdir" in files

        # Check sizes
        assert files["file1.txt"]["size"] == 100
        assert files["file2.txt"]["size"] == 200
        assert files["subdir"]["is_dir"] == True

def test_scanner_folder_size():
    """Test recursive folder size calculation"""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create test structure
        Path(tmpdir, "file1.txt").write_text("a" * 100)
        os.makedirs(os.path.join(tmpdir, "subdir"))
        Path(tmpdir, "subdir", "file2.txt").write_text("b" * 200)
        os.makedirs(os.path.join(tmpdir, "subdir", "deep"))
        Path(tmpdir, "subdir", "deep", "file3.txt").write_text("c" * 300)

        scanner = WindowsScanner()
        total_size = scanner.get_folder_size(tmpdir)

        # Total should be 100 + 200 + 300 = 600
        assert total_size == 600, f"Expected 600, got {total_size}"
