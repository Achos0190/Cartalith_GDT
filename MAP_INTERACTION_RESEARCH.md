# How other map and drawing apps handle the map under a finger (research, 2026-11-01)

**Agent-produced 2026-11-01**, from the owner's answer to the DS-13 panning questions: *"Research how other mapping
apps have their interface for map interactions."* (Ruling EB, item 1). The questions are DS13-Q1 and DS13-Q2 in
`MOBILE_SHELL_PROPOSAL.md` §2.5. Research, not a scope: it has no milestones and no status column.

Each claim names its source. Most were read from the vendor's own documentation, fetched on 2026-11-01. A claim marked
*(unconfirmed)* came from a search excerpt or a community thread only. Documentation describes intent; none of these
apps was run on a device here.

## 1. The questions

- **DS13-Q1.** Is ✋, the pan latch, drawn only while a drawing tool is armed? (Default: yes.)
- **DS13-Q2.** Does one finger pan with a tap-only tool? (Default: yes, and a drawing tool keeps the finger.)
- **Ruling EB item 3** added one-handed continuous zoom (double-tap, hold, slide) "Google Maps style". It must not
  collide with the two-finger-tap undo (Ruling DU Q5) or with a drawing tool's strokes, and that is settled with
  Q1's answer.

## 2. Navigation maps: one finger always pans

| App | One finger drag | Double-tap | Zoom out one-handed | Zoom buttons |
|---|---|---|---|---|
| Google Maps SDK, Android | pans | zoom in | two-finger tap; double-tap-hold, slide up | bottom right, **off by default** |
| Google Maps SDK, iOS | pans | zoom in | two-finger tap | not mentioned |
| Apple Maps, iPhone (iOS 14 and 15 guides) | pans | zoom in | double-tap-hold, slide **down** | none |
| Mapbox, Android and iOS v11 | pans | zoom in | two-finger double-tap (Android) or tap (iOS); quick zoom, slide up | none by default |
| OsmAnd | pans | zoom in | two-finger double-tap; double-tap-hold, slide up | **always shown**, by My Location |
| Organic Maps, CoMaps | pans | zoom in | double-tap-hold, slide up | a setting in Organic Maps (`pref_show_zoom_buttons`) |
| Gaia GPS, iOS and Android | pans | zoom in | two-finger tap; double-tap-hold, slide up | not mentioned |
| TomTom GO | pans | zoom in | zoom buttons ("pinch not available on all devices") | yes |

Sources: developers.google.com/maps/documentation/android-sdk/controls and …/ios-sdk/controls;
support.apple.com/guide/iphone/view-maps (iOS 14.0 and 15.0 editions); docs.mapbox.com/android/maps/guides/user-interaction/gestures
and docs.mapbox.com/ios/maps/guides/user-interaction/gestures; osmand.net/docs/user/map/interact-with-map;
organicmaps.app/news/2026-02-07 and the Organic Maps `strings.xml` / `prefs_main.xml`; comaps.app/support;
help.gaiagps.com (articles 360046448474 and 4409332168727); download.tomtom.com GO manual, "Using gestures".

**The slide direction of one-handed zoom is not universal.**
- **Slide down zooms in, up zooms out:** Google Maps SDK for Android, Android's own `ScaleGestureDetector` quick
  scale (AOSP source: moving down scales up), Mapbox Android and iOS v11, Organic Maps, CoMaps, OsmAnd and Gaia GPS
  (its iOS article too).
- **Slide up zooms in:** Apple Maps (both iPhone guides) and Mapbox's legacy iOS SDK 6.4.1.
- So "Google Maps style", the owner's words, means **down zooms in**. It is also the majority, and the platform
  default on Android, where the phone build ships.

**Timing, from AOSP `ViewConfiguration.java`:** double-tap timeout 300 ms, long-press timeout 400 ms, touch slop
8 dp, double-tap slop 100 dp. Android's `GestureDetector` reports a single tap only once it is sure no second tap
follows (`onSingleTapConfirmed`), so a tap that also has a double-tap meaning waits up to 300 ms.

## 3. Drawing on a map: five patterns

