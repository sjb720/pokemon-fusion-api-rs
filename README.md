# Pokemon fusion API! (Rust)

To get running, make sure you have rust installed. Then run `cargo run` and you're set!

## Routes

### Health check

- `/` Will return a string if the API is up and healthy.

### Fusions

Where `id1` is the main Pokemon and `id2` is the Pokemon to be fused onto the main Pokemon. (First is head second is body.)

- `/pokemon/fuse/:id1/:id2` Gets a JSON object with the fused Pokemon's information. Returns MISSING NO if it fails.
- `/pokemon/fuse/:id1/:id2/iamge` Gets a png of the fused Pokemon. Returns a 404 if no image is available.

# Features

- Pokemon caching for requested pokemon.
- Fusion logic built in! Just pass two IDs and get back the new Pokemon's.
- Static file serving for fusion images.

# Future Plans

This is just a fun project to help myself learn rust! Not sure what will come of it but here are some future plans:

- Calculate fusion abilities
- Save cached data locally and reload the state on start. Allow for cache clearning by deleting cached file. (Pokemon might change once every year and it's almost always by addition so likely they will never change.)

# Credits

I absolutely did not create *ANY* of the beautiful images this API serves. Credit is owed to the following:

- The Pokemon infinite fusion community. All sprites were created by hundreds of artists over the past few years. Their latest work is found [here](https://gitlab.com/pokemoninfinitefusion/customsprites "A repository where the latest sprites are uploaded.").
- The PokeAPI. All pokemon data is originally pulled from [here](https://pokeapi.co/) (and then cached of course!)