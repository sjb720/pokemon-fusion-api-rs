use axum::{
    routing::get,
    Router, extract::Path,
};
use std::collections::HashMap;
use serde_json::Value;


#[tokio::main]
async fn main() {
    let mut collected_pokemon: HashMap<u16, Pokemon> = HashMap::new();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(ping))
        .route("/pokemon/name/:id", get(name_handler))
        .route("/pokemon/fuse/:id1/:id2", get(fuse_handler));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn ping() -> &'static str {
    "Welcome to the pokemon fusion API!"
}

struct Stats {
    hp: u16,
    attack: u16,
    defense: u16,
    special_attack: u16,
    special_defense: u16,
    speed: u16,
}

struct Pokemon {
    name: String,
    stats: Stats,
    types: (String, String)
}

async fn fuse_handler(Path((id1, id2)): Path<(u16, u16)>) -> String {

    let p1 : Pokemon;
    let p2 : Pokemon;

    let pokemon_1_r = get_pokemon_data(id1).await;
    match pokemon_1_r {
        Err(_err) => return "Failed to grab pokemon 1".to_string(),
        Ok(pkmn) => {p1 = pkmn},
    }

    let pokemon_2_r = get_pokemon_data(id2).await;
    match pokemon_2_r {
        Err(_err) => return "Failed to grab pokemon 2".to_string(),
        Ok(pkmn) => {p2 = pkmn},
    }

    format!("Fusing {} with {}", p1.name, p2.name )
}

async fn name_handler(Path(id): Path<u16>) -> String {
    let resp = get_pokemon_data(id).await;

    match resp {
        Err(err) => "Failed to grab".to_string(),
        Ok(msg) => msg.name
    }
}

async fn get_pokemon_data(id: u16) -> Result<Pokemon,reqwest::Error>  {
    
    let url = format!("https://pokeapi.co/api/v2/pokemon/{}", id);
    let resq: serde_json::Value = reqwest::get(url).await?
    .json()
    .await?;

    Ok(pokemon_api_to_struct(&resq))
}

fn pokemon_api_to_struct(data : &Value) -> Pokemon {
    Pokemon { name: data["name"].as_str().unwrap_or_default().to_string(), stats: Stats {
        attack: 0,
        defense: 0,
        hp: 0,
        special_attack: 0,
        special_defense: 0,
        speed: 0,
    }, types: ("psychic".to_string(), "grass".to_string()) }
}