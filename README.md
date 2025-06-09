# Fast OSM Extraction

Workflows for performant extraction of entities from OSM protobuf (.pbf) files with various tools like DuckDB spatial and Rust-based OSM pbf readers.

Aim: Extract all roads with ongoing construction fast and export to geoparquet/pmtiles or other file formats.

## 🥇 1 DuckDB spatial: 4:12 minutes for planet.pbf (incl. post-processing) 👑

The whole idea revolves around DuckDB spatial's `st_readOSM()` function (I stumbled upon on [HN](https://news.ycombinator.com/item?id=40891644)) that can directly read a pbf file. And no, you do not need tons of RAM! I actually tested it with 16Gb max RAM usage vs 96Gb and it had no effect on the processing time; literally none (not even a few seconds or so). Maybe I did something wrong, but considering it works so beautifully, I won't investigate further. 

Have a look at the attached Jupyter Notebook I used for convenience. You might even squeeze some more seconds performance out of DuckDB if it's not wrapped in Python.

Extracting all entities from Germany took only **13.3 seconds**! 

## 🥈 2 Osmium tool: 7 minutes for planet.pbf (without post-processing)

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

### 🥉 Osmpbfreader-rs (without post-processing)

I had high expectations but was disappointed. Trying to wrestle with the compiler and ever-changing APIs in the Rust ecosystem with dependency issues really gave me headaches. Also, unfortunately the geo ecosystem on Rust is underdeveloped. E.g. GeoPolars is stale: 

> Update (August 2024): GeoPolars is [blocked](https://github.com/pola-rs/polars/issues/1830#issuecomment-2218102856) on Polars supporting [Arrow extension types](https://github.com/pola-rs/polars/issues/9112), which would allow GeoPolars to persist geometry type information and coordinate reference system (CRS) metadata. It's not feasible to create a `geopolars.GeoDataFrame` as a subclass of a `polars.DataFrame` (similar to how the `geopandas.GeoDataFrame` is a subclass of `pandas.DataFrame`) because polars explicitly [does not support subclassing of core data types](https://github.com/pola-rs/polars/issues/2846#issuecomment-1711799869). See https://github.com/geopolars/geopolars/pull/240.

I ended up writing a short script that only read the data to an array as all the downstream tasks like persisting to geoparquet turned out to be too time-consuming for now. The speed was ok-ish: 

####  Germany only 48 seconds

```bash
time ./target/release/osm-construction-extractor --input ../germany-latest.osm.pbf
```

```bash
(base) ➜  osm-construction-extractor git:(master) ✗ time ./target/release/osm-construction-extractor --input ../germany-latest.osm.pbf
-> Opening PBF file: "../germany-latest.osm.pbf"
-> Pass 1: Finding ways and collecting dependencies...
   Found 60124 total objects (ways and their required nodes) in 48.54s.
-> Pass 2: Re-structuring extracted data into final format...
  [00:00:00] [########################################]   10049/10049   (0s)                                                                                                                                                                                                                                       
--- BENCHMARK RESULTS ---
Total ways extracted: 10049
Core extraction (PBF read & dependency resolution): 48.54s
Data restructuring (geometry building, etc.):       14.87ms
----------------------------------------------------
Total runtime:                                      48.55s

✅ Success! Data is held in an in-memory array.

Example of first extracted way:
ConstructionWay {
    id: WayId(
        3358460,
    ),
    tags: {
        "highway": "construction",
        "bicycle": "yes",
        "name": "Paul-Stritter-Weg",
        "surface": "paving_stones",
        "construction": "footway",
        "check_date": "2025-05-29",
        "lit": "yes",
    },
    geometry: LINESTRING(10.0243135 53.6102686,10.0243418 53.6102772,10.0249313 53.610489099999995),
}
./target/release/osm-construction-extractor --input ../germany-latest.osm.pbf  238.37s user 14.80s system 518% cpu 48.832 total
```

#### Planet


```bash
time ./target/release/osm-construction-extractor --input ../planet-250602.osm.pbf 
```

Aborted the run after 15 minutes. Not useful to measure the performance at this point.

### Other contenders

- [planetiler](https://github.com/b-r-u/osmpbf) - used it to create a protomaps/basemap once, and took roughly 2h, strong contender, also for convenience as it can export dircetly to mbtiles or pmtiles
- [osmpbf](https://github.com/b-r-u/osmpbf) - Rust-based too, haven't tried yet 
