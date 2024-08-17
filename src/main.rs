use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use once_cell::sync::Lazy;
use serde_json::{json, Map, Value};

macro_rules! h200 {
    ($resp:expr) => {
        HttpResponse::Ok().body($resp)
    };
}

macro_rules! h403 {
    ($resp:expr) => {
        HttpResponse::BadRequest().body($resp)
    };
}

macro_rules! tplt {
    ($api:tt) => {
        std::fs::read_to_string(format!("api/{api}.json", api = $api)).map_or("".to_string(), |v| v)
    };
}

macro_rules! api {
    ($api:tt, $func:ident) => {
        #[post($api)]
        async fn $func(cache: web::Data<Map<String, Value>>) -> impl Responder {
            serde_json::to_string(&cache[(stringify!($func))])
                .map_or(h403! {"Invalid Template"}, |v| h200! {v})
        }
    };
}

static CONFIG: Lazy<Value> = Lazy::new(|| {
    std::fs::read_to_string("config.json").map_or(json!({}), |v| {
        serde_json::from_str(&v).map_or(json!({}), |v| v)
    })
});

#[post("/api/api/login")]
async fn alogin() -> impl Responder {
    h200! {"{\"page\":\"user_login\",\"error_flg\": 0}"}
}

api! {"/title/login", tlogin}
api! {"/mypage/mypage", mypage}
api! {"/library/librarytop/midoga_story/", midoga_story}
api! {"/library/librarytop/story/", story}
api! {"/library/librarytop/character_pictorial/", pictorial}

#[post("/library/librarytop")]
async fn library(
    params: web::Form<Map<String, Value>>,
    cache: web::Data<Map<String, Value>>,
) -> impl Responder {
    match params.get("view_type") {
        Some(Value::String(vtype)) => {
            cache["pictorial"]
                .as_object()
                .map_or(h403! {"Missing Template"}, |v| {
                    let mut p = Map::new();
                    let mut characters: Vec<Value> = Vec::new();
                    let mut cards: Vec<Value> = Vec::new();
                    match params["character_id"].as_str() {
                        Some(cid) => {
                            if vtype == "2" {
                                p.insert(
                                    "m_text".to_string(),
                                    json!({
                                        "serif": {
                                            "text": ""
                                        },
                                        "flavor": {
                                            "text": ""
                                        },
                                        "race": {
                                            "text": ""
                                        },
                                        "birthplace": {
                                            "text": ""
                                        },
                                        "favorite_things": {
                                            "text": ""
                                        },
                                        "appearance_scenario": {
                                            "text": ""
                                        },
                                        "profile_height": {
                                            "text": ""
                                        },
                                        "breast_size": {
                                            "text": ""
                                        },
                                        "hobby": {
                                            "text": ""
                                        },
                                        "related_characters": {
                                            "text": ""
                                        }
                                    }),
                                );
                                cards.push(cache[&format!("card_{cid}")].clone());
                            } else {
                                let c = cache[&format!("chara_{cid}")].clone();
                                p.insert(
                                    "m_text".to_string(),
                                    json!({
                                        "serif": {
                                            "text": c["m_text"]["serif"]
                                        },
                                        "flavor": {
                                            "text": c["m_text"]["flavor"]
                                        },
                                        "race": {
                                            "text": c["m_text"]["race"]
                                        },
                                        "birthplace": {
                                            "text": c["m_text"]["birthplace"]
                                        },
                                        "favorite_things": {
                                            "text": c["m_text"]["favorite_things"]
                                        },
                                        "appearance_scenario": {
                                            "text": c["m_text"]["appearance_scenario"]
                                        },
                                        "profile_height": {
                                            "text": c["m_text"]["profile_height"]
                                        },
                                        "breast_size": {
                                            "text": c["m_text"]["breast_size"]
                                        },
                                        "hobby": {
                                            "text": c["m_text"]["hobby"]
                                        },
                                        "related_characters": {
                                            "text": c["m_text"]["related_characters"]
                                        }
                                    }),
                                );
                                characters.push(c);
                            }
                        }
                        _ => {}
                    }
                    p.insert("chara_datas".to_string(), Value::Array(characters));
                    p.insert("card_datas".to_string(), Value::Array(cards));
                    p.insert("page".to_string(), json!("librarytop"));
                    p.insert("tips_data".to_string(), v["tips_data"].clone());
                    p.insert("user_data".to_string(), v["user_data"].clone());
                    serde_json::to_string(&p).map_or(h403! {"Invalid Template"}, |v| h200! {v})
                })
        }
        _ => {
            h403! {"Invalid Gallery Type"}
        }
    }
}

#[actix_web::main]
#[allow(unused_must_use)]
async fn main() -> std::io::Result<()> {
    let host = match CONFIG["host"].as_str() {
        Some(v) => v,
        _ => "127.0.0.1:443",
    };
    let wks = match CONFIG["workers"].as_u64() {
        Some(v) => v as usize,
        _ => 1,
    };
    println!("Game server started, plz check the following url address");
    println!("http://{}/app_data/index.html?app_domain=dmm.midogar.saikyo.biz&web_domain=res.midogar.saikyo.biz&device_id=1&galant_user_token=-1", host);
    HttpServer::new(|| {
        let mut engine = upon::Engine::new();
        let mut cache: Map<String, Value> = Map::new();
        for x in ["tlogin", "mypage", "midoga_story", "story", "pictorial"] {
            engine.add_template(x.to_string(), tplt! {x});
            engine.get_template(x).map_or({}, |template| {
                template.render(CONFIG.clone()).to_string().map_or({}, |s| {
                    match serde_json::from_str(&s) {
                        Ok(Value::Object(j)) => {
                            if x == "pictorial" {
                                j["character_pictorial_list"].as_array().map_or({}, |arr| {
                                    for chara in arr.iter() {
                                        chara["character_id"].as_str().map_or({}, |cid| {
                                            cache.insert(format!("chara_{cid}"), chara.clone());
                                        })
                                    }
                                });
                                j["card_datas"].as_array().map_or({}, |arr| {
                                    for chara in arr.iter() {
                                        chara["character_id"].as_str().map_or({}, |cid| {
                                            cache.insert(format!("card_{cid}"), chara.clone());
                                        })
                                    }
                                });
                            }
                            cache.insert(x.to_string(), Value::Object(j));
                        }
                        _ => {}
                    }
                });
            })
        }
        App::new()
            .app_data(web::Data::new(cache))
            .service(alogin)
            .service(tlogin)
            .service(mypage)
            .service(midoga_story)
            .service(story)
            .service(pictorial)
            .service(library)
            .service(actix_files::Files::new("/", "."))
    })
    .workers(wks)
    .bind(host)?
    .run()
    .await
}
