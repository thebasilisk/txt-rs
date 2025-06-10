# Introduciton
This project is an attempt at making a macOS native markdown editor with internal links and other useful productivity features. It uses Rust bindings to Freetype for font/glpyh loading and creating bitmap representations, and uses Metal to render the text.

Currently this is a very bare bones text editor that doesn't support the full range of characters. Future steps are to implement new backing data structures for text, probably ropes, and then to add some basic user interfaces. There's also much to do in terms of text shaping, I might end up using rustybuzz for that. Currently the kerning tables through freetype-rs don't seem to be working, and I haven't handled glyph scaling/LoD yet either for especially small or large text.

# Installation/Usage
On any macOS machine with rust installed, simply clone the repository and use **cargo run** to launch.

Press any keys in the window to type, upon hitting the close button in the window it will save the file inside the folder. If you do not wish to save your file, terminate the app from the terminal.
