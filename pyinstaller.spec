block_cipher = None

open("_pyinstaller.py", "w").write("""
import tise

if __name__ == "__main__":
    tise.app()
""")

a = Analysis(['_pyinstaller.py'],
             pathex=['.'],
             binaries=[],
             datas=[('img/github-mark.png','.')],
             hiddenimports=[],
             hookspath=[],
             runtime_hooks=[],
             excludes=[],
             win_no_prefer_redirects=False,
             win_private_assemblies=False,
             cipher=block_cipher,
             noarchive=False)
pyz = PYZ(a.pure, a.zipped_data,
             cipher=block_cipher)
exe = EXE(pyz,
          a.scripts,
          a.binaries,
          a.zipfiles,
          a.datas,
          [],
          name='tise',
          debug=False,
          bootloader_ignore_signals=False,
          strip=False,
          upx=True,
          upx_exclude=[],
          runtime_tmpdir=None,
          console=False )

os.remove("_pyinstaller.py")

