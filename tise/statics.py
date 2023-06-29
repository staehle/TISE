"""TISE: Terra Invicta Save Editor
Created by Jake Staehle <jacob@staehle.us>
"""
import sys
from pathlib import Path

# pylint:disable=unused-variable

# Application constants
APP_NAME = "TISE: Terra Invicta Save Editor"
APP_VERSION = "1.1.0"
APP_AUTHOR = "Jake Staehle <jacob@staehle.us>"
APP_LINKTXT = "staehle/tise"
APP_LINKURI = "https://github.com/staehle/tise"
APP_NOTICE = [
    f"Version {APP_VERSION}",
    f"Created by {APP_AUTHOR}",
    "For more information, visit our GitHub page:",
]

if getattr(sys, "frozen", False):
    # PyInstaller
    # pylint:disable=protected-access
    EXEC_PATH = Path(sys.executable).parent
    BASE_PATH = Path(sys._MEIPASS)
    APP_ENV = "-PyInstaller"
else:
    # Normal Python
    EXEC_PATH = Path(__file__).parent.parent
    BASE_PATH = EXEC_PATH
    APP_ENV = ""

GITHUB_LOGO = BASE_PATH.joinpath("img").joinpath("github-mark.png")

LOGGING_FMT = "%(process)d %(asctime)s %(levelname)8s: %(message)s"
LOGGING_OPTS_S = {
    "format": LOGGING_FMT,
}
LOGGING_OPTS_D = {
    "filename": str(EXEC_PATH.joinpath("tise.log")),
    "encoding": "utf-8",
}

WINDOW_WIDTH = 1280
WINDOW_HEIGHT = 720
ABOUT_WIDTH = 420

DELEM_HEIGHT = 400

DEFAULT_SAVE_PATH = Path("~/Documents/My Games/TerraInvicta/Saves")

ENC = "utf-8"

UI_SAVEFILE = "Save Game"
OD_SAVEEXTS = "*.json *.gz"
UI_LOADJSON = "Load Game"
UI_SAVEJSON = "Save Game"
UI_SAVEITEM = "Save Property"
UI_SAVENULL = "Save Property as 'null' (Erase)"
UI_GOTOREF = "Go to Ref"
UI_ABOUT = "About"
UI_LABEL_G = "Groups"
UI_LABEL_L = "List"
UI_LABEL_D = "Details"

UI_KEY = "Property"
UI_VAL = "Value"
UI_REF = "Reference"
UI_TYPE = "Property Type"

UI_TOTAL = "Total ="

UI_SELECT_ITEM = """
Select a Group, then select an Item from the List!
See and edit details here!
"""

UI_SELECT_INST = """
Select a Property above!
See and edit details here!
"""

UI_LABEL_PAD = {"padx": 5, "pady": 5}
UI_LIST_WIDTH = 250

UI_STATUS_READY = "Ready"

# JSON file related constants

GAMESTATES = "gamestates"
CURRENTID = "currentID"
COMMON_NAMESPACE = "PavonisInteractive.TerraInvicta."

KEY = "Key"
VALUE = "Value"
RVAL = "value"  # relational value is lowercase
RTYPE = "$type"
DISPLAY_NAME = "displayName"
ID = "id"
GTYPE = "$type"

PUBLICOPINION = "publicOpinion"
PUBLICOPINION_VAR = "Undecided"

UNK = "<?~?~?>"
