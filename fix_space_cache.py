# -*- coding: utf-8 -*-
"""
Cache layer for disk analysis results.
Uses SQLite to persist folder sizes and validate with mtime.
"""

import sqlite3
import os
import time
from pathlib import Path
from typing import Optional, Dict

class DiskCache:
    """SQLite-based cache for folder sizes and file info"""

    def __init__(self, cache_path: str = None):
        """Initialize cache with SQLite database"""
        if cache_path is None:
            cache_dir = Path.home() / ".fix_space"
            cache_dir.mkdir(exist_ok=True)
            cache_path = str(cache_dir / "cache.db")

        self.cache_path = cache_path
        self.conn = sqlite3.connect(cache_path)
        self._init_db()

    def _init_db(self):
        """Create tables if they don't exist"""
        cursor = self.conn.cursor()
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS entries (
                path TEXT PRIMARY KEY,
                size INTEGER,
                is_dir INTEGER,
                mtime REAL,
                cached_at REAL
            )
        """)
        self.conn.commit()

    def set(self, path: str, size: int, is_dir: bool, mtime: float):
        """Cache an entry"""
        cursor = self.conn.cursor()
        cursor.execute("""
            INSERT OR REPLACE INTO entries (path, size, is_dir, mtime, cached_at)
            VALUES (?, ?, ?, ?, ?)
        """, (path, size, int(is_dir), mtime, time.time()))
        self.conn.commit()

    def get(self, path: str) -> Optional[Dict]:
        """Retrieve a cached entry"""
        cursor = self.conn.cursor()
        cursor.execute("SELECT size, is_dir, mtime FROM entries WHERE path = ?", (path,))
        row = cursor.fetchone()
        if row:
            return {"size": row[0], "is_dir": bool(row[1]), "mtime": row[2]}
        return None

    def is_valid(self, path: str) -> bool:
        """Check if cache entry is still valid (mtime hasn't changed)"""
        if not os.path.exists(path):
            return False

        entry = self.get(path)
        if entry is None:
            return False

        current_mtime = os.path.getmtime(path)
        return abs(current_mtime - entry["mtime"]) < 0.01  # Allow 10ms tolerance

    def close(self):
        """Close database connection"""
        self.conn.close()

    def get_stats(self) -> dict:
        """Get cache statistics"""
        cursor = self.conn.cursor()
        cursor.execute("SELECT COUNT(*) FROM entries")
        count = cursor.fetchone()[0]
        cursor.execute("SELECT SUM(size) FROM entries")
        total_size = cursor.fetchone()[0] or 0
        return {"entries": count, "total_cached_size": total_size}
