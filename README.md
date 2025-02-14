# Quotation Align

A simple process to align audio to text.
It tolerates gaps or unrelated insertions in sequences of up to ~100 words by default, depending on the alignment window size in the code.

# Use

This is an example use case of producing audio to text alignment of the first 10 of Shakespeare's Sonnets for further use in the Quotation app https://github.com/sowcow/quotation.

There are two inputs in this case:
- sonnets_001-010_shakespeare.mp3 - audio of reading
- md.md - the corresponding text
- some scripts use ffmpeg, so it should be installed, unless Docker way is used

I used linux with docker, which is configured to use GPU.

Use these or altered commands (if VRAM or other constraints apply) to produce json with timings of words in the audio file of interest:

- `chmod 766 word_timing/`
- `docker run --gpus all -it -v ".:/app" -v whisper_cache:/.cache ghcr.io/jim60105/whisperx:large-v3-en -- --output_format json --align_model WAV2VEC2_ASR_LARGE_LV60K_960H --output_dir /app/word_timing /app/sonnets_001-010_shakespeare.mp3`
- `ls word_timing/sonnets_001-010_shakespeare.json`

Then either use docker or rust way to produce `alignment.json`.

> TODO:
>
> Docker way, more user friendly:
> - `mkdir log/`
> - `docker run ...` - TODO
> - probably just build it here and just store binary? (no serious dependencies it seems)

Rust way, more detailed and developer-friendly:
- `cd rust
- `cargo build --release`
- `cd ..`
- `./rust/target/release/quotation-align ./word_timing/sonnets_001-010_shakespeare.json md.md alignment.json`

There are by-products of generating `alignment.json`:
- it should generate `log/` dir as an intermediate state that it continues from in case it did not complete alignment; for full reruns it should be deleted first
- when it completes there is `path-of-alignment.json` generated in the root, which is a joined path through windows that are shown in `log/`

To preview the result of alignment run: `ruby ruby/check_alignment.rb`.
It should read `alignment.json` and produce a file `preview.md` that mirrors the previous input file `md.md`, but shows which words ended up having start and end times from audio associated with them by `|` character like this:

> # THE SONNETS
> 
> by William Shakespeare
> 
> ## I
> 
> |From| |fairest| |creatures| |we| |desire| |increase,|

For more developer-friendly insights, check/visualize what it produces in log/:
- `cd ruby`
- `bundle`
- `ruby render_log.rb`
- => this should create .png files in log/, where each pixel is a word, brighter pixels are close matches, yellow line is the alignment produced

Remaining steps are about specializing alignment output to be consumed by Quotation app, and generation of fully useful `resources/` dir.

- `ruby quotation/content.rb`
- `ruby quotation/toc.rb`
- `ruby quotation/cut.rb sonnets_001-010_shakespeare.mp3`

This produces `resources/`, the very final step is to produce the app (`.apk` file) and to install it.

- `touch jks.jks signed.apk`
- `export JKS_PASS=aoeuaoeu`
- `docker run -e JKS_PASS -v "$(pwd)/./resources:/app/resources" -v "$(pwd)/jks.jks:/app/jks.jks" -v "$(pwd)/signed.apk:/app/signed.apk" ghcr.io/sowcow/quotation-build:latest sh -c "rake reg_make"`
- `adb install signed.apk`

# License: [DAFUQPL](https://github.com/dafuqpl/dafuqpl)

## Beaver teeth place