Navigation apps never have to share the finger. Apps that draw on a map do, and they solve it in five ways.

| Pattern | Apps | How the map moves while drawing |
|---|---|---|
| **A. One finger draws, two fingers pan** | ArcGIS Field Maps markup on iOS: *"you can pan the map using two fingers while using the markup tools"* | Two fingers; one-finger pan returns after **Done** |
| **B. The tool lets go after one mark** | Field Maps markup on Android (*"After drawing a single markup element, the type is automatically deselected so you can pan the map"*; double-tap the type to lock it); Footpath (*"return to 'move' mode"* after each trace); Strava (lifting the finger ends the sketch; tap the pencil to add more); Survey123 sketch (finishes after one stroke) | One finger, between marks |
| **C. Crosshair and an Add button** | QField (default), Mergin Maps, Google Earth measure, ArcGIS Earth drawings, Field Maps capture, Organic Maps point picking | One finger always pans; points are added at the screen's centre |
| **D. A pan toggle** | Survey123 (*"tap the Pan and zoom button"* to move the map instead of placing vertices); Tabletop RPG Map Editor II (a "Move map" button near the map) | One finger, while the toggle is on |
| **E. The pen draws, the finger pans** | Field Maps (a stylus locks the markup type; *"your fingers can still be used to pan, zoom, or interact with the map"*); QField freehand (activates on stylus hover) | One finger, whenever a pen is in use |

Sources: doc.arcgis.com/en/field-maps/android/use-maps/markup.htm and …/capture.htm; Esri's iOS markup help (mirror at
gisportalny.dot.ny.gov); doc.arcgis.com/en/survey123/capture/field-app/capturegeometry.htm;
docs.qfield.org/how-to/data-collection/digitize; merginmaps.com/docs/field/mobile-features;
support.google.com/earth/answer/9010337; doc.arcgis.com/en/arcgis-earth/mobile/create-drawings.htm;
footpathapp.com/user-guide/quickstart; support.strava.com article 18001474720397;
deepnight.itch.io/tabletop-rpg-map-editor (devlog 107906). Felt documents its tools but not how the map pans while
drawing *(unconfirmed)*. Inkarnate is browser-only and documents no touch controls; Google My Maps has had no mobile
app since 2021.

## 4. Drawing and painting apps

- **Procreate.** The finger paints. *"By default Procreate requires a two finger tap and drag motion to move the
  canvas."* A two-finger tap undoes and a three-finger tap redoes. Its accessibility **Single Touch Gestures
  Companion** adds a floating **Move** button that, when on, turns one finger into a pan: the same latch as ✋, offered
  *"for those with limited hand mobility"*. A **Right-hand interface** switch mirrors the sidebar.
  (help.procreate.com, Gestures and Preferences.)
- **Concepts.** Two fingers pan, or the **Pan tool** pans with one finger; one-finger pan can be enabled; with a
  stylus, **Finger Action** can be set to pan; a **handedness** setting sets the default layout. (concepts.app manual.)
- **Krita.** **Touch Painting**: Auto (the default; touch paints only when no stylus is present), on or off. When
  touch paints, three fingers pan; when it does not, one finger pans. A **Pan tool** and Space+drag also exist.
  (docs.krita.org, General settings and Pan tool.)
- **Infinite Painter.** The finger paints and two fingers navigate. With an active stylus it assigns *"the stylus
  … for painting while the fingers for screen navigations"*. (docs.infinitestudio.art.)
- **ibis Paint** offers four palm-rejection modes, from "both draw" to "stylus only". **Clip Studio Paint** answers
  *"my finger … makes unintended lines"* with "Draw with finger" off, and by default a single-finger swipe uses the
  Hand tool *(from the FAQ page's summary)*. **Adobe Fresco and Illustrator for iPad**: two-finger tap undo, three-finger tap redo *(unconfirmed:
  helpx.adobe.com refused the fetch; search excerpts only)*.
- **Platforms.** Apple PencilKit's `.pencilOnly` drawing policy and the system "Only Draw with Apple Pencil" setting;
  Windows Ink accepts pen input only by default.

