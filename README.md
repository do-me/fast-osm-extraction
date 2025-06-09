# Fast OSM Extraction

Workflows for performant extraction of entities from OSM protobuf (.pbf) files with various tools like DuckDB spatial and Rust-based OSM pbf readers.

Aim: Extract all roads with ongoing construction fast.

## 🥇 1 DuckDB spatial: 4:12 minutes (incl. post processing) 👑

The whole idea revolves around DuckDB spatial's `st_readOSM()` function (I stumbled upon on [HN](https://news.ycombinator.com/item?id=40891644)) that can directly read a pbf file. And no, you do not need tons of RAM! I actually tested it with 16Gb max RAM usage vs 96Gb and it had no effect on the processing time; literally none (not even a few seconds or so). Maybe I did something wrong, but considering it works so beautifully, I won't investigate further. 

Have a look at the attached Jupyter Notebook I used for convenience. You might even squeeze some more seconds performance out of DuckDB if it's not wrapped in Python.

## 🥈 2 Osmium tool: 7 minutes (without post processing)

Osmium (C++ based) could preprocess the planet file so all consecutive processing becomes faster. However, only this reduction already took 7 mins and turned my Mac into a helicopter.
Considering that Osmium is inferior, I didn't even continue this workflow but instead looked a little closer at Rust-based OSM Pbd-Readers as alternative.

```shell
time osmium tags-filter \
    planet-250602.osm.pbf \
    w/highway,construction \
    -o construction-roads.osm.pbf \
    --overwrite
[======================================================================] 100%
osmium tags-filter planet-250602.osm.pbf w/highway,construction -o    2964.27s user 153.42s system 736% cpu 7:03.38 total
```
