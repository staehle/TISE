"""TISE: Terra Invicta Save Editor
Created by Jake Staehle <jacob@staehle.us>
"""
import argparse
import logging
import json
import re
from typing import Any, Dict, List, Tuple, Optional, Union
from pathlib import Path
from tkinter import (
    filedialog,
    ttk,
    Menu,
    Tk,
    Label,
    Radiobutton,
    BooleanVar,
    IntVar,
    DoubleVar,
    StringVar,
    Listbox,
    Message,
    PanedWindow,
    PhotoImage,
    Entry,
    Toplevel,
    Text,
    Button,
)

# from tkinter import TclError
from tkinter.font import Font
import webbrowser

from . import statics as s


def pretty_group(groupname: str) -> str:
    """Strip the namespace from from group name and return"""
    return groupname.replace(s.COMMON_NAMESPACE, "", 1)


def event_treeview_column_sort(tktv: ttk.Treeview, col, reverse: bool):
    """on column header click, sort by values"""
    newkeys = [(tktv.set(key, col), key) for key in tktv.get_children("")]
    newkeys.sort(key=lambda a: str(a[0]).casefold(), reverse=reverse)
    # rearrange items in sorted positions
    for index, (_, key) in enumerate(newkeys):
        tktv.move(key, "", index)

    # reverse sort next time
    tktv.heading(col, command=lambda _col=col: event_treeview_column_sort(tktv, col, not reverse))


def preprocess_dict(indict):
    """Recursive append __dummy__ val to any empty dicts"""
    if isinstance(indict, dict):
        if not indict:
            indict["__dummy__"] = 1
        else:
            for key, val in indict.items():
                if isinstance(val, dict):
                    if len(val) == 0:  # if the dict is empty
                        indict[key] = {"__dummy__": 1}
                    else:
                        preprocess_dict(val)
                elif isinstance(val, list):
                    for item in val:
                        preprocess_dict(item)
    elif isinstance(indict, list):
        for item in indict:
            preprocess_dict(item)
    return indict


def hackify_json(indict: dict) -> str:
    """For some reason, official save files have empty "{}" on multiple lines, not single ones"""
    mydict = preprocess_dict(indict.copy())
    raw = json.dumps(mydict, cls=UpperCaseEFloatEncoder, ensure_ascii=True, indent=4)
    lines = raw.split("\n")
    processed_lines = []
    for line in lines:
        if line.count("__dummy__") == 0:
            processed_lines.append(line)
        else:
            processed_lines.append("")
    return "\n".join(processed_lines)


class UpperCaseEFloatEncoder(json.JSONEncoder):
    """For some reason, official save files use uppercase E for scientific notation floats"""

    float_pattern = re.compile(r"\d+\.?\d*e[-+]\d+")

    def iterencode(self, o, _one_shot=False):
        for chunk in super().iterencode(o, _one_shot):
            yield self.float_pattern.sub(lambda match: match.group(0).replace("e", "E"), chunk)


class RelationalReference:
    """JSON relational reference type"""

    def __init__(self, value: dict):
        self.reference = -1
        if s.RVAL in value.keys():
            if len(value.keys()) == 1 or (len(value.keys()) == 2 and s.RTYPE in value.keys()):
                self.reference = value[s.RVAL]

    def is_reference(self) -> Union[bool, int]:
        """Is this a reference?"""
        if self.reference > -1:
            return self.reference
        return False

    def to_reference(self) -> dict:
        """Return this back as a relational dict"""
        return {s.RVAL: self.reference}

    def __str__(self) -> str:
        """Return this back as a json string"""
        return json.dumps(self.to_reference())


class EditableEntry:
    """Class to hold data for an editable JSON entry"""

    def __init__(self, value, frame: ttk.Frame):
        self.inner_var = StringVar(value=str(value or ""))
        self.widget: Union[ttk.Frame, Entry, Text] = Entry(frame, textvariable=self.inner_var)

    def validate(self) -> bool:
        """Check that our inner_var is correct type"""
        return True

    def real(self) -> str:
        """Return the real string"""
        return str(self.inner_var.get())

    def __str__(self) -> str:
        return str(self.real())


class EditableStrEntry(EditableEntry):
    """Class to hold strings -- this is the default EditableEntry"""

    def __init__(self, value: str, frame: ttk.Frame):
        super().__init__(value, frame)


class EditableBoolEntry(EditableEntry):
    """Class to hold bools"""

    def __init__(self, value: bool, frame: ttk.Frame):
        super().__init__(value, frame)
        self.widget = ttk.Frame(frame)
        edit_bool_true = Radiobutton(self.widget, text="True", variable=self.inner_var, value="True")
        edit_bool_false = Radiobutton(self.widget, text="False", variable=self.inner_var, value="False")
        edit_bool_true.pack(anchor="w")
        edit_bool_false.pack(anchor="w")

    def real(self) -> bool:
        """Return the real bool"""
        if self.inner_var.get() == "True":
            return True
        return False


