# 10 – Required Metadata: AppStream, Desktop File, Icons, and OARS

## Why Metadata Matters

Metadata is not cosmetic. It is parsed by GNOME Software, KDE Discover, and the Flathub website
to display the app's name, description, screenshots, age rating, and release history. Missing or
invalid metadata will cause build failures or rejection on Flathub.

---

## AppStream Metainfo (`*.metainfo.xml`)

### File Location in the Game Source Tree

```
flatpak/io.github.deavid.unhaunter.metainfo.xml
```

Installed to `/app/share/metainfo/io.github.deavid.unhaunter.metainfo.xml` by the manifest.

### Fully Annotated Template

```xml
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">

  <!-- The unique App ID. Must match the Flatpak app-id. -->
  <id>io.github.deavid.unhaunter</id>

  <!-- Human-visible name -->
  <name>Unhaunter</name>

  <!-- One-line summary (max ~80 chars, no period at end) -->
  <summary>A 2D isometric paranormal investigation game</summary>

  <!-- License for this metadata file (CC0 is expected by Flathub) -->
  <metadata_license>CC0-1.0</metadata_license>

  <!-- SPDX expression for the application's own license -->
  <project_license>MIT OR Apache-2.0</project_license>

  <!-- Developer info -->
  <developer id="io.github.deavid">
    <name>David Martínez Martí (deavid)</name>
  </developer>

  <!-- Multi-paragraph description. Use <p> and <ul>/<li>. No raw HTML. -->
  <description>
    <p>
      Unhaunter is a 2D isometric ghost-hunting game. Explore haunted locations,
      gather paranormal evidence with your instruments, identify the ghost type,
      and banish it before the hunting phase kills you.
    </p>
    <p>Features:</p>
    <ul>
      <li>Multiple ghost types with unique behaviours and evidence patterns</li>
      <li>Atmospheric lighting and thermal simulation</li>
      <li>Procedural sound propagation</li>
      <li>Local and online co-op (in development)</li>
    </ul>
  </description>

  <!-- Links -->
  <url type="homepage">https://deavid.github.io/unhaunter</url>
  <url type="bugtracker">https://github.com/deavid/unhaunter/issues</url>
  <url type="vcs-browser">https://github.com/deavid/unhaunter</url>

  <!-- The desktop file that launches the app -->
  <launchable type="desktop-id">io.github.deavid.unhaunter.desktop</launchable>

  <!-- Screenshots. Flathub recommends 1248x702 (16:9) or 624x351. -->
  <screenshots>
    <screenshot type="default">
      <caption>Investigating a haunted location</caption>
      <image type="source" width="1248" height="702">
        https://raw.githubusercontent.com/deavid/unhaunter/main/screenshots/gameplay-01.png
      </image>
    </screenshot>
  </screenshots>

  <!--
    OARS content ratings.
    Use the OARS generator at: https://hughsie.github.io/oars/
    Be accurate – Flathub audits these.
    Placeholder values below; adjust to the actual game content.
  -->
  <content_rating type="oars-1.1">
    <!-- Unhaunter involves ghost themes and mild scare content -->
    <content_attribute id="violence-cartoon">mild</content_attribute>
    <content_attribute id="violence-fantasy">mild</content_attribute>
    <!-- No sexual content, drugs, gambling, strong language (adjust if needed) -->
    <content_attribute id="social-chat">none</content_attribute>
    <content_attribute id="social-info">none</content_attribute>
  </content_rating>

  <!-- Release history. Add a new <release> for each version. -->
  <releases>
    <release version="0.X.X" date="YYYY-MM-DD">
      <url type="details">https://github.com/deavid/unhaunter/releases/tag/v0.X.X</url>
      <description>
        <p>Initial Flatpak release.</p>
      </description>
    </release>
  </releases>

  <!-- Categories (shown in software centres) -->
  <categories>
    <category>Game</category>
    <category>AdventureGame</category>
  </categories>

  <!-- Search keywords -->
  <keywords>
    <keyword>ghost</keyword>
    <keyword>horror</keyword>
    <keyword>paranormal</keyword>
    <keyword>investigation</keyword>
    <keyword>isometric</keyword>
    <keyword>bevy</keyword>
  </keywords>

</component>
```

### Validation

```bash
appstreamcli validate flatpak/io.github.deavid.unhaunter.metainfo.xml
```

This must produce zero errors before submitting. Warnings are acceptable but should be minimised.

---

## Desktop File (`*.desktop`)

```ini
[Desktop Entry]
Version=1.0
Type=Application
Name=Unhaunter
GenericName=Paranormal Investigation Game
Comment=Investigate haunted locations and banish ghosts
Exec=unhaunter
Icon=io.github.deavid.unhaunter
Categories=Game;AdventureGame;
Keywords=ghost;horror;paranormal;investigation;
StartupNotify=true
StartupWMClass=unhaunter_game
```

### Validation

```bash
desktop-file-validate flatpak/io.github.deavid.unhaunter.desktop
```

---

## Icons

Flathub requires at least one icon. Recommended set:

| Size | Format | Path in `/app/` |
|------|--------|-----------------|
| 64×64 | PNG | `/app/share/icons/hicolor/64x64/apps/io.github.deavid.unhaunter.png` |
| 128×128 | PNG | `/app/share/icons/hicolor/128x128/apps/io.github.deavid.unhaunter.png` |
| 256×256 | PNG | `/app/share/icons/hicolor/256x256/apps/io.github.deavid.unhaunter.png` |
| Scalable | SVG | `/app/share/icons/hicolor/scalable/apps/io.github.deavid.unhaunter.svg` |

The repo already has `favicon-512x512.png`, `favicon-32x32.png`, `favicon-16x16.png`. Add
`favicon-64x64.png`, `favicon-128x128.png`, and `favicon-256x256.png` (or derive them from
the 512 version with ImageMagick during asset preparation, before the Flatpak build).

**Requirements:**
- Icon must have a transparent background.
- Must not be a screenshot or contain text overlays.
- Should visually represent the app (the ghost / haunted house logo).

---

## OARS Age Rating

OARS (Open Age Ratings Service) is how Flatpak apps declare content suitability. Use the
interactive generator at **[https://hughsie.github.io/oars/](https://hughsie.github.io/oars/)**
to fill in values honestly. For Unhaunter:

| Category | Likely value | Rationale |
|----------|-------------|-----------|
| `violence-cartoon` | `mild` | Ghost threats, jump scares |
| `violence-fantasy` | `mild` | Banishing ghosts |
| `language` | `none` or `mild` | Check in-game text |
| `social-chat` | `none` | No in-game chat yet |
| `social-info` | `none` | No personal data collection |
| `money-purchasing` | `none` | FOSS, no IAP |

The generator outputs the XML fragment to paste into `metainfo.xml`.

---

## Screenshots

Flathub displays screenshots prominently on the app page. Requirements:

- At least one screenshot.
- Minimum 752×423 pixels; recommended 1248×702 (16:9).
- Should show actual gameplay, not the main menu alone.
- File format: PNG or JPEG.
- Can be hosted remotely (raw GitHub URL works) or bundled in `/app/share/`.

The game's `screenshots/` directory in the repo is a natural source. Pick the best 3–5
gameplay shots and reference them in `metainfo.xml`.
