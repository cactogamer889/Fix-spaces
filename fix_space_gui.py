# -*- coding: utf-8 -*-
"""
fix_space_gui.py

Improved GUI for Fix Space (Tkinter, optional ttkbootstrap).
Features:
- Paned layout: Treeview (left) + Details (right)
- Columns: size and percent of disk
- Search/filter by name and minimum size
- Sort options
- Context menu (Abrir, Excluir, Propriedades)
- Use ttkbootstrap if installed, fallback to ttk 'clam' theme
- Helper scripts provided to create a venv and install requirements
"""

import os
import sys
import threading
import shutil
import ctypes
import uuid
import time
import tkinter as tk
from tkinter import ttk, messagebox
import tkinter.font as tkfont
from datetime import datetime

try:
    import ttkbootstrap as tb
    TB_AVAILABLE = True
except Exception:
    TB_AVAILABLE = False

import asyncio
from fix_space_async import AsyncDiskScanner
from fix_space_cache import DiskCache
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Utilities

def is_windows() -> bool:
    return os.name == 'nt'


def is_admin() -> bool:
    if not is_windows():
        return False
    try:
        return ctypes.windll.shell32.IsUserAnAdmin() != 0
    except Exception:
        return False


def relaunch_as_admin():
    if not is_windows():
        return False
    params = ' '.join([f'"{arg}"' for arg in sys.argv])
    try:
        ctypes.windll.shell32.ShellExecuteW(None, 'runas', sys.executable, params, None, 1)
        return True
    except Exception:
        return False


def get_drives():
    drives = []
    if is_windows():
        try:
            bitmask = ctypes.windll.kernel32.GetLogicalDrives()
            for i in range(26):
                if bitmask & (1 << i):
                    drives.append(f"{chr(65 + i)}:\\")
        except Exception:
            pass
    else:
        drives.append(os.path.abspath(os.sep))
    return drives


def format_size(num_bytes: int) -> str:
    try:
        num = float(num_bytes)
    except Exception:
        return '0 B'
    for unit in ['B', 'KB', 'MB', 'GB', 'TB', 'PB']:
        if num < 1024.0:
            return f"{num:3.1f} {unit}"
        num /= 1024.0
    return f"{num:.1f} PB"


def safe_scandir_has_entries(path: str) -> bool:
    try:
        with os.scandir(path) as it:
            for _ in it:
                return True
    except Exception:
        return False
    return False


def get_folder_size(path: str, stop_flag=None, update_cb=None) -> int:
    total = 0
    stack = [path]
    last_update = 0
    try:
        while stack:
            current = stack.pop()
            try:
                with os.scandir(current) as it:
                    for entry in it:
                        # check stop flag (callable or Event)
                        try:
                            if stop_flag:
                                if callable(stop_flag):
                                    if stop_flag():
                                        return total
                                else:
                                    if bool(stop_flag):
                                        return total
                        except Exception:
                            pass
                        try:
                            if entry.is_dir(follow_symlinks=False):
                                stack.append(entry.path)
                            else:
                                try:
                                    total += entry.stat(follow_symlinks=False).st_size
                                except Exception:
                                    pass
                        except Exception:
                            continue
            except Exception:
                continue
            if update_cb:
                now = time.time()
                if now - last_update > 0.5:
                    try:
                        update_cb(total)
                    except Exception:
                        pass
                    last_update = now
    except Exception:
        pass
    return total