class EditableIntEntry(EditableEntry):
    """Class to hold ints"""

    def __init__(self, value: int, frame: ttk.Frame):
        super().__init__(value, frame)
        self.widget = Entry(frame, textvariable=self.inner_var)

    def validate(self) -> bool:
        """Check that our inner_var is actually an int"""
        return self.inner_var.get().isnumeric()

    def real(self) -> int:
        """Return the real int"""
        return int(self.inner_var.get())


class EditableFloatEntry(EditableEntry):
    """Class to hold float"""

    def __init__(self, value: float, frame: ttk.Frame):
        super().__init__(value, frame)
        self.widget = Entry(frame, textvariable=self.inner_var)

    def validate(self) -> bool:
        """Check that our inner_var is actually a float"""
        try:
            float(self.inner_var.get())
        except ValueError:
            return False
        return True

    def real(self) -> float:
        """Return the real float"""
        return float(self.inner_var.get())


class EditableReferenceEntry(EditableIntEntry):
    """Class to hold RelationalReference. Expand IntEntry"""

    def __init__(self, value: RelationalReference, frame: ttk.Frame, root: "TISE"):
        super().__init__(value.reference, frame)
        self._rreference = value
        self.widget = ttk.Frame(frame)
        sub_entry = Entry(self.widget, textvariable=self.inner_var)
        sub_entry.grid(row=0, column=0, sticky="e")
        # Go To Ref Button
        button_gtr = Button(
            self.widget,
            text=s.UI_GOTOREF,
            command=lambda: root.go_to_ref(int(self.inner_var.get()), None),
        )
        button_gtr.grid(row=0, column=1, sticky="w")

    def real(self) -> RelationalReference:
        """Take whatever was set to inner_var (should be an int), put it in our old ref, return"""
        self._rreference.reference = int(self.inner_var.get())
        return self._rreference


class EditableDictEntry(EditableEntry):
    """Class to hold generic dicts"""

    def __init__(self, value: dict, frame: ttk.Frame):
        super().__init__(value, frame)
        self.inner_var = None
        # This may be a dict with just text: val entries:
        self._is_simple_dict = True
        for _, propval in value.items():
            if not isinstance(propval, (str, int, float, bool)):
                self._is_simple_dict = False
                break
        if self._is_simple_dict:
            self.widget = ttk.Frame(frame)

            self._dict_entries = {}
            self._dict_types = {}
            for i, (propkey, propval) in enumerate(value.items()):
                label = Label(self.widget, text=propkey, justify="right")
                label.grid(row=i, column=0)
                entry = Entry(self.widget)
                entry.insert(0, propval)
                entry.grid(row=i, column=1)
                self._dict_entries[propkey] = entry
                self._dict_types[propkey] = type(propval)

        else:
            # Good luck:
            self.widget = Text(frame)
            self.widget.insert("end", json.dumps(value, indent=2))

    def validate(self) -> bool:
        """just return true, trust we have something"""
        return True

    def real(self) -> dict:
        """Try to recreate this dict"""
        if self._is_simple_dict:
            result = {}
            for key, entry in self._dict_entries.items():
                value = entry.get()
                if self._dict_types[key] is int:
                    value = int(value)
                elif self._dict_types[key] is float:
                    value = float(value)
                elif self._dict_types[key] is bool:
                    value = value.lower() in ("true", "yes", "1")
                elif self._dict_types[key] is None:
                    value = None
                result[key] = value
            return result
        else:
            assert isinstance(self.widget, Text)
            dictstr = self.widget.get("1.0", "end")
            dictreal = json.loads(dictstr)
            assert isinstance(dictreal, dict)
            return dictreal


class EditableDictCalcEntry(EditableEntry):
    """Class to hold dict[float] that have calculations to 1 (publicOpinion)"""

    def __init__(self, value: dict, frame: ttk.Frame):
        super().__init__(value, frame)
        self.inner_var = None
        self.widget = ttk.Frame(frame)
        self._dict_entries = {}
        i = 0
        total = 0.0
        for i, (propkey, propval) in enumerate(value.items()):
            label = Label(self.widget, text=propkey, justify="right")
            label.grid(row=i, column=0)
            total += propval
            if propkey == s.PUBLICOPINION_VAR:
                self._varvar = propkey
                label2 = Label(self.widget, text=propval, justify="left")
                label2.grid(row=i, column=1)
                continue
            entry = Entry(self.widget)
            entry.insert(0, propval)
            entry.grid(row=i, column=1)
            self._dict_entries[propkey] = entry
        # Final calculation entry
        label = Label(self.widget, text=s.UI_TOTAL, justify="right")
        label2 = Label(self.widget, text=total, justify="left")
        label.grid(row=i + 1, column=0)
        label2.grid(row=i + 1, column=1)

    def validate(self) -> bool:
        """just return true, trust we have something"""
        return True

    def real(self) -> dict:
        """Try to recreate this dict"""
        result = {}
        rem = 1.0
        for key, entry in self._dict_entries.items():
            value = float(entry.get())
            rem -= value
            result[key] = value
        result[self._varvar] = rem
        return result


