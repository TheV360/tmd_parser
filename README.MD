## Basic TMD Parser

I made it to parse Vib-Ribbon's font. To make it, I used a copy of `FILEFRMT.PDF` that actually had visible diagrams (the &copy;2000 one) and [this random website](https://wiki.xentax.com/index.php/Playstation_TMD) as a baseline.

**This literally only supports linemeshes right now.**

I'm gonna go back in and fix a few design flaws, but otherwise I'm *very* happy with how this is going! [`nom`](https://github.com/geal/nom) is an incredible crate; that and Rust's strict type system made the parser incredibly easy to develop.

This outputs `OBJ` files (Cool!) and `JHF` files (What?). Some caveats:
- the `OBJ` files consist exclusively of line meshes. These will not have any geometry beyond just edges, so when importing into Blender ensure that "Lines" is checked in the Import OBJ dialog.
- the `OBJ` files live in a negative Y-up world in the direction of positive Z, so select those values while importing into Blender or whatevs.
- `JHF` (esoteric format designed for Hershey fonts) exports are also not perfect because I don't quite know what to do with the left/right hand values. I'm working on seein how Vib-Ribbon draws its text so I can get it to be more accurate.
- `JHF` exports also have some quality loss from shrinking the vertices. Not much I can do without making my own format.

I've saved a "pretty alright" export of the Vib-Ribbon JHF font to the root of this repository. Maybe I'll update it later with accurate metrics. I decided to export in `JHF` because [I had just written a tool to view them](https://github.com/TheV360/hershey_fonts) and I thought that'd be convenient lol.

## Usage

This isn't very intuitive right now, is it.

Recreate this folder structure:

```nothing
tmd_parser.exe
samples/
	WHATEVER.TMD
	YOU NEED.TMD
	EXTRACTED.TMD
out/
	fonts/
	models/
```

Then run the tool, and `JHF` fonts and `OBJ` models will appear in their respective directory. Okay, I definitely need to refactor this thing.

## Some Vib-Ribbon Notes

Seems [this video](https://www.youtube.com/watch?v=t2bxsgElWGg) was also made around the time of developing this tool! My method differed from theirs, though.

- I didn't use Magic ISO Maker to obtain the executable -- instead I used [Iso7z](https://www.tc4shell.com/en/7zip/iso7z/) which is an extension for 7-Zip that lets you extract a `CUE`/`BIN` combo into an `ISO`, which you can then extract into the executable.
- I didn't use `vibext` to extract my assets -- instead I used [QuickBMS](https://aluigi.altervista.org/quickbms.htm) with the Vib-Ribbon `PAK` extractor. (They're both on the same page -- use Ctrl+F to find the Vib-Ribbon extractor file) No idea who made that, but shoutouts to them.
- I... didn't use [`TMD2LWO`](https://www.zophar.net/utilities/psxutil/tmd2lwo.html) to convert the assets. It was a learning experience for me, to read through the file docs!
	- Actually, `LWO` files are a kind of pain to open in modern Blender so in reality I am completely justified and cool!

For people on Linux, there's likely not a Vib-Ribbon `PAK` extractor -- found one implemented in C# [over here](https://github.com/Resistiv/VibRibbonPakExtractor) (oh dang they did their own Reverse-Engineering work!). Plus, you probably have a way of extracting `CUE`/`BIN`s.

Anyway, now that you have the files:

- `FONT/01_FONT.TMD` is the Japanese font.
- `FONT/FE_FONT.TMD` is the English font.
- `FONT/MARK.TMD` is the ... [Score Coupons](https://vibribbon.fandom.com/wiki/Score_Coupons)? The symbols that rotate around in the top part of the screen during normal gameplay, the ones that indicate your score.

## Other Cool TMD Parsers

* [tmd2obj](https://github.com/taedixon/tmd2obj) (Java) - This person wrote their converter to extract Bubsy 3D assets, and I wrote mine to extract Vib-Ribbon assets. I guess this proves that you should never underestimate a fan of a game, no matter how big/small the game!! Theirs doesn't support line meshes -- but it does convert models into a complete OBJ/MDL combo!
* [tmd & PSXViewer](https://github.com/roblouie/tmd) (JavaScript) - While the [in-browser model viewer](https://roblouie.com/psx-viewer/) couldn't directly load Vib-Ribbon models, they are (likely) supported in the library!
