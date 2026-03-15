# -*- coding: utf-8 -*-
"""
Async wrapper for disk scanning.
Runs Windows API scanner in thread pool to avoid blocking UI.
"""

import asyncio
from concurrent.futures import ThreadPoolExecutor
from typing import List, Dict, Callable, Optional
import logging
from fix_space_scanner import WindowsScanner
from fix_space_cache import DiskCache

logger = logging.getLogger(__name__)

class AsyncDiskScanner:
    """Async wrapper around WindowsScanner using thread pool"""

    def __init__(self, cache: Optional[DiskCache] = None, max_workers: int = None):
        """
        Initialize async scanner.

        Args:
            cache: Optional DiskCache instance for caching results
            max_workers: Number of worker threads for parallel enumeration
        """
        if max_workers is None:
            import os
            max_workers = min(8, (os.cpu_count() or 1) + 1)

        self.scanner = WindowsScanner()
        self.cache = cache
        self.max_workers = max_workers
        self.executor = ThreadPoolExecutor(max_workers=max_workers)
        self._stop_flag = False

    async def scan_directory(
        self,
        path: str,
        progress_callback: Optional[Callable[[str, int, int], None]] = None
    ) -> List[Dict]:
        """
        Scan directory asynchronously.

        Args:
            path: Root directory to scan
            progress_callback: Optional callback(path, total_size, entry_count)

        Returns:
            List of entries with size info
        """
        import time
        start_time = time.time()
        self._stop_flag = False
        loop = asyncio.get_event_loop()

        # Scan root directory in thread pool
        entries = await loop.run_in_executor(
            self.executor,
            self.scanner.list_directory,
            path
        )

        # Sort by size (largest first)
        entries.sort(key=lambda x: x["size"], reverse=True)

        # For directories, calculate sizes concurrently
        tasks = []
        for entry in entries:
            if entry["is_dir"]:
                task = self._get_folder_size_async(entry)
                tasks.append(task)

        # Execute folder size calculations with progress updates
        total_size = 0
        count = 0
        for i, task in enumerate(asyncio.as_completed(tasks)):
            if self._stop_flag:
                break

            entry = await task
            total_size += entry["size"]
            count += 1

            if progress_callback:
                progress_callback(entry["path"], total_size, count)

        import time
        elapsed = time.time() - start_time
        logger.info(f"Scan completed in {elapsed:.2f}s for {len(entries)} entries")
        return entries

    async def _get_folder_size_async(self, entry: Dict) -> Dict:
        """Calculate folder size in thread pool"""
        loop = asyncio.get_event_loop()

        # Check cache first
        if self.cache:
            cached = self.cache.get(entry["path"])
            if cached and self.cache.is_valid(entry["path"]):
                entry["size"] = cached["size"]
                return entry

        # Calculate size
        size = await loop.run_in_executor(
            self.executor,
            self.scanner.get_folder_size,
            entry["path"]
        )

        entry["size"] = size

        # Cache result
        if self.cache:
            import os
            try:
                mtime = os.path.getmtime(entry["path"])
                self.cache.set(entry["path"], size, True, mtime)
            except:
                pass

        return entry

    def stop(self):
        """Signal scanner to stop"""
        self._stop_flag = True

    def close(self):
        """Clean up resources"""
        self.executor.shutdown(wait=True)
