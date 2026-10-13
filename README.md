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