**A failure mode worth copying the fix for.** Krita merge request !2680 (2026-03-07, bug 517034): after a pinch, a
user lifts one finger and moves the other, and touch painting draws *"random blotches"*. The proposed fix (the merge
request appeared open when read) blocks touch painting for the rest of the touch once more than one finger has been
down. Any app where one finger draws and two
fingers pan has this hazard.

## 5. Guidance

- **Apple HIG.** Gestures must be discoverable and *"Not the only way to perform an important action"*; *"Offer
  alternatives to gestures"*; *"avoid custom multifinger … gestures"*. Default control 44 × 44 pt, minimum 28 × 28.
  For a pencil: *"Design a great left- and right-handed experience"* and avoid controls either hand may cover.
- **Android.** Touch targets at least 48 × 48 dp, about 9 mm, separated by 8 dp.
- **WCAG 2.2.**
  - **2.5.1 Pointer Gestures:** a multipoint gesture needs a single-pointer alternative. Its own example is a map
    whose pinch zoom has plus and minus buttons.
  - **2.5.7 Dragging Movements:** a drag needs a non-drag alternative. Its example is a map with direction buttons
    beside drag-to-pan.
  - **2.5.8 Target Size (Minimum):** 24 × 24 CSS px.
- **Reach.** Hoober (2013, 1 333 observations): 49 % of phone use is one-handed, and 33 % of that is the **left**
  thumb; grips change every few seconds. Hoober (2017): corner targets need about 12 mm, centre ones about 7 mm.
- **Handedness settings** exist in TomTom (it moves the zoom bar), Procreate and Concepts. None was found in Google
  Maps, Apple Maps, OsmAnd or Organic Maps *(a negative result, unconfirmed)*.

## 6. What this means for Cartalith

The shipped phone build is pattern A with pattern D's toggle, for every tool: one finger goes to the tool, two
fingers pan, and ✋ turns one finger into a pan (`MOBILE_SHELL_PROPOSAL.md` §2.5, "What ships today"). The DS-13
design keeps that for drawing tools and gives tap-only tools navigation-app behaviour.

- **DS13-Q2, one finger pans with a tap-only tool.** Every navigation map does this (§2), and so do pattern C's field
  apps, which never let the finger draw at all. The research supports the default: **yes**.
- **DS13-Q1, ✋ only while a drawing tool is armed.** The latch has precedents: Survey123's Pan and zoom button,
  Procreate's Move button and the Tabletop RPG Map Editor's Move map button. Under WCAG 2.5.1 it is also needed: two-finger
  pan is a multipoint gesture, so a drawing tool needs a one-finger way to move the map, and ✋ (with the −|+ rocker)
  is it. With a tap-only tool, a drag already pans, so a latch would change nothing. The research supports the
  default: **yes**.
- **Two findings the defaults do not yet cover:**
  1. **Krita's guard.** Once two fingers have been down, a drawing tool should ignore the rest of that touch, so
     lifting one finger after a two-finger pan or pinch does not draw a stroke.
  2. **When thumb zoom works** (Ruling EB item 3). A double-tap with a drawing tool is two dabs, and with a tool
     whose tap places something (Settlement, Label, Icon, Conflict, Route waypoint, Measure point) it places before
     it can zoom. Android's own answer is to delay the tap 300 ms; the alternative is to keep double-tap zoom
     **off** while such a tool is armed, except while ✋ is on, when the finger is navigating anyway. The second
     keeps placement instant and gives ✋ one meaning: *the finger moves the map*.
- **The slide direction:** down zooms in, up zooms out (§2; Google Maps, Android's default and the majority).
- **Pattern B (the tool lets go after one mark)** suits one-shot markup. It does not suit sculpting or painting,
  which take many strokes, so it is not recommended for the drawing tools here.

## 7. Limits

- Documentation only; no app was run on a device, and documentation can lag an app.
- Felt's pan-while-drawing behaviour, Adobe's iPad gestures and the absence of handedness settings in the big map
  apps are unconfirmed.
- The raw notes and fetched pages were working files in the session's scratchpad and were not kept; the URLs above
  are the sources.
