"""TISE: Terra Invicta Save Editor
Created by Jake Staehle <jacob@staehle.us>
"""
import sys
from pathlib import Path

# pylint:disable=unused-variable

# Application constants
APP_NAME = "TISE: Terra Invicta Save Editor"
APP_AUTHOR = "Jake Staehle <jacob@staehle.us>"
APP_LINKTXT = "staehle/tise"
APP_LINKURI = "https://github.com/staehle/tise"
APP_NOTICE = [
    APP_NAME,
    f"Created by {APP_AUTHOR}",
    "For more information, visit our GitHub page:",
]
try:
    # PyInstaller location for resources
    BASE_PATH = Path(sys._MEIPASS)
    GITHUB_LOGO = BASE_PATH.joinpath("github-mark.png")
except Exception:
    # Local build
    BASE_PATH = Path(__file__).parent.parent
    GITHUB_LOGO = BASE_PATH.joinpath("img").joinpath("github-mark.png")

LOGGING_FMT = "%(asctime)s %(levelname)8s: %(message)s"

WINDOW_WIDTH = 1280
WINDOW_HEIGHT = 720
ABOUT_WIDTH = 420

DELEM_HEIGHT = 400

DEFAULT_SAVE_PATH = Path("~/Documents/My Games/TerraInvicta/Saves")

ENC = "utf-8"

UI_LOADJSON = "Load JSON"
UI_SAVEJSON = "Save JSON"
UI_SAVEITEM = "Save Property"
UI_SAVENULL = "Save Property as 'null'"
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
JSON_FILE = "JSON Files"
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
