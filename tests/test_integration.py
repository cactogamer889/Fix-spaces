import asyncio
import os
import tempfile
from pathlib import Path
import sys
import pytest
sys.path.insert(0, str(Path(__file__).parent.parent))
from fix_space_async import AsyncDiskScanner

@pytest.mark.asyncio
async def test_async_scan_with_progress():
    """Test async scanning with progress callback"""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create test structure
        Path(tmpdir, "file1.txt").write_text("a" * 1000)
        os.makedirs(os.path.join(tmpdir, "subdir"))
        Path(tmpdir, "subdir", "file2.txt").write_text("b" * 2000)

        scanner = AsyncDiskScanner()
        progress_updates = []

        def on_progress(path, size, count):
            progress_updates.append({"path": path, "size": size, "count": count})

        result = await scanner.scan_directory(tmpdir, progress_callback=on_progress)

        # Should have entries and progress updates
        assert len(result) > 0, "No entries returned"
        assert len(progress_updates) > 0, "No progress updates"

        scanner.close()

def test_async_scan_sync_wrapper():
    """Test async scanner via sync wrapper (for pytest compatibility)"""
    with tempfile.TemporaryDirectory() as tmpdir:
        Path(tmpdir, "file1.txt").write_text("a" * 1000)
        os.makedirs(os.path.join(tmpdir, "subdir"))
        Path(tmpdir, "subdir", "file2.txt").write_text("b" * 2000)

        scanner = AsyncDiskScanner()
        result = asyncio.run(scanner.scan_directory(tmpdir))

        assert len(result) >= 2, f"Expected at least 2 entries, got {len(result)}"

        scanner.close()