class FixSpaceGUI:
    def __init__(self, root: tk.Tk):
        self.root = root
        if TB_AVAILABLE:
            # If using ttkbootstrap, root is a tb.Window and styling is handled by tb
            try:
                self.style = tb.Style()
            except Exception:
                self.style = ttk.Style()
        else:
            self.style = ttk.Style()
            try:
                self.style.theme_use('clam')
            except Exception:
                pass

        default_font = tkfont.nametofont("TkDefaultFont")
        default_font.configure(size=10)
        self.root.option_add("*Font", default_font)

        root.title('Fix Space - Analyze Disk (Improved)')
        root.geometry('1100x700')
        root.minsize(900, 480)

        self.path_map = {}          # iid -> full path
        self.top_entries = []       # last scanned top-level entries
        self.scanning = False
        self.stop_event = threading.Event()
        self.scan_thread = None
        self.total_disk = 1

        # Async scanner and cache
        from pathlib import Path as PathlibPath
        cache_dir = PathlibPath.home() / ".fix_space"
        cache_dir.mkdir(exist_ok=True)
        self.cache = DiskCache(str(cache_dir / "cache.db"))
        self.async_scanner = AsyncDiskScanner(self.cache, max_workers=8)
        self.scan_task = None

        # Header
        header = ttk.Frame(root)
        header.pack(side='top', fill='x', padx=8, pady=(8, 2))
        title_lbl = ttk.Label(header, text='Fix Space - Analyze Disk', font=(default_font.actual('family'), 14, 'bold'))
        title_lbl.pack(side='left')
        subtitle_lbl = ttk.Label(header, text=' Analise rápida e segura do uso de disco', foreground='#666666')
        subtitle_lbl.pack(side='left', padx=(8, 0))

        # Controls
        ctrl = ttk.Frame(root)
        ctrl.pack(side='top', fill='x', padx=8, pady=(2, 6))

        ttk.Label(ctrl, text='Unidade:').pack(side='left')
        self.drive_var = tk.StringVar()
        self.drive_combo = ttk.Combobox(ctrl, textvariable=self.drive_var, state='readonly', width=10)
        self.drive_combo['values'] = get_drives()
        if self.drive_combo['values']:
            self.drive_combo.current(0)
        self.drive_combo.pack(side='left', padx=(4, 8))

        # Caminho personalizado
        ttk.Label(ctrl, text='ou Caminho:').pack(side='left', padx=(8, 0))
        self.path_var = tk.StringVar()
        self.path_entry = ttk.Entry(ctrl, textvariable=self.path_var, width=40)
        self.path_entry.pack(side='left', padx=(4, 8))
        self.path_entry.insert(0, 'ex: C:\\Users')

        self.scan_btn = ttk.Button(ctrl, text='Analisar', command=self.start_scan)
        self.scan_btn.pack(side='left')
        self.stop_btn = ttk.Button(ctrl, text='Parar', command=self.stop_scan, state='disabled')
        self.stop_btn.pack(side='left', padx=(6, 10))

        ttk.Label(ctrl, text='Ordenar:').pack(side='left')
        self.sort_var = tk.StringVar(value='size_desc')
        self.sort_combo = ttk.Combobox(ctrl, textvariable=self.sort_var, state='readonly', width=16)
        self.sort_combo['values'] = ['size_desc', 'size_asc', 'name_az', 'name_za']
        self.sort_combo.pack(side='left', padx=(4, 8))
        self.sort_combo.bind('<<ComboboxSelected>>', lambda e: self.apply_filter())

        ttk.Label(ctrl, text='Filtro nome:').pack(side='left')
        self.filter_name = tk.StringVar()
        self.filter_entry = ttk.Entry(ctrl, textvariable=self.filter_name, width=20)
        self.filter_entry.pack(side='left', padx=(4, 8))
        self.filter_entry.bind('<Return>', lambda e: self.apply_filter())

        ttk.Label(ctrl, text='Min MB:').pack(side='left')
        self.filter_min = tk.StringVar()
        self.filter_min_entry = ttk.Entry(ctrl, textvariable=self.filter_min, width=8)
        self.filter_min_entry.pack(side='left', padx=(4, 8))
        self.filter_min_entry.bind('<Return>', lambda e: self.apply_filter())

        ttk.Button(ctrl, text='Aplicar', command=self.apply_filter).pack(side='left')
        ttk.Button(ctrl, text='Limpar', command=self.clear_filter).pack(side='left', padx=(6,0))

        # Paned layout
        paned = ttk.PanedWindow(root, orient='horizontal')
        paned.pack(fill='both', expand=True, padx=8, pady=6)

        left_frame = ttk.Frame(paned)
        right_frame = ttk.Frame(paned, width=300)
        paned.add(left_frame, weight=3)
        paned.add(right_frame, weight=1)

        # Treeview (left)
        cols = ('size', 'percent')
        self.tree = ttk.Treeview(left_frame, columns=cols, show='tree headings')
        self.tree.heading('#0', text='Pasta / Arquivo', anchor='w')
        self.tree.heading('size', text='Tamanho', anchor='e')
        self.tree.heading('percent', text='% do disco', anchor='e')
        self.tree.column('size', width=140, anchor='e')
        self.tree.column('percent', width=100, anchor='e')
        self.tree.pack(side='left', fill='both', expand=True)

        vsb = ttk.Scrollbar(left_frame, orient='vertical', command=self.tree.yview)
        self.tree.configure(yscrollcommand=vsb.set)
        vsb.pack(side='right', fill='y')

        # Tree tags/colors
        self.tree.tag_configure('very_heavy', background='#ffdddd')
        self.tree.tag_configure('heavy', background='#fff0d6')
        self.tree.tag_configure('medium', background='#fffddc')
        self.tree.tag_configure('light', background='#eaffea')

        # Bind events
        self.tree.bind('<<TreeviewOpen>>', self.on_tree_open)
        self.tree.bind('<<TreeviewSelect>>', self.on_tree_select)
        self.tree.bind('<Double-1>', self.on_double_click)
        self.tree.bind('<Button-3>', self.on_right_click)

        # Right details
        details_label = ttk.Label(right_frame, text='Detalhes', font=(default_font.actual('family'), 12, 'bold'))
        details_label.pack(anchor='nw', pady=(6,2), padx=8)

        self.usage_label = ttk.Label(right_frame, text='')
        self.usage_label.pack(anchor='nw', padx=8)
        self.usage_bar = ttk.Progressbar(right_frame, orient='horizontal', mode='determinate', length=250)
        self.usage_bar.pack(anchor='nw', padx=8, pady=(2,8))

        self.detail_text = tk.Text(right_frame, height=12, width=40, wrap='word')
        self.detail_text.pack(fill='both', expand=True, padx=8, pady=(4,8))
        self.detail_text.configure(state='disabled')

        btn_frame = ttk.Frame(right_frame)
        btn_frame.pack(fill='x', padx=8, pady=(0,8))
        ttk.Button(btn_frame, text='Abrir', command=self.open_selected).pack(side='left')
        ttk.Button(btn_frame, text='Apagar', command=self.delete_selected).pack(side='left', padx=(6,6))
        ttk.Button(btn_frame, text='Propriedades', command=self.show_properties_selected).pack(side='left')

        # Status bar
        status = ttk.Frame(root)
        status.pack(side='bottom', fill='x', padx=8, pady=(0,8))
        self.status_label = ttk.Label(status, text='Pronto.')
        self.status_label.pack(side='left')
        self.scan_progress = ttk.Progressbar(status, mode='indeterminate')
        self.scan_progress.pack(side='right', fill='x', expand=True)

    # --------- Controls and scanning
    def refresh_drives(self):
        vals = get_drives()
        self.drive_combo['values'] = vals
        if vals and self.drive_combo.get() not in vals:
            self.drive_combo.current(0)

    def start_scan(self):
        if self.scanning:
            return

        # Usar caminho personalizado se fornecido, senão usar unidade selecionada
        custom_path = self.path_var.get().strip()
        if custom_path and custom_path != 'ex: C:\\Users':
            drive = custom_path
        else:
            drive = self.drive_var.get()

        if not drive:
            messagebox.showwarning('Caminho não selecionado', 'Escolha uma unidade ou digite um caminho.')
            return

        # Validar se o caminho existe
        if not os.path.exists(drive):
            messagebox.showerror('Caminho inválido', f'O caminho não existe:\n{drive}')
            return
        # prepare
        for iid in self.tree.get_children(''):
            self.tree.delete(iid)
        self.path_map.clear()
        self.top_entries = []
        self.scanning = True
        self.stop_event.clear()
        self.scan_btn['state'] = 'disabled'
        self.stop_btn['state'] = 'normal'
        self.scan_progress.start(10)
        self.status_label['text'] = f'Analisando {drive} ...'
        try:
            total, used, free = shutil.disk_usage(drive)
            self.total_disk = total or 1
        except Exception:
            self.total_disk = 1
        self.scan_thread = threading.Thread(target=self.scan_drive, args=(drive,), daemon=True)
        self.scan_thread.start()

    def stop_scan(self):
        """Stop scanning"""
        if self.scanning:
            self.scanning = False
            self.async_scanner.stop()
            self.status_label['text'] = 'Parando análise...'

    def scan_drive(self, drive_root: str):
        """Start async scanning in background"""
        # Run async scan in thread to avoid blocking
        threading.Thread(
            target=lambda: asyncio.run(self._async_scan_drive(drive_root)),
            daemon=True
        ).start()

    async def _async_scan_drive(self, drive_root: str):
        """Async version of drive scanning"""
        try:
            entries = []
            scanned_count = 0

            def on_progress(path, total_size, count):
                nonlocal scanned_count
                scanned_count = count
                msg = f"Analisando {path} ({count} pastas)..."
                self.root.after(0, lambda: self.status_label.config(text=msg))

            # Scan root directory
            entries = await self.async_scanner.scan_directory(
                drive_root,
                progress_callback=on_progress
            )

            if not self.scanning:
                self.finish_scan('Análise interrompida.')
                return

            # Sort by size
            entries.sort(key=lambda x: x["size"], reverse=True)
            max_size = max((e["size"] for e in entries), default=1)

            def insert_items():
                for entry in entries:
                    name = entry["name"]
                    full = entry["path"]
                    size = entry["size"]
                    is_dir = entry["is_dir"]

                    # DEBUG
                    logger.info(f"Inserting: {name} is_dir={is_dir} size={size} size_gb={size/(1024**3):.2f}GB")

                    iid = str(uuid.uuid4())
                    self.path_map[iid] = full
                    tag = self.size_tag(size, max_size)
                    percent_disk = (size / self.total_disk * 100) if self.total_disk else 0
                    try:
                        self.tree.insert('', 'end', iid=iid, text=name,
                                       values=(format_size(size), f"{percent_disk:3.2f} %"), tags=(tag,))
                        if is_dir and self._has_children(full):
                            self.tree.insert(iid, 'end', iid=iid + '_dummy',
                                           text='...', values=('', ''))
                    except Exception:
                        pass

                # Update disk info
                try:
                    total, used, free = shutil.disk_usage(drive_root)
                    self.info_label.config(
                        text=f'Total: {format_size(total)}  Usado: {format_size(used)}  Livre: {format_size(free)}'
                    )
                except Exception:
                    pass

            self.root.after(0, insert_items)
            self.finish_scan(f'Análise concluída ({scanned_count} pastas).')

        except Exception as e:
            logger.exception("Error during scan")
            self.finish_scan(f'Erro: {e}')

    def _has_children(self, path: str) -> bool:
        """Check if directory has children"""
        try:
            entries = self.async_scanner.scanner.list_directory(path)
            return len([e for e in entries if e["is_dir"]]) > 0
        except:
            return False

    def finish_scan(self, message: str):
        self.scanning = False
        self.root.after(0, lambda: self.scan_progress.stop())
        self.root.after(0, lambda: self.scan_btn.config(state='normal'))
        self.root.after(0, lambda: self.stop_btn.config(state='disabled'))
        try:
            drive = self.drive_var.get()
            total, used, free = shutil.disk_usage(drive)
            perc = int((used / total) * 100) if total else 0
            self.root.after(0, lambda: self.usage_label.config(text=f'Total: {format_size(total)}  Usado: {format_size(used)}  Livre: {format_size(free)}'))
            self.root.after(0, lambda: self.usage_bar.config(value=perc, maximum=100))
        except Exception:
            pass
        self.root.after(0, lambda: self.status_label.config(text=message))

    # --------- Tree population and UI helpers
    def populate_tree(self):
        # Clear top-level items
        for iid in self.tree.get_children(''):
            self.tree.delete(iid)
        self.path_map.clear()

        entries = list(self.top_entries)

        # Apply filter
        name_filter = (self.filter_name.get() or '').strip().lower()
        min_mb = 0
        try:
            if self.filter_min.get():
                min_mb = float(self.filter_min.get())
        except Exception:
            min_mb = 0
        min_bytes = int(min_mb * 1024 * 1024)

        if name_filter:
            entries = [e for e in entries if name_filter in e.get("name", "").lower()]
        if min_bytes > 0:
            entries = [e for e in entries if e.get("size", 0) >= min_bytes]

        # Sort
        mode = self.sort_var.get()
        if mode == 'size_desc':
            entries.sort(key=lambda x: x.get("size", 0), reverse=True)
        elif mode == 'size_asc':
            entries.sort(key=lambda x: x.get("size", 0))
        elif mode == 'name_az':
            entries.sort(key=lambda x: x.get("name", "").lower())
        elif mode == 'name_za':
            entries.sort(key=lambda x: x.get("name", "").lower(), reverse=True)

        max_size = max((e.get("size", 0) for e in entries), default=1)

        for entry in entries:
            name = entry.get("name", "")
            full = entry.get("path", "")
            size = entry.get("size", 0)
            is_dir = entry.get("is_dir", False)

            iid = str(uuid.uuid4())
            self.path_map[iid] = full
            tag = self.size_tag(size, max_size)
            percent_disk = (size / self.total_disk * 100) if self.total_disk else 0
            try:
                self.tree.insert('', 'end', iid=iid, text=name, values=(format_size(size), f"{percent_disk:3.2f} %"), tags=(tag,))
                if is_dir and self._has_children(full):
                    self.tree.insert(iid, 'end', iid=iid + '_dummy', text='...', values=('', ''))
            except Exception:
                pass

    def size_tag(self, size: int, max_size: int) -> str:
        if max_size <= 0:
            return 'light'
        percent = size / max_size
        if percent >= 0.66:
            return 'very_heavy'
        if percent >= 0.33:
            return 'heavy'
        if percent >= 0.10:
            return 'medium'
        return 'light'

    def on_tree_open(self, event):
        """Load children on demand using async scanner"""
        try:
            item = self.tree.focus()
            if not item:
                return
            children = self.tree.get_children(item)
            if len(children) == 1 and children[0].endswith('_dummy'):
                try:
                    self.tree.delete(children[0])
                except Exception:
                    pass
                path = self.path_map.get(item)
                if not path:
                    return
                # Load in background thread
                threading.Thread(
                    target=lambda: asyncio.run(self._async_load_children(item, path)),
                    daemon=True
                ).start()
        except Exception:
            pass

    async def _async_load_children(self, parent_iid: str, path: str):
        """Async load children"""
        try:
            loop = asyncio.get_event_loop()
            entries = await loop.run_in_executor(
                None,
                self.async_scanner.scanner.list_directory,
                path
            )

            entries.sort(key=lambda x: x["size"], reverse=True)
            max_size = max((e["size"] for e in entries), default=1)

            def insert_children():
                for entry in entries:
                    iid = str(uuid.uuid4())
                    self.path_map[iid] = entry["path"]
                    tag = self.size_tag(entry["size"], max_size)
                    percent_disk = (entry["size"] / self.total_disk * 100) if self.total_disk else 0
                    try:
                        self.tree.insert(parent_iid, 'end', iid=iid,
                                       text=entry["name"],
                                       values=(format_size(entry["size"]), f"{percent_disk:3.2f} %"),
                                       tags=(tag,))
                        if entry["is_dir"] and self._has_children(entry["path"]):
                            self.tree.insert(iid, 'end', iid=iid + '_dummy',
                                           text='...', values=('', ''))
                    except Exception:
                        pass

            self.root.after(0, insert_children)
        except Exception as e:
            logger.exception(f"Error loading children: {e}")

    def on_tree_select(self, event):
        sel = self.tree.selection()
        if not sel:
            return
        iid = sel[0]
        path = self.path_map.get(iid, '')
        size_text = ''
        try:
            size_text = self.tree.set(iid, 'size')
        except Exception:
            size_text = ''
        self.status_label.config(text=f'Selecionado: {path}  ({size_text})')
        self.show_details(path)

    def on_double_click(self, event):
        item = self.tree.identify_row(event.y)
        if not item:
            return
        path = self.path_map.get(item)
        if not path:
            return
        try:
            if os.path.isdir(path):
                if is_windows():
                    os.startfile(path)
                else:
                    messagebox.showinfo('Abrir pasta', f'Pasta: {path}')
            else:
                try:
                    os.startfile(path)
                except Exception:
                    messagebox.showinfo('Abrir arquivo', f'Arquivo: {path}')
        except Exception as e:
            messagebox.showerror('Erro', f'Não foi possível abrir: {e}')

    def on_right_click(self, event):
        iid = self.tree.identify_row(event.y)
        if not iid:
            return
        # select clicked
        self.tree.selection_set(iid)
        path = self.path_map.get(iid)
        menu = tk.Menu(self.root, tearoff=0)
        menu.add_command(label='Abrir', command=self.open_selected)
        menu.add_command(label='Propriedades', command=self.show_properties_selected)
        menu.add_separator()
        menu.add_command(label='Apagar', command=self.delete_selected)
        try:
            menu.tk_popup(event.x_root, event.y_root)
        finally:
            menu.grab_release()

    # --------- Details / actions
    def show_details(self, path: str):
        text = ''
        if not path:
            return
        try:
            st = os.stat(path)
            text += f'Path: {path}\n'
            text += f'Tamanho (bytes): {getattr(st, "st_size", 0)}\n'
            text += f'Criado: {datetime.fromtimestamp(getattr(st, "st_ctime", 0)).isoformat()}\n'
            text += f'Modificado: {datetime.fromtimestamp(getattr(st, "st_mtime", 0)).isoformat()}\n'
            if os.path.isdir(path):
                # show number of items
                try:
                    n = len([1 for _ in os.scandir(path)])
                    text += f'Entradas: {n}\n'
                except Exception:
                    pass
        except Exception as e:
            text = f'Erro ao obter detalhes: {e}'
        self.detail_text.configure(state='normal')
        self.detail_text.delete('1.0', 'end')
        self.detail_text.insert('1.0', text)
        self.detail_text.configure(state='disabled')

    def open_selected(self):
        sel = self.tree.selection()
        if not sel:
            messagebox.showinfo('Abrir', 'Nenhum item selecionado.')
            return
        iid = sel[0]
        path = self.path_map.get(iid)
        if not path:
            return
        try:
            if os.path.isdir(path):
                if is_windows():
                    os.startfile(path)
                else:
                    messagebox.showinfo('Abrir pasta', f'Pasta: {path}')
            else:
                os.startfile(path)
        except Exception as e:
            messagebox.showerror('Erro', f'Não foi possível abrir: {e}')

    def show_properties_selected(self):
        sel = self.tree.selection()
        if not sel:
            messagebox.showinfo('Propriedades', 'Nenhum item selecionado.')
            return
        iid = sel[0]
        path = self.path_map.get(iid)
        if not path:
            return
        try:
            st = os.stat(path)
            size = getattr(st, 'st_size', 0)
            created = datetime.fromtimestamp(getattr(st, 'st_ctime', 0)).isoformat()
            modified = datetime.fromtimestamp(getattr(st, 'st_mtime', 0)).isoformat()
            msg = f'Path: {path}\nTamanho: {format_size(size)}\nCriado: {created}\nModificado: {modified}'
            messagebox.showinfo('Propriedades', msg)
        except Exception as e:
            messagebox.showerror('Erro', f'Nao foi possivel obter propriedades: {e}')

    def delete_selected(self):
        sel = self.tree.selection()
        if not sel:
            messagebox.showinfo('Excluir', 'Nenhum item selecionado.')
            return
        for iid in sel:
            path = self.path_map.get(iid)
            if not path:
                continue
            confirm = messagebox.askyesno('Confirmar exclusão', f'Deseja realmente apagar:\n{path}\n\nEsta ação não pode ser desfeita.')
            if not confirm:
                continue
            try:
                if os.path.isfile(path) or os.path.islink(path):
                    os.remove(path)
                elif os.path.isdir(path):
                    shutil.rmtree(path)
                else:
                    os.remove(path)
                try:
                    self.tree.delete(iid)
                except Exception:
                    pass
                messagebox.showinfo('Excluído', f'{path} foi removido.')
            except Exception as e:
                messagebox.showerror('Erro ao excluir', f'Não foi possível excluir {path}:\n{e}')

    # --------- Filter helpers
    def apply_filter(self):
        self.populate_tree()

    def clear_filter(self):
        self.filter_name.set('')
        self.filter_min.set('')
        self.sort_var.set('size_desc')
        self.populate_tree()


def main():
    # Attempt elevation on Windows
    if is_windows() and not is_admin():
        rel = relaunch_as_admin()
        if rel:
            sys.exit(0)
        else:
            messagebox.showwarning('Permissões', 'Aplicativo sem privilégios de administrador. Algumas pastas poderão não ser acessíveis.')

    if TB_AVAILABLE:
        root = tb.Window(themename='litera')
    else:
        root = tk.Tk()

    app = FixSpaceGUI(root)

    def on_closing():
        app.async_scanner.close()
        app.cache.close()
        root.destroy()

    root.protocol("WM_DELETE_WINDOW", on_closing)
    root.mainloop()


if __name__ == '__main__':
    main()
