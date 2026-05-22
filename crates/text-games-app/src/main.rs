include!(concat!(env!("OUT_DIR"), "/ink_games.rs"));

fn main() {
    ink_dioxus::web::launch(
        ink_dioxus::web::WebLaunchConfig::new_catalog(INK_GAMES)
            .with_app_label("TEXT GAMES")
            .with_storage_key("text_games_app.web_save.v1")
            .with_default_prompt_title("Choices"),
    );
}
