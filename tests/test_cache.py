import os
import tempfile
import sqlite3
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).parent.parent))
from fix_space_cache import DiskCache

def test_cache_initialization():
    """Test that cache DB is created and initialized"""
    with tempfile.TemporaryDirectory() as tmpdir:
        cache_path = os.path.join(tmpdir, "test.db")
        cache = DiskCache(cache_path)

        # Verify database file exists
        assert os.path.exists(cache_path), "Cache DB file not created"

        # Verify table exists
        conn = sqlite3.connect(cache_path)
        cursor = conn.cursor()
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='entries'")
        result = cursor.fetchone()
        assert result is not None, "entries table not created"
        conn.close()

        cache.close()

def test_cache_set_and_get():
    """Test setting and retrieving cache entries"""
    with tempfile.TemporaryDirectory() as tmpdir:
        cache_path = os.path.join(tmpdir, "test.db")
        cache = DiskCache(cache_path)

        # Set an entry
        cache.set("/test/path", 1024, True, 1234567890.0)

        # Get the entry
        result = cache.get("/test/path")
        assert result is not None, "Entry not found"
        assert result["size"] == 1024, "Size mismatch"
        assert result["is_dir"] == True, "is_dir mismatch"

        cache.close()

def test_cache_validation_with_mtime():
    """Test that cache validates entries against file mtime"""
    with tempfile.TemporaryDirectory() as tmpdir:
        cache_path = os.path.join(tmpdir, "test.db")
        cache = DiskCache(cache_path)
        test_file = os.path.join(tmpdir, "test.txt")

        # Create test file
        Path(test_file).write_text("test")
        mtime = os.path.getmtime(test_file)

        # Cache it
        cache.set(test_file, 100, False, mtime)

        # Should be valid
        assert cache.is_valid(test_file) == True, "Valid entry marked invalid"

        # Modify file (update mtime)
        import time
        time.sleep(0.1)
        Path(test_file).write_text("test2")

        # Should be invalid now
        assert cache.is_valid(test_file) == False, "Invalid entry marked valid"

        cache.close()