class EditableListEntry(EditableEntry):
    """Class to hold a list of EditableEntries"""

    def __init__(self, value, frame: ttk.Frame):
        super().__init__(value, frame)
        # Good luck:
        self.inner_var = None
        self.widget = Text(frame)
        self.widget.insert("end", json.dumps(value, indent=2))

        # if isinstance

        # for item in value:

        # self.widget = ttk.Frame(frame)
        # Organize this with an inner listbox for all entries

        # Right side should be another details pane
        # templabel = Label(self.widget, text="TODO: EditableListEntry")
        # templabel.pack()

    def real(self) -> list:
        """Return the text inside the Text as a list"""
        assert isinstance(self.widget, Text)
        dictstr = self.widget.get("1.0", "end")
        dictreal = json.loads(dictstr)
        assert isinstance(dictreal, list)
        return dictreal


# pylint:disable=no-member
class TISE:
    """
    Terra Invicta Save Editor Main Application Class
    """

    def __init__(self, root: Tk):
        self.root = root
        self.root.geometry(f"{s.WINDOW_WIDTH}x{s.WINDOW_HEIGHT}")
        self.root.title(s.APP_NAME)
        self.default_directory = s.DEFAULT_SAVE_PATH.expanduser()
        self.json_data: Dict[str, Any] = {}
        self.statustext = ""
        self.current_id = 0
        self.ids: Dict[int, Tuple[str, int]] = {}  # Key of object ID. Value of (group string key, list index)
        self.groups: List[str] = []  # List of known group strings
        self.imgs: Dict[str, PhotoImage] = {}

        # Build Menu Bar
        self.menu_bar = Menu(self.root)
        self.menu_bar.add_command(
            label=s.UI_LOADJSON,
            command=self._load_json_file,
        )
        self.menu_bar.add_command(
            label=s.UI_SAVEJSON,
            command=self._save_json_file,
            state="disabled",
        )
        self.menu_bar.add_separator()
        self.menu_bar.add_command(
            label=s.UI_GOTOREF,
            command=self._show_gotoref_popup,
            state="disabled",
        )
        self.menu_bar.add_separator()
        self.menu_bar.add_command(
            label=s.UI_ABOUT,
            command=self._show_about_popup,
        )
        self.root.config(menu=self.menu_bar)

        # +-Application----------------------------_-#-X-+
        # | menu_bar                                     |
        # +-Grid:--------+--------------+----------------+
        # | LabelG       | LabelL       | LabelD         |
        # +--------------+--------------+----------------+
        # | FrameG       | FrameL       | PanedWindowD   |
        # | [ListBox|SB] | [ListBox|SB] |                |
        # +--------------+--------------+----------------+
        # | LabelF                                       |
        # +--------------+--------------+----------------+

        # Labels and Frames
        self.label_g = Label(self.root, text=s.UI_LABEL_G, font=Font(size=12), justify="center")
        self.label_l = Label(self.root, text=s.UI_LABEL_L, font=Font(size=12), justify="center")
        self.label_d = Label(self.root, text=s.UI_LABEL_D, font=Font(size=12), justify="center")
        self.frame_g = ttk.Frame(self.root, width=s.UI_LIST_WIDTH)
        self.frame_l = ttk.Frame(self.root, width=s.UI_LIST_WIDTH)
        self.panedwindow_d = PanedWindow(self.root, orient="vertical", height=s.WINDOW_HEIGHT - 80)
        # Details bar at frame_footer
        self.label_f = Label(
            self.root,
            text=s.UI_STATUS_READY,
            font=Font(size=8),
            justify="left",
        )

        # Grid Configuration
        self.label_g.grid(row=0, column=0, sticky="nsew", **s.UI_LABEL_PAD)
        self.label_l.grid(row=0, column=1, sticky="nsew", **s.UI_LABEL_PAD)
        self.label_d.grid(row=0, column=2, sticky="nsew", **s.UI_LABEL_PAD)
        self.frame_g.grid(row=1, column=0, sticky="nsew")
        self.frame_l.grid(row=1, column=1, sticky="nsew")
        self.panedwindow_d.grid(row=1, column=2, sticky="nsew")
        self.label_f.grid(row=2, column=0, columnspan=3, sticky="w", **s.UI_LABEL_PAD)
        self.root.grid_rowconfigure(1, weight=1)
        # self.root.grid_rowconfigure(2, weight=0)
        self.root.grid_columnconfigure(0, minsize=s.UI_LIST_WIDTH)
        self.root.grid_columnconfigure(1, minsize=s.UI_LIST_WIDTH)
        self.root.grid_columnconfigure(2, weight=1)

        # Inner FrameG for group list
        self.listbox_g = Listbox(self.frame_g, exportselection=0)
        self.listbox_g.bind("<<ListboxSelect>>", lambda _: self._select_group())
        self.listbox_g.pack(side="left", fill="both", expand=1)
        self.scrollbar_g = ttk.Scrollbar(self.frame_g)
        self.scrollbar_g.pack(side="right", fill="y")
        self.listbox_g.config(yscrollcommand=self.scrollbar_g.set)
        self.scrollbar_g.config(command=self.listbox_g.yview)

        # Inner FrameL for list of items in group
        self.listbox_l = Listbox(self.frame_l, exportselection=0)
        self.listbox_l.bind("<<ListboxSelect>>", lambda _: self._select_listitem())
        self.listbox_l.pack(side="left", fill="both", expand=1)
        self.scrollbar_l = ttk.Scrollbar(self.frame_l)
        self.scrollbar_l.pack(side="right", fill="y")
        self.listbox_l.config(yscrollcommand=self.scrollbar_l.set)
        self.scrollbar_l.config(command=self.listbox_l.yview)

        # Temp item within panedwindow_d:
        label_pwd = Label(self.panedwindow_d, text=s.UI_SELECT_ITEM)
        self.panedwindow_d.add(label_pwd)

        logging.info("TISE Startup Completed")

    def _refresh_status_bar(self, extratext: Optional[str] = None):
        """From text in `statustext`, refresh `label_f` text"""
        text = [s.UI_STATUS_READY]
        if self.current_id:
            text = [f"Current ID: {self.current_id}"]
        if self.groups:
            text.append(f"Groups: {len(self.groups)}")
        if self.ids:
            text.append(f"Unique objects: {len(self.ids)}")
        if extratext:
            text.append(extratext)
        self.label_f.config(text=", ".join(text))

    def _show_about_popup(self):
        """Open a new window showing the about info"""
        logging.info("Showing about popup")
        about_window = Toplevel(self.root)
        about_text = f"About {s.APP_NAME}"
        about_window.title(about_text)
        about_window.resizable(False, False)

        title = Label(
            about_window,
            text=about_text,
            font=Font(size=18),
        )
        title.grid(row=0, column=0, columnspan=2, sticky="sew", padx=20, pady=20)

        msg = Message(
            about_window,
            text="\n\n".join(s.APP_NOTICE),
            width=s.ABOUT_WIDTH,
            justify="center",
        )
        msg.grid(row=1, column=0, columnspan=2, sticky="nsew")

        img = self.imgs.get("github")
        if not img:
            assert s.GITHUB_LOGO.is_file()
            img = PhotoImage(file=str(s.GITHUB_LOGO))
            img = img.subsample(10, 10)
            self.imgs["github"] = img

        image_label = Label(about_window, image=img)
        image_label.grid(row=2, column=0, sticky="nse", pady=20)

        link = Label(
            about_window,
            text=s.APP_LINKTXT,
            fg="blue",
            cursor="hand2",
        )
        link.bind("<Button-1>", lambda e: webbrowser.open_new(s.APP_LINKURI))
        link.grid(row=2, column=1, sticky="nsw", pady=20)

        about_window.grid_rowconfigure(1, weight=1)
        about_window.grid_columnconfigure(0, weight=1)
        about_window.grid_columnconfigure(1, weight=1)

        about_window.focus_set()

    def _load_json_file(self):
        """
        Command function to load JSON file
        """
        file_path = filedialog.askopenfilename(
            initialdir=self.default_directory,
            title=s.UI_LOADJSON,
            filetypes=[
                (s.JSON_FILE, "*.json"),
            ],
        )
        logging.info("Loading JSON file: %s", file_path)
        if not file_path:
            logging.warning("Not loading empty file!")
            return
        with Path(file_path).open(mode="r", encoding=s.ENC) as fp:
            self.json_data = json.load(fp)
            assert isinstance(self.json_data, dict), f"Bad loading of JSON file, got type {type(self.json_data)}"
        # Clear previous keys and groups
        logging.info("Clearing old keys, groups")
        self.ids = {}
        self.groups = []
        self.current_id = 0
        # Reset UI
        logging.info("Clearing listbox and panedwindow_d content")
        self.listbox_g.delete(0, "end")
        self.listbox_l.delete(0, "end")
        for widget in self.panedwindow_d.winfo_children():
            widget.destroy()
        # Re-parse new data
        self._parse_json_data()
        # Enable the save button now
        self.menu_bar.entryconfig(s.UI_SAVEJSON, state="normal")
        self.menu_bar.entryconfig(s.UI_GOTOREF, state="normal")

    def _save_json_file(self):
        """
        Save the changes to a JSON file
        """
        file_path = filedialog.asksaveasfilename(
            initialdir=self.default_directory,
            title=s.UI_SAVEJSON,
            filetypes=[
                (s.JSON_FILE, "*.json"),
            ],
        )
        logging.info("Saving JSON file: %s", file_path)
        if not file_path:
            logging.warning("Not saving to blank file path!")
            return
        with Path(file_path).open(mode="w", encoding=s.ENC) as fp:
            fp.write(hackify_json(self.json_data))

    def _parse_json_data(self):
        """
        We have a new `json_data`!
        Reset our `keys` and `groups` values.
        Finally populate the items in listbox_g.
        """
        self.current_id = self.json_data[s.CURRENTID][s.RVAL]
        logging.info("PARSE: New JSON data: Current ID -> %d", self.current_id)
        self._refresh_status_bar("Refreshing data")
        # Refresh our group and ID registries
        for gidx, (group, listitems) in enumerate(self.json_data[s.GAMESTATES].items()):
            assert isinstance(group, str), f"group not str? {group}"
            if not listitems:
                logging.info("PARSE: Found group '%s', but it is EMPTY!", group)
            else:
                assert isinstance(
                    listitems, list
                ), f"listitems not list? {type(listitems)} -- group {group}, idx {gidx}"
                logging.info("PARSE: Found group '%s' with %d objects", group, len(listitems))
            assert group not in self.groups, f"Group {group} already in self.groups?"
            self.groups.append(group)
            if not listitems:
                continue
            for lidx, item in enumerate(listitems):
                assert isinstance(item, dict)
                iid = item[s.KEY][s.RVAL]
                assert isinstance(iid, int), f"PJ: item id not int? {iid} ({type(iid)})"
                values = item[s.VALUE]
                assert isinstance(values, dict), f"PJ: item values not dict? {type(values)}"
                # Add to ID registry
                assert iid not in self.ids, f"iid already in registry? {iid}"
                inner_group = values[s.GTYPE]
                assert inner_group == group, f"Mismatch: group {group} has item id {iid} with group {inner_group}?"
                self.ids[iid] = (group, lidx)
                logging.debug(
                    "PARSE: Found item ID '%d' for group '%s'(%d) with %d values", iid, group, gidx, len(values)
                )

        # Refresh our listbox_g now: Add items to listbox_g alphabetically
        for group in sorted([pretty_group(group) for group in self.groups]):
            self.listbox_g.insert("end", pretty_group(group))

        self._refresh_status_bar()

    def _select_group(self):
        """
        Populate listbox_l with list of items from selected group
        """
        # Clear listbox_l
        self.listbox_l.delete(0, "end")
        # Get our selection
        selidx = self.listbox_g.curselection()
        if not selidx:
            logging.warning("SELGROUP: No current selection in listbox_g!")
            return
        selidx = selidx[0]
        group = str(self.listbox_g.get(selidx))
        logging.info("SELGROUP: Selected listbox_g index %d: group '%s'", selidx, group)
        self._refresh_status_bar(f"Refreshing group '{group}'")
        # Get our group from gamestates. Check given name first, then try prepending namespace
        try:
            listitems = self.json_data[s.GAMESTATES][group]
        except KeyError:
            group = f"{s.COMMON_NAMESPACE}{group}"
            listitems = self.json_data[s.GAMESTATES][group]
        logging.info("SELGROUP: Found group with real name '%s' with %d objects", group, len(listitems))
        objmap = {}
        for item in listitems:
            assert isinstance(item, dict)
            iid = item[s.KEY][s.RVAL]
            assert isinstance(iid, int), f"SG: item id not int? {iid} ({type(iid)})"
            values = item[s.VALUE]
            assert isinstance(values, dict), f"SG: item values not dict? {type(values)}"
            display_name = str(values.get(s.DISPLAY_NAME, s.UNK))
            assert iid in self.ids, f"SG: iid {iid} ({display_name}) not in registry?"
            objmap[display_name] = iid

        # Refresh our listbox_l now: Add items to listbox_l alphabetically.
        # Use text of "iid:name" for `_select_listitem` below
        try:
            for item in sorted(objmap):
                self.listbox_l.insert("end", f"{objmap[item]:04}:{item}")
        except TypeError as err:
            logging.error(err)
            logging.error("Attempted sort of items in objmap: %s", str(objmap))

        self._refresh_status_bar()

    def _select_listitem(self):
        """
        Populate the right frame with the details of the selected item from listbox
        """
        # Clear the panedwindow
        for widget in self.panedwindow_d.winfo_children():
            widget.destroy()
        # Get our selection
        selidx = self.listbox_l.curselection()
        if not selidx:
            logging.warning("SELLIST: No current selection in listbox_l!")
            return
        selidx = selidx[0]
        iid, _, display_name = str(self.listbox_l.get(selidx)).partition(":")
        iid = int(iid)
        logging.info("SELLIST: Selected listbox_l index %d: Item ID %d: '%s'", selidx, iid, display_name)
        self._refresh_status_bar(f"Refreshing item '{display_name}' (ID {iid})")
        # Grab our item and verify we didn't mess up
        group, lidx = self.ids[iid]
        item = self.json_data[s.GAMESTATES][group][lidx]
        assert (
            iid == item[s.KEY][s.RVAL]
        ), f"SL: Mismatch iid {iid}, group '{group}', lidx {lidx}: Got iid {item[s.KEY][s.RVAL]}"
        values = item[s.VALUE]
        logging.info("SELLIST: Got item ID '%d' with %d values", iid, len(values))

        # Create a TreeView inside the panedwindow for all entries
        frame_entries = ttk.Frame(self.panedwindow_d, height=s.DELEM_HEIGHT)
        self.panedwindow_d.add(frame_entries)
        # frame_entries.grid(row=0, column=0, sticky="nsew", **s.UI_LABEL_PAD)
        # frame_entries.pack(side="top", fill="both", expand=True)
        treeview_entries_columns = (s.UI_KEY, s.UI_VAL, s.UI_REF)
        treeview_entries = ttk.Treeview(frame_entries, columns=treeview_entries_columns)
        treeview_entries.column("#0", width=0, stretch=False)
        treeview_entries.column(s.UI_KEY, anchor="e", width=s.UI_LIST_WIDTH)
        treeview_entries.column(s.UI_VAL, anchor="w", width=s.UI_LIST_WIDTH)
        treeview_entries.column(s.UI_REF, anchor="w", width=s.UI_LIST_WIDTH)
        for col in treeview_entries_columns:
            treeview_entries.heading(
                col,
                text=col,
                command=lambda _col=col: event_treeview_column_sort(treeview_entries, _col, False),
            )
        # treeview_entries.grid(row=0, column=0, sticky="nsew", **s.UI_LABEL_PAD)
        treeview_entries.pack(side="left", fill="both", expand=True)

        # Scrollbar
        scrollbar_entries = ttk.Scrollbar(frame_entries, command=treeview_entries.yview)
        scrollbar_entries.pack(side="right", fill="y")
        treeview_entries.config(yscrollcommand=scrollbar_entries.set)

        # Create details frame
        frame_details = ttk.Frame(self.panedwindow_d, height=s.DELEM_HEIGHT)
        self.panedwindow_d.add(frame_details)
        # frame_entries.grid(row=1, column=0, sticky="nsew", **s.UI_LABEL_PAD)
        # frame_details.pack(side="bottom", fill="both", expand=True)
        label_temp_fd = Label(frame_details, text=s.UI_SELECT_INST)
        label_temp_fd.pack()

        treeview_entries.bind(
            "<ButtonRelease-1>",
            func=lambda event: self._event_entry_click(event, iid, treeview_entries, frame_details),
        )

        for propidx, (propkey, propval) in enumerate(values.items()):
            rref = None
            showval = propval
            showref = ""
            if isinstance(propval, dict):
                rref = RelationalReference(propval)
                if rref.is_reference():
                    showval = rref.reference
                    showref = self._get_reference_display(rref.reference)
                else:
                    showval = f"Nested Dict with {len(propval.keys())} items -- Click to See"

            elif isinstance(propval, list):
                if propval:
                    if isinstance(propval[0], dict):
                        rref = RelationalReference(propval[0])
                        if rref.is_reference():
                            listids = []
                            listref = []
                            for rawref in propval:
                                iref = RelationalReference(rawref)
                                if not iref.is_reference():
                                    continue
                                listids.append(str(iref.reference))
                                listref.append(str(self._get_reference_display(iref.reference)))
                            showval = "[" + ", ".join(listids) + "]"
                            showref = "[" + ", ".join(listref) + "]"
                        else:
                            showval = f"Nested List[Dict] with {len(propval)} items -- Click to See"
                    elif isinstance(propval[0], list):
                        showval = f"Nested List[List] with {len(propval)} items -- Click to See"
                    else:
                        showval = "[" + ", ".join([str(i) for i in propval]) + "]"
                else:
                    showval = "[]"

            treeview_entries.insert("", "end", text=f"{propidx}", values=(propkey, showval, showref))

        self._refresh_status_bar()

    def _get_reference_display(self, rid: int) -> str:
        """
        Get the display value for the given reference value
        """
        assert isinstance(self.json_data, dict), "beep boop 456"
        logging.debug("GETREF: Finding %d...", rid)
        group, lidx = self.ids.get(rid) or (None, None)
        if group is not None and lidx is not None:
            logging.debug("        REF IN GROUP %s LIST INDEX %d...", group, lidx)
            item = self.json_data[s.GAMESTATES][group][lidx]
            assert isinstance(item, dict)
            iid = item[s.KEY][s.RVAL]
            assert isinstance(iid, int), f"REF key not int? {iid} ({type(iid)})"
            values = item[s.VALUE]
            assert isinstance(values, dict), f"REF values not dict? {type(values)}"
            if rid != iid:
                logging.error("GET_REF(%d) -> Got group '%s' and lidx %d -> Got Ref ID %d ?!?", rid, group, lidx, iid)
            display_name = values[s.DISPLAY_NAME]
            logging.debug("        REF %d SUCCESS! '%s' with {len(values)} values", iid, display_name)
            return display_name

        logging.error("GETREF: %d NOT IN ID REGISTRY!", rid)
        return s.UNK

    def _event_entry_click(self, event, iid: int, tktv: ttk.Treeview, frame: ttk.Frame):
        """When row in tree view is clicked:
        Display items details in `frame`
        """
        # Get the property key (column 0)
        rowid = tktv.identify_row(event.y)
        if not rowid:
            return
        propkey = str(tktv.set(rowid, s.UI_KEY))
        logging.info("TVEEC: User clicked on row %s, property %s, of object ID %d", rowid, propkey, iid)
        # Clear the frame
        for widget in frame.winfo_children():
            widget.destroy()
        # Get the property value
        group, lidx = self.ids[iid]
        values = self.json_data[s.GAMESTATES][group][lidx][s.VALUE]
        propval = values[propkey]
        # Populate details of property in frame
        label_head = Label(frame, text=f'{s.UI_KEY}: "{propkey}"')
        # label_head.pack(side="top")
        label_head.grid(row=0, column=0, sticky="nsew", **s.UI_LABEL_PAD)
        label_type = Label(frame, text=f"{s.UI_TYPE}: {type(propval)}")
        label_type.grid(row=0, column=1, sticky="nsew", **s.UI_LABEL_PAD)

        # Make a default EditableEntry from our propval
        edit_var: Union[EditableEntry, List[EditableEntry]] = EditableEntry(propval, frame)

        # Let's narrow this down to what it exactly is, however:
        if isinstance(propval, bool):
            # JSON True or False type
            edit_var = EditableBoolEntry(propval, frame)

        elif isinstance(propval, int):
            # JSON int type
            edit_var = EditableIntEntry(propval, frame)

        elif isinstance(propval, float):
            # JSON float type
            edit_var = EditableFloatEntry(propval, frame)

        elif isinstance(propval, str):
            # JSON string type
            edit_var = EditableStrEntry(propval, frame)

        elif isinstance(propval, dict):
            # If it is a relational dict, show that.
            rref = RelationalReference(propval)
            if rref.is_reference():
                label_type.config(text=f"{s.UI_TYPE}: REFERENCE")
                edit_var = EditableReferenceEntry(rref, frame, self)

            elif propkey == s.PUBLICOPINION:
                # Special editor for publicOpinion:
                label_type.config(text=f"{s.UI_TYPE}: Calculated Dict")
                edit_var = EditableDictCalcEntry(propval, frame)

            else:
                # Nope, just a regular dict
                # Put it in pretty-json-format in editable Text:
                edit_var = EditableDictEntry(propval, frame)

        elif isinstance(propval, list):
            # TODO: Handle list of refs

            edit_var = EditableListEntry(propval, frame)
            if not propval:
                label_type.config(text=f"{s.UI_TYPE}: list[] - empty!")
            else:
                if isinstance(propval[0], dict):
                    # Could be a list of references
                    rref = RelationalReference(propval[0])
                    if rref.is_reference():
                        label_type.config(text=f"{s.UI_TYPE}: list[REFERENCE]")

                    else:
                        # Just a list of dicts.
                        label_type.config(text=f"{s.UI_TYPE}: list[dict]")
                else:
                    # Just a regular list of whatever
                    label_type.config(text=f"{s.UI_TYPE}: list[{type(propval[0])}]")

        else:
            logging.warning("TVEEC: Unknown property value type: %s", str(type(propval)))

        edit_var.widget.grid(row=1, column=0, columnspan=4, sticky="nsew", **s.UI_LABEL_PAD)

        # Save Buttons
        button_save_val = Button(
            frame,
            text=s.UI_SAVEITEM,
            command=lambda: self._entry_save(edit_var, iid, propkey),
        )
        button_save_val.grid(row=0, column=2, sticky="nsew", **s.UI_LABEL_PAD)

        button_save_null = Button(
            frame,
            text=s.UI_SAVENULL,
            command=lambda: self._entry_save(None, iid, propkey),
        )
        button_save_null.grid(row=0, column=3, sticky="nsew", **s.UI_LABEL_PAD)

        frame.grid_rowconfigure(0, weight=0)
        frame.grid_rowconfigure(1, weight=1, minsize=s.DELEM_HEIGHT)
        frame.grid_columnconfigure(0, weight=1)

    def _entry_save(self, edit_var: Optional[EditableEntry], iid: int, propkey: str):
        """User clicked save button for a property value"""
        logging.info("EES: Got iid %d, propval '%s' -> Setting to var '%s'", iid, propkey, str(edit_var))
        if edit_var:
            real_var = edit_var.real()
        else:
            real_var = None
        logging.debug("EES: Our set var actually is: '%s' (%s)", str(real_var), str(type(real_var)))
        group, lidx = self.ids[iid]
        chkid = self.json_data[s.GAMESTATES][group][lidx][s.KEY][s.RVAL]
        assert iid == chkid, f"SL: Mismatch iid {iid}, group '{group}', lidx {lidx}: Got iid {chkid}"
        oldval = self.json_data[s.GAMESTATES][group][lidx][s.VALUE][propkey]
        logging.info(
            "EES: Got item ID '%d': Property '%s' has old value '%s' (%s)",
            iid,
            propkey,
            str(oldval),
            str(type(oldval)),
        )
        if type(oldval) is not type(real_var):
            logging.warning(
                "EES: IID %d, Property '%s': Old value is not same type and new value (%s != %s)",
                iid,
                propkey,
                type(oldval),
                type(real_var),
            )
        self.json_data[s.GAMESTATES][group][lidx][s.VALUE][propkey] = real_var
        newval = self.json_data[s.GAMESTATES][group][lidx][s.VALUE][propkey]
        logging.info(
            "EES: Got item ID '%d': Property '%s' has new value '%s' (%s)",
            iid,
            propkey,
            str(newval),
            str(type(newval)),
        )
        # Refresh the details pane
        self._select_listitem()

    def go_to_ref(self, iid: int, executing_window: Optional[Toplevel] = None):
        """Navigate the user to the given IID. If given a window, destroy that window"""
        logging.info("GTR: Attempting to go to item %d. Window? %s", iid, str(executing_window))
        if executing_window is not None:
            executing_window.destroy()
            self.root.focus_set()
        group, _ = self.ids[iid]
        # Get group's index
        try:
            gidx = list(self.listbox_g.get(0, "end")).index(group)
        except ValueError:
            # Attempt to get full namespace group name:
            gidx = list(self.listbox_g.get(0, "end")).index(pretty_group(group))
        self.listbox_g.selection_clear("0", "end")
        self.listbox_g.selection_set(gidx)
        self._select_group()
        # Get item's index, within the list of listbox_l
        self.listbox_l.selection_clear("0", "end")
        real_lbidx = -1
        for lbidx, lbname in enumerate(self.listbox_l.get(0, "end")):
            lid, _, _ = str(lbname).partition(":")
            if int(lid) == iid:
                real_lbidx = lbidx
                break
        if real_lbidx < 0:
            logging.error("GTR: Could not find iid %d within listbox of group '%s'", iid, group)
        else:
            self.listbox_l.selection_set(real_lbidx)
            self._select_listitem()

    def _show_gotoref_popup(self):
        """Show navigation popup"""
        logging.info("Showing gotoref popup")
        gotoref_window = Toplevel(self.root)
        gotoref_window.title(s.UI_GOTOREF)
        # gotoref_window.geometry(f"{s.SZW_POPUP_W}x{s.SZW_POPUP_H}")
        gotoref_window.resizable(False, False)
        entry_val = StringVar(gotoref_window)
        entry = Entry(gotoref_window, textvariable=entry_val)
        entry.grid(row=0, column=0)
        button = Button(
            gotoref_window,
            text=s.UI_GOTOREF,
            command=lambda: self.go_to_ref(int(entry_val.get()), gotoref_window),
        )
        entry.bind("<Return>", lambda _: self.go_to_ref(int(entry_val.get()), gotoref_window))
        button.grid(row=0, column=1)
        entry.focus_set()


# pylint:disable=unused-variable


def app():
    """
    Main application function to be called by Poetry
    """
    parser = argparse.ArgumentParser(description="TISE CLI")
    parser.add_argument("-v", action="count", default=0, help="Set the level of verbosity")
    args = parser.parse_args()
    verbosity = max(logging.WARNING - (int(args.v) * 10), logging.DEBUG)
    if verbosity == logging.INFO:
        logging.basicConfig(level=logging.INFO, format=s.LOGGING_FMT)
    elif verbosity == logging.DEBUG:
        logging.basicConfig(level=logging.DEBUG, format=s.LOGGING_FMT)
    else:
        logging.basicConfig(level=logging.WARNING, format=s.LOGGING_FMT)
    logging.log(verbosity, "<-- Set verbosity level")
    logging.info(f"BASE_PATH = {s.BASE_PATH}")
    # Start app
    root = Tk()
    TISE(root)
    root.mainloop()
