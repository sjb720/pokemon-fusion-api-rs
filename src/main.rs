use axum::{
    debug_handler,
    routing::get,
    Router, extract::{ Path, State},
    http::{header, StatusCode, HeaderMap},
    Json, 
    body::StreamBody,
    response::IntoResponse,
};

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde_json::Value;
use serde::Serialize;
use tokio_util::io::ReaderStream;


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
        .route("/pokemon/fuse/:id1/:id2", get(fuse_handler))
        .route("/pokemon/fuse/:id1/:id2/image", get(img_handler))
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

#[derive(Serialize, Clone)]
struct Stats {
    hp: u16,
    attack: u16,
    defense: u16,
    special_attack: u16,
    special_defense: u16,
    speed: u16,
}

#[derive(Serialize, Clone)]
struct Pokemon {
    name: String,
    stats: Stats,
    types: (String, String)
}

#[debug_handler]
async fn img_handler(Path((id1, id2)): Path<(u16, u16)>) -> impl IntoResponse {
    let file_name = format!("{}.{}.png",id1, id2);

    let file = match tokio::fs::File::open(format!("assets/custom-fusions/{}", file_name)).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let mut headers = HeaderMap::new();

    headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());
    headers.insert(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_name).parse().unwrap());

    Ok((headers,body))
}

#[debug_handler]
async fn fuse_handler(Path((id1, id2)): Path<(u16, u16)>, State(state): State<SharedState>) -> Json<Pokemon> {

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
            Err(_err) => {},
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
            Err(_err) => {},
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
                    let fused = fuse_pokemon(&pkmn1, &pkmn2);
                    return Json(fused);
                },
                None => {
                    Json(
                        Pokemon { name: "MISSING NO".to_string(), stats: Stats { hp: 1, attack: 1, defense: 1, special_attack: 1, special_defense: 1, speed: 1 }, types: ("none".to_string(), "none".to_string()) }
                    )
                }
            }
        },
        None => {
            Json(
                Pokemon { name: "MISSING NO".to_string(), stats: Stats { hp: 1, attack: 1, defense: 1, special_attack: 1, special_defense: 1, speed: 1 }, types: ("none".to_string(), "none".to_string()) }
            )
        }
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

    let types: &Vec<Value> = data["types"].as_array().unwrap();
    let type_touple: (String, String);

    if types.len() == 1 {
        type_touple = (data["types"][0]["type"]["name"].as_str().unwrap_or_default().to_string(), "none".to_string())
    } else {
        type_touple = (
            data["types"][0]["type"]["name"].as_str().unwrap_or_default().to_string(), 
            data["types"][1]["type"]["name"].as_str().unwrap_or_default().to_string()
        )
    }

    Pokemon { name: data["name"].as_str().unwrap_or_default().to_string(), stats: Stats {
        hp: data["stats"][0]["base_stat"].as_u64().unwrap_or_default() as u16,
        attack: data["stats"][1]["base_stat"].as_u64().unwrap_or_default() as u16,
        defense: data["stats"][2]["base_stat"].as_u64().unwrap_or_default() as u16,
        special_attack: data["stats"][3]["base_stat"].as_u64().unwrap_or_default() as u16,
        special_defense: data["stats"][4]["base_stat"].as_u64().unwrap_or_default() as u16,
        speed: data["stats"][5]["base_stat"].as_u64().unwrap_or_default() as u16,
    }, types: type_touple }
}

fn fuse_pokemon(pokemon1: &Pokemon, pokemon2: &Pokemon) -> Pokemon {

    let second_fuse_type : String;
    match pokemon2.types.1.as_str() {
        "none" => {second_fuse_type = pokemon2.types.0.clone()},
        _ => {second_fuse_type= pokemon2.types.1.clone()}
    }

    Pokemon {
        name: format!("{}{}",&pokemon1.name[0..(pokemon1.name.len()/2)], &pokemon2.name[(pokemon2.name.len()/2)..pokemon2.name.len()] ),
        stats: Stats {
            attack: (pokemon1.stats.attack * 2 + pokemon2.stats.attack) / 3,
            defense: (pokemon1.stats.defense * 2 + pokemon2.stats.defense) / 3,
            hp: (pokemon1.stats.hp * 2 + pokemon2.stats.hp) / 3,
            special_attack: (pokemon1.stats.special_attack * 2 + pokemon2.stats.special_attack) / 3,
            special_defense: (pokemon1.stats.special_defense * 2 + pokemon2.stats.special_defense) / 3,
            speed: (pokemon1.stats.attack * 2 + pokemon2.stats.speed) / 3,
        },
        types: (
            pokemon1.types.0.clone(),
            second_fuse_type
        )
    }
}
