## Basic TMD Parser

I made it to parse Vib-Ribbon's font. To make it, I used a copy of `FILEFRMT.PDF` that actually had visible diagrams (the &copy;2000 one) and [this random website](https://wiki.xentax.com/index.php/Playstation_TMD) as a baseline.

**This literally only supports linemeshes right now.**

I'm gonna go back in and fix a few design flaws, but otherwise I'm *very* happy with how this went! [`nom`](https://github.com/geal/nom) is an incredible crate; that and Rust's strict type system made the parser incredibly easy to develop.

This outputs OBJ files (Cool!) and JHF files (What?). Some caveats:
* the OBJ files live in a negative Y-up world in the direction of positive Z, so select those values while importing into blender or whatevs.
* JHF (esoteric format designed for Hershey fonts) exports are also not perfect because I don't quite know what to do with the left/right hand values. I'm working on seein how Vib-Ribbon draws its text so I can get it to be more accurate.
* JHF exports also have some quality loss from smalling the vertices. Not much I can do.

I've saved a "pretty alright" export of the Vib-Ribbon JHF font to the root of this repository. Maybe I'll update it later with accurate metrics. I decided to export in JHF because [I had just written a tool to view them](https://github.com/TheV360/hershey_fonts) and I thought that'd be convenient lol.

## Other Cool TMD Parsers

* [tmd2obj](https://github.com/taedixon/tmd2obj) (Java) - This person wrote their converter to extract Bubsy 3D assets, and I wrote mine to extract Vib-Ribbon assets. I guess this proves that you should never underestimate a fan of a game, no matter how big/small the game!! Theirs doesn't support line meshes -- but it does convert models into a complete OBJ/MDL combo!
* [tmd & PSXViewer](https://github.com/roblouie/tmd) (JavaScript) - While the [in-browser model viewer](https://roblouie.com/psx-viewer/) couldn't directly load Vib-Ribbon models, they are (likely) supported in the library!
