use axum::{
    debug_handler,
    routing::get,
    Router, extract::{ Path, State},
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde_json::Value;


type SharedState = Arc<RwLock<AppState>>;

#[derive(Clone, Default)]
struct AppState {
    unfused_pokemon: HashMap<u16, Pokemon>,
}


#[tokio::main]
async fn main() {
    let shared_state = SharedState::default();

    // build our application with a single route
    let app = Router::new()
        .route("/", get(ping))
        .route("/pokemon/name/:id", get(name_handler))
        .route("/pokemon/fuse/:id1/:id2", get(fuse_handler))
        .with_state(Arc::clone(&shared_state));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}


async fn ping() -> &'static str {
    "Welcome to the pokemon fusion API!"
}

#[derive(Clone)]
struct Stats {
    hp: u16,
    attack: u16,
    defense: u16,
    special_attack: u16,
    special_defense: u16,
    speed: u16,
}

#[derive(Clone)]
struct Pokemon {
    name: String,
    stats: Stats,
    types: (String, String)
}

#[debug_handler]
async fn fuse_handler(Path((id1, id2)): Path<(u16, u16)>, State(state): State<SharedState>) -> String {

    let mut p1 : Option<Pokemon> = None;
    let mut p2 : Option<Pokemon> = None;

    {
        let db = &state.read().unwrap().unfused_pokemon;

        if let Some(pkmn) = db.get(&id1) {
            p1 = Some(pkmn.clone());
            println!("Using cache for pokemon 1.");
        }
    }

    if p1.is_none() {
        println!("Using API for pokemon 1.");
        let pokemon_1_r = get_pokemon_data(id1).await;
        match pokemon_1_r {
            Err(_err) => return "Failed to grab pokemon 1".to_string(),
            Ok(pkmn) => {
                state.write().unwrap().unfused_pokemon.insert(id1, pkmn.clone());
                p1 = Some(pkmn);
            },
        }
    }

    {
        let db = &state.read().unwrap().unfused_pokemon;

        if let Some(pkmn) = db.get(&id2) {
            p2 = Some(pkmn.clone());
            println!("Using cache for pokemon 2.");
        }
    }

    if p2.is_none() {
        println!("Using API for pokemon 2.");
        let pokemon_2_r = get_pokemon_data(id2).await;
        match pokemon_2_r {
            Err(_err) => return "Failed to grab pokemon 2".to_string(),
            Ok(pkmn) => {
                state.write().unwrap().unfused_pokemon.insert(id2, pkmn.clone());
                p2 = Some(pkmn);
            },
        }
    } 

    match p1 {
        Some(pkmn1) => {
            match p2 {
                Some(pkmn2) => {
                    format!("Fusing {} with {}", pkmn1.name, pkmn2.name)
                },
                None => {"Failed to get pokemon 2 data".to_string()}
            }
        },
        None => {"Failed to get pokemon 1 data".to_string()}
    }
}

async fn name_handler(Path(id): Path<u16>, State(state): State<SharedState>) -> String {
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