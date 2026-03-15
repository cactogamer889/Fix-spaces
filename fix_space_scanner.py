# -*- coding: utf-8 -*-
"""
Fast disk scanner using Windows native APIs via ctypes.
Uses FindFirstFileW/FindNextFileW for enumeration (faster than os.walk).
"""

import ctypes
import os
from ctypes import wintypes, byref, create_unicode_buffer
from typing import List, Dict, Optional
import logging

logger = logging.getLogger(__name__)

# Windows API constants
INVALID_HANDLE_VALUE = -1
FILE_ATTRIBUTE_DIRECTORY = 0x10
FILE_ATTRIBUTE_SYSTEM = 0x04
FILE_ATTRIBUTE_HIDDEN = 0x02
FILE_ATTRIBUTE_REPARSE_POINT = 0x400  # Junctions, symlinks, etc

# ctypes function signatures
FindFirstFileW = ctypes.windll.kernel32.FindFirstFileW
FindFirstFileW.argtypes = [wintypes.LPWSTR, ctypes.POINTER(ctypes.c_char * 592)]
FindFirstFileW.restype = wintypes.HANDLE

FindNextFileW = ctypes.windll.kernel32.FindNextFileW
FindNextFileW.argtypes = [wintypes.HANDLE, ctypes.POINTER(ctypes.c_char * 592)]
FindNextFileW.restype = wintypes.BOOL

FindClose = ctypes.windll.kernel32.FindClose
FindClose.argtypes = [wintypes.HANDLE]
FindClose.restype = wintypes.BOOL

class WindowsScanner:
    """Uses Windows native APIs for fast directory enumeration"""

    def __init__(self, hide_system_files: bool = True):
        """Initialize scanner

        Args:
            hide_system_files: If True, skip Windows system files like desktop.ini
        """
        self.max_path = 260
        self.hide_system_files = hide_system_files

    def list_directory(self, path: str) -> List[Dict]:
        """
        List directory contents using Windows APIs.
        Returns list of dicts: {"name": str, "size": int, "is_dir": bool, "path": str}
        """
        entries = []

        # Add wildcard for FindFirstFile
        search_path = os.path.join(path, "*")

        # WIN32_FIND_DATAW structure (592 bytes)
        find_data = ctypes.create_string_buffer(592)

        try:
            handle = FindFirstFileW(search_path, byref(find_data))

            if handle == INVALID_HANDLE_VALUE:
                logger.warning(f"Cannot enumerate {path}")
                return entries

            try:
                while True:
                    # Parse WIN32_FIND_DATAW structure (correct offsets)
                    # Offset 0: dwFileAttributes (4 bytes)
                    # Offset 28: nFileSizeHigh (4 bytes)
                    # Offset 32: nFileSizeLow (4 bytes)
                    # Offset 44: cFileName (260 wide chars = 520 bytes)

                    attributes = int.from_bytes(find_data[0:4], 'little')
                    size_high = int.from_bytes(find_data[28:32], 'little')
                    size_low = int.from_bytes(find_data[32:36], 'little')
                    filename_bytes = find_data[44:304]  # 44 + (260*2) = 44 + 520 = 564, use 44:304 for 260 chars

                    # Extract filename (null-terminated unicode)
                    filename = filename_bytes.decode('utf-16le', errors='ignore').split('\0')[0]

                    # Skip . and ..
                    if filename in ('.', '..'):
                        if not FindNextFileW(handle, byref(find_data)):
                            break
                        continue

                    # Skip Windows system files, junctions, symlinks if hide_system_files is enabled
                    is_system = bool(attributes & FILE_ATTRIBUTE_SYSTEM)
                    is_hidden = bool(attributes & FILE_ATTRIBUTE_HIDDEN)
                    is_dir = bool(attributes & FILE_ATTRIBUTE_DIRECTORY)
                    is_reparse = bool(attributes & FILE_ATTRIBUTE_REPARSE_POINT)  # Junctions, symlinks

                    # Skip system files and reparse points (junctions/symlinks) always
                    # But only skip hidden files if they're not directories
                    # (we need to see hidden folders like AppData)
                    if self.hide_system_files:
                        if is_system or is_reparse:
                            if not FindNextFileW(handle, byref(find_data)):
                                break
                            continue
                        # Only hide hidden FILES, not hidden DIRECTORIES
                        if is_hidden and not is_dir:
                            if not FindNextFileW(handle, byref(find_data)):
                                break
                            continue

                    size = (size_high << 32) | size_low
                    full_path = os.path.join(path, filename)

                    entries.append({
                        "name": filename,
                        "size": size,
                        "is_dir": is_dir,
                        "path": full_path
                    })

                    if not FindNextFileW(handle, byref(find_data)):
                        break
            finally:
                FindClose(handle)

        except Exception as e:
            logger.warning(f"Error enumerating {path}: {e}")

        return entries

    def get_folder_size(self, path: str, use_cache=None) -> int:
        """
        Calculate total folder size recursively.
        Can use cache if provided.
        """
        total = 0

        try:
            entries = self.list_directory(path)

            for entry in entries:
                if entry["is_dir"]:
                    # Recursively sum subdirectories
                    total += self.get_folder_size(entry["path"], use_cache)
                else:
                    total += entry["size"]

        except Exception as e:
            logger.warning(f"Error calculating folder size for {path}: {e}")

        return total
