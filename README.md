# ishtaria-worldgen

<img src="https://git.vitexsoftware.com/Ishtaria/ishtaria-client/raw/branch/main/assets/branding/emblem.png" alt="Ishtaria" width="96">

Deterministic procedural generator of an Earth-sized planet. Terrain is a pure function of `(seed, position)` on a cube-sphere; worlds store only deltas.

```sh
ishtaria-worldgen 42 256 > planet.pgm     # six cube faces as a height-map strip
```

The generator combines continental shelves, lowland hills and ridged mountain
regions. Noise is sampled in three-dimensional sphere coordinates, so neighboring
cube faces meet without seams. Default elevation amplitude is 8000 metres;
the PGM encodes `[-8000, 8000]` as `[0, 255]`, with sea level at 127.5.
Servers derive lakes, downhill drainage, river catchments and climate biomes from
the persisted map; clients render water, snow and biome-selected scenery.

## Real planets (datadisks)

A story datadisk may define the world's real planet: the real continent outlines,
heights and water of the Earth as it was when the story takes place (maximum
historical fidelity — see ADR 0017 and the datadisk schema's `world.terrain`).
The disk ships two equirectangular P5 PGMs of the same size (`width == 2 * height`,
row 0 at +90 degrees, column 0 at -180 degrees): a 16-bit heightmap (maxval 65535,
big-endian, pixel = elevation in metres as a signed value, 0 is sea level) and an
8-bit water mask (0 land, 255 water). `--earth` renders the strip from them instead
of the noise:

```sh
ishtaria-server --print-world-terrain gilgamesh    # prints the disk's two rasters
ishtaria-worldgen --earth HEIGHT.pgm --water MASK.pgm 512 > planet.pgm
```

The sphere is mapped to the planet's real geography as `latitude = asin(y)` and
`longitude = atan2(z, x)` (the +Y face is the north pole), so the server's compass
and historical bearings are the real ones. The water mask decides the coastline:
water pixels stay at or below sea level (so the shallow Persian Gulf stays sea),
land pixels stay above it (so the dry Qattara depression stays land). The seed
plays no part in this mode; the server still stores it for the seeded local relief
and world identity.

This terrain algorithm changes newly generated maps, including maps generated
with an existing seed. Do not regenerate over an imported world's source file or
overwrite its database. Existing imports remain intact and conflicting imports
are rejected. An algorithm upgrade for a populated world requires a separate
version/migration decision. The finite PGM is a preview, not metre-resolution
Earth terrain or hydraulic/erosion simulation.

## Installation

Debian / Ubuntu (x86-64) only:

```sh
echo "deb http://repo.vitexsoftware.com $(lsb_release -sc) main" | sudo tee /etc/apt/sources.list.d/vitexsoftware.list
sudo wget -O /etc/apt/trusted.gpg.d/vitexsoftware.gpg http://repo.vitexsoftware.com/keyring.gpg
sudo apt update
sudo apt install ishtaria-worldgen
```

License: MIT

## Part of Ishtaria

Ishtaria is an open-source, persistent, federated virtual planet of Earth size.
Documentation: https://vitexsoftware.github.io/ishtaria-docs/ · All repositories: https://git.vitexsoftware.com/Ishtaria
